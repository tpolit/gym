#!/usr/bin/env python3
"""Generates crates/domain/src/catalog.json from free-exercise-db and tools/pool/selection.json.

Usage: python3 tools/pool/build_catalog.py [--download]
  --download  fetches the pinned free-exercise-db version into target/pool/ (host raw.githubusercontent.com).

selection.json lists the exercises kept, in display-independent order. Each entry has an `id`, a
French `name` and optional extra `aliases`. With a `source` (free-exercise-db id), category, muscle
group, equipment and measure are derived from the source and may be overridden one by one; the English
source name is added as the first alias. Without a source (families the database lacks), all four
fields are required. `"retired": true` hides an exercise from the lists while keeping it readable in
past sessions: pool ids are permanent, never deleted.
"""

import argparse
import collections
import json
import os
import re
import sys
import urllib.request

SOURCE_COMMIT = "f00c92c7dcf1216a928a52c3706c7ce8e2f71ed5"
SOURCE_URL = f"https://raw.githubusercontent.com/yuhonas/free-exercise-db/{SOURCE_COMMIT}/dist/exercises.json"

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
SOURCE_PATH = os.path.join(ROOT, "target", "pool", "exercises.json")
SELECTION_PATH = os.path.join(ROOT, "tools", "pool", "selection.json")
CATALOG_PATH = os.path.join(ROOT, "crates", "domain", "src", "catalog.json")

# The database has no mobility category: mobility entries are overridden from `stretching`.
CATEGORIES = {"strength": "strength", "powerlifting": "strength", "stretching": "stretching", "cardio": "cardio"}
EQUIPMENT = {
    "barbell": "barbell", "e-z curl bar": "ez_bar", "dumbbell": "dumbbell", "kettlebells": "kettlebell",
    "machine": "machine", "cable": "cable", "body only": "bodyweight", None: "bodyweight", "bands": "band",
    "medicine ball": "medicine_ball", "exercise ball": "other", "foam roll": "other", "other": "other",
}
MUSCLES = {
    "chest": "chest", "lats": "back", "middle back": "back", "lower back": "lower_back", "traps": "traps",
    "shoulders": "shoulders", "biceps": "biceps", "triceps": "triceps", "forearms": "forearms",
    "abdominals": "abs", "glutes": "glutes", "quadriceps": "quads", "hamstrings": "hamstrings",
    "adductors": "adductors", "abductors": "abductors", "calves": "calves", "neck": "neck",
}
FIELDS = ("category", "muscle_group", "equipment", "measure")
ENTRY_KEYS = {"source", "id", "name", "aliases", "retired", *FIELDS}


def download():
    os.makedirs(os.path.dirname(SOURCE_PATH), exist_ok=True)
    with urllib.request.urlopen(SOURCE_URL) as response, open(SOURCE_PATH, "wb") as output:
        output.write(response.read())
    print(f"Downloaded {SOURCE_URL}")


def default_measure(category, equipment, is_static):
    if category == "cardio":
        return "cardio"
    if category == "stretching" or is_static:
        return "duration"
    if category == "mobility" or equipment in ("bodyweight", "band"):
        return "reps"
    return "weight_reps"


# Category and equipment overrides are applied before the measure is derived, so that e.g. a
# stretching entry moved to mobility gets the mobility default measure.
def derived_fields(source, entry):
    category = CATEGORIES.get(source["category"])
    if category is None:
        raise ValueError(f"excluded category {source['category']!r}")
    equipment = EQUIPMENT[source["equipment"]]
    if equipment == "machine" and "smith" in source["id"].lower():
        equipment = "smith_machine"
    base = {"category": category, "muscle_group": MUSCLES[source["primaryMuscles"][0]], "equipment": equipment}
    fields = {field: entry.get(field, value) for field, value in base.items()}
    base["measure"] = default_measure(fields["category"], fields["equipment"], source["force"] == "static")
    fields["measure"] = entry.get("measure", base["measure"])
    useless = [field for field in FIELDS if field in entry and entry[field] == base[field]]
    return fields, useless


def readable_source_name(name):
    return re.sub(r"\s+", " ", name.replace(" - ", " ")).strip()


def build(selection, sources):
    catalog, errors = [], []
    used_ids, used_sources = set(), set()
    for entry in selection:
        label = entry.get("id", "?")
        unknown = set(entry) - ENTRY_KEYS
        if unknown:
            errors.append(f"{label}: unknown keys {sorted(unknown)}")
            continue
        if label in used_ids:
            errors.append(f"{label}: duplicate id")
        if entry.get("retired", True) is not True:
            errors.append(f"{label}: `retired` is either true or absent")
        used_ids.add(label)
        aliases = list(entry.get("aliases", []))
        source_id = entry.get("source")
        if source_id is None:
            missing = [field for field in FIELDS if field not in entry]
            if missing:
                errors.append(f"{label}: no source, missing {missing}")
                continue
            fields = {field: entry[field] for field in FIELDS}
        else:
            source = sources.get(source_id)
            if source is None:
                errors.append(f"{label}: unknown source {source_id!r}")
                continue
            if source_id in used_sources:
                errors.append(f"{label}: source {source_id!r} used twice")
            used_sources.add(source_id)
            try:
                fields, useless = derived_fields(source, entry)
            except ValueError as error:
                errors.append(f"{label}: {error}")
                continue
            errors.extend(f"{label}: useless override of {field}" for field in useless)
            aliases.insert(0, readable_source_name(source["name"]))
        unique_aliases = []
        for alias in aliases:
            if alias.lower() != entry["name"].lower() and alias.lower() not in map(str.lower, unique_aliases):
                unique_aliases.append(alias)
        retired = {"retired": True} if entry.get("retired") else {}
        catalog.append({"id": label, "name": entry["name"], "aliases": unique_aliases, **fields, **retired})
    return catalog, errors


def family(exercise):
    if exercise["category"] == "strength":
        return "bodyweight" if exercise["equipment"] == "bodyweight" else "gym"
    return exercise["category"]


def write_catalog(catalog):
    lines = [json.dumps(exercise, ensure_ascii=False, separators=(", ", ": ")) for exercise in catalog]
    with open(CATALOG_PATH, "w", encoding="utf-8") as output:
        output.write("[\n  " + ",\n  ".join(lines) + "\n]\n")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--download", action="store_true")
    args = parser.parse_args()
    if args.download:
        download()
    if not os.path.exists(SOURCE_PATH):
        sys.exit(f"Missing {SOURCE_PATH}: run with --download.")
    sources = {exercise["id"]: exercise for exercise in json.load(open(SOURCE_PATH, encoding="utf-8"))}
    selection = json.load(open(SELECTION_PATH, encoding="utf-8"))
    catalog, errors = build(selection, sources)
    if errors:
        sys.exit("\n".join(errors))
    write_catalog(catalog)
    from_source = sum(1 for entry in selection if entry.get("source"))
    shown = [exercise for exercise in catalog if not exercise.get("retired")]
    print(f"{CATALOG_PATH}: {len(catalog)} exercises ({from_source} from free-exercise-db, "
          f"{len(catalog) - from_source} written for the project), {len(catalog) - len(shown)} retired")
    for name, count in collections.Counter(map(family, shown)).most_common():
        print(f"  {name:<12} {count}")


if __name__ == "__main__":
    main()
