#!/bin/sh
# Régénère les captures PNG des maquettes (Chrome headless ; hors sandbox).
# Usage : sh mockups/shoot.sh [v1|v2|v2-abricot|v2-sauge]   (par défaut : tout)
# Les variantes de couleur se génèrent d'abord avec : sh mockups/make_variants.sh
cd "$(dirname "$0")" || exit 1
shoot() {
  google-chrome --headless=new --disable-gpu --hide-scrollbars \
    --user-data-dir=/tmp/gym-chrome --force-device-scale-factor=2 --window-size="${3:-390,844}" \
    --screenshot="$PWD/$2" "file://$PWD/$1" 2>&1 | grep -o 'written.*'
}
if [ -z "$1" ] || [ "$1" = "v1" ]; then
  for f in accueil seance exercice exercice-duree selecteur historique stats-exercice; do
    shoot "$f.html" "png/$f.png"
  done
fi
for d in v2 v2-abricot v2-sauge; do
  if [ -z "$1" ] || [ "$1" = "$d" ]; then
    mkdir -p "$d/png"
    for f in accueil exercice clavier seance recap stats; do
      shoot "$d/$f.html" "$d/png/$f.png"
    done
  fi
done
