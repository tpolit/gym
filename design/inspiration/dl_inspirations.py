#!/usr/bin/env python3
"""
Télécharge les captures d'écran officielles de l'App Store pour une liste d'apps.
Usage personnel (inspiration / veille UX). Les images restent la propriété de leurs auteurs :
le dossier "captures/" est ignoré par Git et ne doit pas être publié.

Utilisation :
    python3 dl_inspirations.py
Résultat : un dossier "captures/<NomApp>/" à côté de ce script, avec les captures numérotées.
Aucune dépendance à installer (bibliothèque standard uniquement).
Hôtes contactés : itunes.apple.com (recherche), *.mzstatic.com (images).
"""
import json
import re
import time
import urllib.parse
import urllib.request
from pathlib import Path

PAYS_PAR_DEFAUT = "fr"
PAYS_DE_SECOURS = "us"
DOSSIER = Path(__file__).resolve().parent / "captures"
LARGEUR = 600  # largeur des captures téléchargées (px)

# nom affiché -> (terme de recherche, mot attendu dans le nom de l'app trouvée)
APPS = {
    "Revolut": ("Revolut", "revolut"),
    "Cash App": ("Cash App", "cash app"),
    "Splitwise": ("Splitwise", "splitwise"),
    "Things 3": ("Things 3", "things"),
    "Uber": ("Uber - Request a ride", "uber"),
    "Uber Eats": ("Uber Eats", "uber eats"),
    "Flighty": ("Flighty", "flighty"),
    "Citymapper": ("Citymapper", "citymapper"),
    "Spotify": ("Spotify", "spotify"),
    "Todoist": ("Todoist", "todoist"),
    "Duolingo": ("Duolingo", "duolingo"),
    "Oura": ("Oura", "oura"),
    "Whoop": ("WHOOP", "whoop"),
    "Gentler Streak": ("Gentler Streak", "gentler"),
    "Robinhood": ("Robinhood", "robinhood"),
    "Strava": ("Strava", "strava"),
    "Nike Training Club": ("Nike Training Club", "nike"),
    "Opal": ("Opal Screen Time", "opal"),
    "Finch": ("Finch Self-Care Pet", "finch"),
    "Hevy": ("Hevy Workout Tracker", "hevy"),
    "Strong": ("Strong Workout Tracker Gym Log", "strong"),
    "Alpha Progression": ("Alpha Progression", "alpha progression"),
}


def chercher(terme, attendu, pays):
    url = "https://itunes.apple.com/search?" + urllib.parse.urlencode(
        {"term": terme, "entity": "software", "country": pays, "limit": 10}
    )
    with urllib.request.urlopen(url, timeout=20) as r:
        resultats = json.load(r)["results"]
    for app in resultats:
        if attendu in app.get("trackName", "").lower() and app.get("screenshotUrls"):
            return app
    return None


def agrandir(url):
    # Les URL mzstatic contiennent la taille (ex. ".../392x696bb.jpg") : on demande une largeur fixe.
    return re.sub(r"/\d+x\d+(bb|w)?\.(\w+)$", rf"/{LARGEUR}x0w.\2", url)


def telecharger(url, chemin):
    req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(req, timeout=30) as r:
        chemin.write_bytes(r.read())


def main():
    DOSSIER.mkdir(exist_ok=True)
    for nom, (terme, attendu) in APPS.items():
        try:
            app = chercher(terme, attendu, PAYS_PAR_DEFAUT) or chercher(terme, attendu, PAYS_DE_SECOURS)
            if not app:
                print(f"[?] {nom} : introuvable")
                continue
            dossier = DOSSIER / re.sub(r"[^\w\- ]", "", nom)
            dossier.mkdir(exist_ok=True)
            urls = app["screenshotUrls"]
            for i, u in enumerate(urls, 1):
                ext = u.rsplit(".", 1)[-1].split("?")[0] or "jpg"
                chemin = dossier / f"{i:02d}.{ext}"
                if chemin.exists():
                    continue
                try:
                    telecharger(agrandir(u), chemin)
                except Exception:
                    telecharger(u, chemin)  # repli sur la taille d'origine
            print(f"[ok] {nom} -> {app['trackName']} ({len(urls)} captures)")
        except Exception as e:
            print(f"[x] {nom} : {e}")
        time.sleep(1)  # on reste poli avec l'API
    print(f"\nTerminé. Regarde dans le dossier : {DOSSIER}")


if __name__ == "__main__":
    main()
