#!/usr/bin/env python3
"""Size of the .wasm by crate and by function (a minimal `twiggy`, standard library only).

Usage: python3 tools/wasm_size.py [path.wasm] [--top N] [--crate NAME]
Default path: the raw release build (`target/wasm32-unknown-unknown/release/ui_leptos.wasm`),
which still has its `name` section; the file shipped in `dist/` has none (wasm-opt strips it).
"""

import argparse
import collections
import re
import sys

DEFAULT_PATH = "target/wasm32-unknown-unknown/release/ui_leptos.wasm"


def read_leb(data, offset):
    result = shift = 0
    while True:
        byte = data[offset]
        offset += 1
        result |= (byte & 0x7F) << shift
        shift += 7
        if byte < 0x80:
            return result, offset


def read_name(data, offset):
    length, offset = read_leb(data, offset)
    return data[offset:offset + length].decode("utf-8", "replace"), offset + length


def sections(data):
    offset = 8
    while offset < len(data):
        section_id = data[offset]
        size, offset = read_leb(data, offset + 1)
        yield section_id, data[offset:offset + size]
        offset += size


def imported_function_count(payload):
    count, offset = read_leb(payload, 0)
    functions = 0
    for _ in range(count):
        _, offset = read_name(payload, offset)
        _, offset = read_name(payload, offset)
        kind = payload[offset]
        offset += 1
        if kind == 0:
            functions += 1
            _, offset = read_leb(payload, offset)
        elif kind == 1:
            offset += 1
            flags, offset = read_leb(payload, offset)
            _, offset = read_leb(payload, offset)
            if flags & 1:
                _, offset = read_leb(payload, offset)
        elif kind == 2:
            flags, offset = read_leb(payload, offset)
            _, offset = read_leb(payload, offset)
            if flags & 1:
                _, offset = read_leb(payload, offset)
        elif kind == 3:
            offset += 2
        else:
            _, offset = read_leb(payload, offset)
    return functions


def function_names(payload):
    names = {}
    offset = 0
    while offset < len(payload):
        subsection = payload[offset]
        size, offset = read_leb(payload, offset + 1)
        end = offset + size
        if subsection == 1:
            count, cursor = read_leb(payload, offset)
            for _ in range(count):
                index, cursor = read_leb(payload, cursor)
                names[index], cursor = read_name(payload, cursor)
        offset = end
    return names


PROJECT_CRATES = ["ui_leptos", "web_platform", "features", "persistence", "domain"]
V0_CRATE_ROOT = re.compile(r"C(?:s[0-9A-Za-z]*_)?(\d+)(?=[A-Za-z_])")


def v0_crates(name):
    crates = []
    position = 0
    while match := V0_CRATE_ROOT.search(name, position):
        length = int(match.group(1))
        crates.append(name[match.end():match.end() + length])
        position = match.end() + length
    return crates


def crates_in(name):
    if name.startswith("_R"):
        return v0_crates(name)
    with_hashes = re.findall(r"(\w+)\[[0-9a-f]{16}\]", name)
    return with_hashes or re.findall(r"(?:^|[<\s&*(])_?(\w+)::", name)


# Generic code instantiated for one of our crates is attributed to the most
# dependent project crate it mentions; otherwise to the first crate it mentions.
def crate_of(name):
    mentioned = crates_in(name)
    for crate in PROJECT_CRATES:
        if crate in mentioned:
            return crate if mentioned[0] == crate else f"{crate} ({mentioned[0]})"
    return mentioned[0] if mentioned else "?"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("path", nargs="?", default=DEFAULT_PATH)
    parser.add_argument("--top", type=int, default=25)
    parser.add_argument("--crate")
    args = parser.parse_args()

    data = open(args.path, "rb").read()
    imports = 0
    bodies = []
    names = {}
    section_sizes = collections.Counter()
    for section_id, payload in sections(data):
        if section_id == 0:
            custom_name, _ = read_name(payload, 0)
            section_sizes["custom:" + custom_name] += len(payload)
            if custom_name == "name":
                names = function_names(payload[len(custom_name) + 1:])
            continue
        section_sizes[section_id] += len(payload)
        if section_id == 2:
            imports = imported_function_count(payload)
        elif section_id == 10:
            count, offset = read_leb(payload, 0)
            for _ in range(count):
                size, offset = read_leb(payload, offset)
                bodies.append(size)
                offset += size

    labels = {1: "type", 2: "import", 3: "function", 4: "table", 5: "memory", 6: "global",
              7: "export", 9: "elem", 10: "code", 11: "data", 12: "datacount"}
    print(f"File: {args.path} ({len(data):,} bytes)")
    for key, size in sorted(section_sizes.items(), key=lambda item: -item[1]):
        print(f"  section {labels.get(key, key):<24} {size:>10,}")
    if not names:
        print("No `name` section: use the raw build from target/.", file=sys.stderr)
        return

    functions = [(names.get(imports + index, f"func[{imports + index}]"), size)
                 for index, size in enumerate(bodies)]
    by_crate = collections.Counter()
    count_by_crate = collections.Counter()
    for name, size in functions:
        by_crate[crate_of(name)] += size
        count_by_crate[crate_of(name)] += 1
    total = sum(bodies)
    print(f"\nCode by crate (total {total:,} bytes, {len(bodies)} functions)")
    for crate, size in by_crate.most_common(args.top):
        print(f"  {crate:<28} {size:>10,}  {100 * size / total:5.1f} %  ({count_by_crate[crate]} fn)")

    selected = [f for f in functions if args.crate is None or crate_of(f[0]) == args.crate]
    by_root = collections.Counter()
    for name, size in functions:
        mentioned = crates_in(name)
        by_root[mentioned[0] if mentioned else "?"] += size
    print("\nCode by first mentioned crate")
    for crate, size in by_root.most_common(args.top):
        print(f"  {crate:<28} {size:>10,}  {100 * size / total:5.1f} %")

    print(f"\nLargest functions{' of ' + args.crate if args.crate else ''}")
    for name, size in sorted(selected, key=lambda f: -f[1])[:args.top]:
        print(f"  {size:>8,}  {name[:110]}")


if __name__ == "__main__":
    main()
