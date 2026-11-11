#!/usr/bin/env python3
"""UI smoke test in headless Chrome (outside the sandbox), without dependencies.

Usage: python3 tools/ui_smoke.py [dist_dir] [screenshots_dir]
Serves the Trunk build under /gym/, drives Chrome through the DevTools protocol
(minimal WebSocket client from the standard library), clicks like a user
and saves a PNG screenshot at each step.
"""

import base64
import functools
import http.server
import json
import os
import shutil
import socket
import struct
import subprocess
import sys
import tempfile
import threading
import time
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DIST = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, "dist"))
SHOTS = os.path.abspath(sys.argv[2] if len(sys.argv) > 2 else os.path.join(ROOT, "target", "ui-smoke"))
HTTP_PORT = 8766
DEVTOOLS_PORT = 9333


class DevTools:
    def __init__(self, websocket_url):
        address, path = websocket_url[len("ws://"):].split("/", 1)
        host, port = address.split(":")
        self.socket = socket.create_connection((host, int(port)))
        key = base64.b64encode(os.urandom(16)).decode()
        self.socket.sendall(
            f"GET /{path} HTTP/1.1\r\nHost: {address}\r\nUpgrade: websocket\r\n"
            f"Connection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n".encode()
        )
        response = b""
        while b"\r\n\r\n" not in response:
            response += self.socket.recv(1)
        self.next_id = 0

    def _send(self, text):
        data = text.encode()
        mask = os.urandom(4)
        header = bytearray([0x81])
        if len(data) < 126:
            header.append(0x80 | len(data))
        elif len(data) < 65536:
            header.append(0x80 | 126)
            header += struct.pack(">H", len(data))
        else:
            header.append(0x80 | 127)
            header += struct.pack(">Q", len(data))
        masked = bytes(byte ^ mask[index % 4] for index, byte in enumerate(data))
        self.socket.sendall(bytes(header) + mask + masked)

    def _read_exact(self, count):
        buffer = b""
        while len(buffer) < count:
            buffer += self.socket.recv(count - len(buffer))
        return buffer

    def _receive(self):
        message = b""
        while True:
            first, second = self._read_exact(2)
            length = second & 0x7F
            if length == 126:
                length = struct.unpack(">H", self._read_exact(2))[0]
            elif length == 127:
                length = struct.unpack(">Q", self._read_exact(8))[0]
            message += self._read_exact(length)
            if first & 0x80:
                return json.loads(message)

    def call(self, method, **params):
        self.next_id += 1
        self._send(json.dumps({"id": self.next_id, "method": method, "params": params}))
        while True:
            reply = self._receive()
            if reply.get("id") == self.next_id:
                if "error" in reply:
                    raise RuntimeError(f"{method}: {reply['error']}")
                return reply.get("result", {})

    def evaluate(self, expression):
        result = self.call("Runtime.evaluate", expression=expression, returnByValue=True, awaitPromise=True)
        if "exceptionDetails" in result:
            raise RuntimeError(result["exceptionDetails"])
        return result["result"].get("value")


class QuietHandler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *args):
        pass


HELPERS = r"""
window.smoke = {
  click(text) {
    const target = [...document.querySelectorAll('button')].find(b => b.textContent.replace(/\s+/g, ' ').includes(text));
    if (!target) throw new Error('Button not found: ' + text);
    target.click();
    return target.textContent.trim();
  },
  key(text) {
    const target = [...document.querySelectorAll('.keypad__key')].find(b => b.textContent.trim() === text);
    if (!target) throw new Error('Key not found: ' + text);
    target.click();
  },
  clickSelector(selector, index = 0) {
    const target = document.querySelectorAll(selector)[index];
    if (!target) throw new Error('Element not found: ' + selector);
    target.click();
  },
  pointer(selector, index, dx) {
    const target = document.querySelectorAll(selector)[index];
    if (!target) throw new Error('Element not found: ' + selector);
    const rect = target.getBoundingClientRect();
    const x = rect.left + rect.width / 2, y = rect.top + rect.height / 2;
    const at = (clientX) => ({ bubbles: true, clientX, clientY: y, pointerId: 1, pointerType: 'touch', isPrimary: true });
    target.dispatchEvent(new PointerEvent('pointerdown', at(x)));
    if (dx) target.dispatchEvent(new PointerEvent('pointermove', at(x + dx)));
    target.dispatchEvent(new PointerEvent('pointerup', at(x + dx)));
  },
  select(selector, value) {
    const field = document.querySelector(selector);
    if (!field) throw new Error('Element not found: ' + selector);
    field.value = value;
    field.dispatchEvent(new Event('change', { bubbles: true }));
  },
  type(selector, text) {
    const input = document.querySelector(selector);
    input.value = text;
    input.dispatchEvent(new Event('input', { bubbles: true }));
  },
  text() { return document.body.innerText; },
};
"""


