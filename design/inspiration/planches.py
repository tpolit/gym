#!/usr/bin/env python3
"""Génère des planches HTML (une ligne par app) à partir de captures/, pour les étudier d'un coup d'œil.
Sortie : captures/_planches/<n>.html (à photographier avec Chrome headless, voir UX.md §4)."""
from pathlib import Path

RACINE = Path(__file__).resolve().parent / "captures"
SORTIE = RACINE / "_planches"

GROUPES = [
    (["Hevy", "Strong"], 255),
    (["Alpha Progression"], 255),
    (["Revolut", "Cash App"], 175),
    (["Splitwise", "Things 3"], 200),
    (["Uber", "Uber Eats"], 200),
    (["Flighty", "Citymapper"], 175),
    (["Spotify", "Todoist"], 200),
    (["Duolingo", "Gentler Streak"], 175),
    (["Oura", "Whoop"], 175),
    (["Robinhood", "Strava"], 175),
    (["Nike Training Club", "Opal", "Finch"], 175),
]


def main():
    SORTIE.mkdir(exist_ok=True)
    for n, (apps, largeur) in enumerate(GROUPES, 1):
        lignes = []
        for app in apps:
            images = sorted(p for p in (RACINE / app).iterdir() if p.suffix in {".jpg", ".png", ".webp"})
            balises = "".join(f'<img src="{p.as_uri()}" style="width:{largeur}px">' for p in images)
            lignes.append(f'<h2>{app}</h2><div class="row">{balises}</div>')
        html = (
            "<!doctype html><meta charset='utf-8'><style>"
            "body{margin:12px;background:#888;font-family:sans-serif}"
            "h2{margin:6px 0;font-size:22px}"
            ".row{display:flex;gap:6px;align-items:flex-start}"
            "img{display:block;border-radius:6px}"
            "</style>" + "".join(lignes)
        )
        (SORTIE / f"{n:02d}.html").write_text(html, encoding="utf-8")
        print(f"{n:02d}: {', '.join(apps)}")


if __name__ == "__main__":
    main()
