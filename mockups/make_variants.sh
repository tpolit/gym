#!/bin/sh
# Génère les variantes de couleur des maquettes à partir de la palette de référence (v2-abricot).
# Usage : sh mockups/make_variants.sh   (puis sh mockups/shoot.sh pour les captures)
cd "$(dirname "$0")" || exit 1

# Pas d'étiquettes d'inspiration dans les maquettes v2
sed -i '/class="annot"/d' v2/*.html v2-abricot/*.html

# Thème « Sauge » (gardé pour un éventuel choix de thèmes) : accent sauge, records de nouveau en or
rm -rf v2-sauge
mkdir -p v2-sauge
cp -r v2-abricot/fonts v2-abricot/style.css v2-sauge/
for f in accueil exercice clavier seance recap stats; do cp "v2-abricot/$f.html" v2-sauge/; done
sed -i \
  -e 's/#ffae80/#a9d6a0/gI' -e 's/rgba(255,174,128/rgba(169,214,160/g' -e 's/#3a281d/#22301f/gI' \
  -e 's/#c3a6ff/#ffc542/gI' -e 's/#e6dbff/#ffe08a/gI' -e 's/#7a5cc7/#b8860b/gI' -e 's/#2a1a55/#3a2a00/gI' \
  -e 's/« Abricot »/« Sauge »/' \
  v2-sauge/style.css v2-sauge/*.html