def main():
    os.makedirs(SHOTS, exist_ok=True)
    serve_root = tempfile.mkdtemp()
    os.symlink(DIST, os.path.join(serve_root, "gym"))
    handler = functools.partial(QuietHandler, directory=serve_root)
    server = http.server.ThreadingHTTPServer(("127.0.0.1", HTTP_PORT), handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()

    profile = tempfile.mkdtemp()
    chrome = subprocess.Popen(
        ["google-chrome", "--headless=new", "--disable-gpu", f"--remote-debugging-port={DEVTOOLS_PORT}",
         f"--user-data-dir={profile}", "about:blank"],
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    try:
        for _ in range(50):
            try:
                pages = json.load(urllib.request.urlopen(f"http://127.0.0.1:{DEVTOOLS_PORT}/json/list"))
                page = next(p for p in pages if p["type"] == "page")
                break
            except Exception:
                time.sleep(0.2)
        devtools = DevTools(page["webSocketDebuggerUrl"])
        devtools.call("Emulation.setDeviceMetricsOverride", width=390, height=844, deviceScaleFactor=2, mobile=True)
        devtools.call("Page.navigate", url=f"http://127.0.0.1:{HTTP_PORT}/gym/")
        time.sleep(2)
        devtools.evaluate(HELPERS)

        def shot(name):
            time.sleep(0.4)
            data = devtools.call("Page.captureScreenshot", format="png")["data"]
            with open(os.path.join(SHOTS, f"{name}.png"), "wb") as file:
                file.write(base64.b64decode(data))
            print(f"screenshot: {name}.png")

        def step(expression):
            devtools.evaluate(f"smoke.{expression}")
            time.sleep(0.3)

        run_scenario(step, shot, devtools)
        print("Smoke test passed.")
    finally:
        chrome.terminate()
        server.shutdown()
        shutil.rmtree(profile, ignore_errors=True)
        shutil.rmtree(serve_root, ignore_errors=True)


def run_scenario(step, shot, devtools):
    shot("01-accueil-vide")
    text = devtools.evaluate("smoke.text()").lower()
    assert "comment vous appelez-vous" in text, text
    step("type('.home input', 'Thomas')")
    step("click('Enregistrer')")
    shot("01b-accueil-prenom")
    text = devtools.evaluate("smoke.text()").lower()
    assert "bonjour thomas" in text and "comment vous appelez-vous" not in text, text
    assert "quel est votre poids de corps" in text, text
    step("type('.body-weight input', '7')")
    step("click('Enregistrer')")
    text = devtools.evaluate("smoke.text()").lower()
    assert "entre 20 et 300 kg" in text, text
    step("type('.body-weight input', '80')")
    step("click('Enregistrer')")
    shot("01c-accueil-poids")
    text = devtools.evaluate("smoke.text()").lower()
    assert "quel est votre poids de corps" not in text, text
    step("clickSelector('.home__greeting')")
    text = devtools.evaluate("document.querySelector('.sheet input').value")
    assert text == "Thomas", text
    step("click('Annuler')")
    step("click('Démarrer une séance')")
    shot("02-nouvelle-seance")
    step("click('Démarrer')")
    shot("03-seance-vide")
    step("click('Ajouter un exercice')")
    step("type('.picker input', 'developpe couche')")
    shot("04-selecteur")
    step("click('Développé couché barre')")
    step("clickSelector('.stepper__value', 0)")
    for key in ["8", "2", ",", "5"]:
        step(f"key('{key}')")
    shot("05-clavier-poids")
    step("click('OK')")
    step("key('8')")
    step("click('OK')")
    shot("06-serie-en-cours")
    step("click('Valider la série 1')")
    step("clickSelector('.stepper__button', 1)")
    step("click('Valider la série 2')")
    shot("07-deux-series")
    step("pointer('.exercise .swipe__content', 0, -120)")
    shot("08-confirmation")
    text = devtools.evaluate("smoke.text()").lower()
    assert "supprimer la série « 82,5 × 8 » ?" in text, text
    step("click('Annuler')")
    count = devtools.evaluate("document.querySelectorAll('.exercise .set--done').length")
    assert count == 2, count
    step("click('Exercices de la séance')")
    shot("09-tiroir")
    step("click('Terminer la séance')")
    shot("10-fin-de-seance")
    step("click('Terminé')")
    shot("11-accueil")
    text = devtools.evaluate("smoke.text()").lower()
    assert "refaire une séance" in text and "séance du" in text, text

    step("click('Démarrer une séance')")
    step("click('Démarrer')")
    step("click('Ajouter un exercice')")
    step("click('Développé couché barre')")
    shot("12-derniere-fois")
    text = devtools.evaluate("smoke.text()").lower()
    assert "dernière fois" in text and "82,5 × 8, 83 × 8" in text, text
    assert "valider la série 1" in text, text

    step("clickSelector('.topbar .icon-btn')")
    shot("13-accueil-mini-lecteur")
    text = devtools.evaluate("smoke.text()").lower()
    assert "cette semaine" in text and "définir un objectif" in text, text
    step("clickSelector('.mini-player')")
    step("clickSelector('.stepper__button', 1)")
    step("clickSelector('.stepper__button', 1)")
    step("click('Valider la série 1')")
    shot("13b-record")
    text = devtools.evaluate("smoke.text()").lower()
    assert "record" in text and "poids max 83,5 kg (avant 83 kg)" in text, text
    for index in range(1, 12):
        step("click('Exercices de la séance')")
        step("click('+ Ajouter un exercice')")
        step(f"clickSelector('.picker__item', {index})")
    step("click('Exercices de la séance')")
    shot("13c-tiroir-long")
    scrolling = devtools.evaluate(
        "(() => { const list = document.querySelector('.drawer__list');"
        " return [document.querySelectorAll('.drawer__entry').length, getComputedStyle(list).overflowY,"
        " list.scrollHeight > list.clientHeight]; })()"
    )
    assert scrolling == [12, "auto", True], scrolling
    last_name = devtools.evaluate(
        "(() => { const entries = document.querySelectorAll('.drawer__entry');"
        " const last = entries[entries.length - 1]; last.scrollIntoView();"
        " return last.querySelector('.drawer__entry-name').textContent; })()"
    )
    shot("13d-tiroir-defile")
    step("clickSelector('.drawer__select', 11)")
    current_name = devtools.evaluate("document.querySelector('.exercise__name').textContent")
    assert current_name == last_name, (current_name, last_name)
    step("click('Exercices de la séance')")
    step("pointer('.drawer__entry .swipe__content', 11, -120)")
    shot("13d2-retirer-exercice")
    text = devtools.evaluate("smoke.text()").lower()
    assert f"retirer « {last_name.lower()} » de la séance ?" in text, text
    step("clickSelector('.confirm__action')")
    count = devtools.evaluate("document.querySelectorAll('.drawer__entry').length")
    assert count == 11, count
    step("click('Terminer la séance')")
    shot("13e-recap-records")
    text = devtools.evaluate("smoke.text()").lower()
    assert "records battus" in text, text
    step("click('Terminé')")

    step("clickSelector('.weekly')")
    step("click('3')")
    shot("13f-objectif")
    text = devtools.evaluate("smoke.text()").lower()
    assert "2 / 3 séances cette semaine" in text, text

    step("click('Historique')")
    shot("14-historique")
    step("select('main select', '0')")
    shot("14b-historique-filtre")
    text = devtools.evaluate("smoke.text()").lower()
    assert "séances avec développé couché barre" in text and text.count("séance du") >= 2, text
    assert "1 série · 668 kg" in text, text
    step("clickSelector('.history__row')")
    step("clickSelector('.topbar .icon-btn')")
    text = devtools.evaluate("smoke.text()").lower()
    assert "séances avec développé couché barre" in text, text
    step("clickSelector('.history__row')")
    shot("15-detail-seance")
    text = devtools.evaluate("smoke.text()").lower()
    assert "83,5 × 8" in text and " · 668 kg" in text, text
    step("click('Ajouter une série')")
    step("click('Enregistrer')")
    step("pointer('.history__exercise .swipe__content', 0, 120)")
    shot("15b-detail-corrige")
    text = devtools.evaluate("smoke.text()").lower()
    assert text.count("83,5 × 8") == 2 and "é\n" in text, text
    bar = devtools.evaluate("document.querySelector('.topbar__title').className")
    assert "topbar__title--hidden" in bar, bar
    devtools.call("Emulation.setDeviceMetricsOverride", width=390, height=420, deviceScaleFactor=2, mobile=True)
    devtools.evaluate("window.scrollTo(0, document.body.scrollHeight)")
    shot("15c-barre-fixe")
    bar = devtools.evaluate(
        "(() => { const bar = document.querySelector('.topbar'); const title = bar.querySelector('.topbar__title');"
        " return [window.scrollY > 100, bar.getBoundingClientRect().top, title.className, title.textContent]; })()"
    )
    assert bar[0] and bar[1] == 0 and "hidden" not in bar[2] and bar[3].startswith("Séance du"), bar
    devtools.call("Emulation.setDeviceMetricsOverride", width=390, height=844, deviceScaleFactor=2, mobile=True)
    devtools.evaluate("window.scrollTo(0, 0)")
    step("click('Planifier à partir de cette séance')")
    step("click('Planifier')")
    shot("16-seance-planifiee")
    step("type('main input[type=text]', 'Push B')")
    step("clickSelector('.topbar__action')")
    text = devtools.evaluate("smoke.text()").lower()
    assert "refaire cette séance" in text, text
    step("clickSelector('.topbar .icon-btn')")
    step("click('Accueil')")
    step("click('Push B')")
    name = devtools.evaluate("document.querySelector('main input[type=text]').value")
    assert name == "Push B", name
    step("click('Lancer la séance')")
    shot("17-lancer")
    step("click('C\\'est parti')")
    text = devtools.evaluate("smoke.text()").lower()
    assert "développé couché barre" in text and "dernière fois" in text, text
    step("click('Exercices de la séance')")
    step("click('Abandonner la séance')")
    shot("17b-abandonner")
    text = devtools.evaluate("smoke.text()").lower()
    assert "abandonner la séance ?" in text, text
    step("clickSelector('.confirm__action')")
    shot("18-accueil-planifiee")
    text = devtools.evaluate("smoke.text()").lower()
    assert "prochaines séances" in text and "lancer" in text and "refaire une séance" in text, text

    step("click('Stats')")
    shot("18b-stats")
    text = devtools.evaluate("smoke.text()").lower()
    assert "séances par semaine" in text and "pectoraux" in text, text
    step("clickSelector('.picker__item', 0)")
    shot("18c-stats-exercice")
    text = devtools.evaluate("smoke.text()").lower()
    assert "1rm estimé" in text and "poids max" in text, text
    step("click('Voir les séances avec cet exercice')")
    text = devtools.evaluate("smoke.text()").lower()
    assert "séances avec développé couché barre" in text, text
    step("clickSelector('.tab-bar__tab', 2)")
    step("clickSelector('.picker__item', 0)")

    step("clickSelector('.topbar .icon-btn')")
    step("click('Accueil')")
    step("clickSelector('.home__header .icon-btn')")
    shot("18d-reglages")
    weight = devtools.evaluate("document.querySelector('.body-weight input').value")
    assert weight == "80", weight
    step("type('.body-weight input', '79,5')")
    step("click('Enregistrer')")
    text = devtools.evaluate("smoke.text()").lower()
    assert "historique du poids" in text and "79,5 kg" in text and "80 kg" not in text, text
    assert "exercise data by repdb (repdb.co)" in text, text
    step("click('Sauvegarde des données')")
    step("click('Exporter mes données')")
    shot("18e-sauvegarde")
    text = devtools.evaluate("smoke.text()").lower()
    assert "fichier de sauvegarde créé" in text, text
    step("clickSelector('.topbar .icon-btn')")
    step("clickSelector('.topbar .icon-btn')")

    step("click('Exercices')")
    shot("19-catalogue")
    step("type('main input.input', 'alternate knees to floor')")
    shot("19b-catalogue-mobilite")
    text = devtools.evaluate("smoke.text()").lower()
    assert "bascule des fléchisseurs de hanche" in text, text
    step("type('main input.input', 'rouleau de massage')")
    text = devtools.evaluate("smoke.text()").lower()
    assert "rouleau de massage" not in text, text
    step("type('main input.input', '')")
    step("click('Créer un exercice')")
    step("type('input[type=text]', 'Rowing Technogym')")
    shot("20-nouvel-exercice")
    step("click('Enregistrer')")
    shot("21-detail-exercice")
    text = devtools.evaluate("smoke.text()").lower()
    assert "rowing technogym" in text and "jamais pratiqué" in text, text

    step("clickSelector('.topbar .icon-btn')")
    step("click('Accueil')")
    step("clickSelector('.home__header .icon-btn')")
    step("click('Sauvegarde des données')")
    document = devtools.call("DOM.getDocument")
    file_input = devtools.call("DOM.querySelector", nodeId=document["root"]["nodeId"], selector="input[type=file]")
    fixture = os.path.join(ROOT, "crates", "persistence", "fixtures", "v1.json")
    devtools.call("DOM.setFileInputFiles", files=[fixture], nodeId=file_input["nodeId"])
    time.sleep(1)
    shot("22-import")
    text = devtools.evaluate("smoke.text()").lower()
    assert "remplacer vos données" in text and "1 séance terminée, 2 exercices perso" in text, text
    step("click('Remplacer')")
    shot("23-apres-import")
    text = devtools.evaluate("smoke.text()").lower()
    assert "reprendre la séance" in text and "jambes" in text, text


if __name__ == "__main__":
    main()
