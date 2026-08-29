"""ADR-0041 harness re-pin: map every released 1-based effect property index
to its grouped-topology position, and rewrite the hard-coded `.property(N)`
sites in the harness scripts.

Both orders are reconstructed here from the same constants the plugin uses
(V1 pools, growth pools, gradient stops, banks). `verify` cross-checks the
generated map against the spot table the slice-3 report recorded; `dry` lists
every rewrite it would make; `apply` performs them. Run `apply` only after
the live phase-A check has confirmed the map on the installed batch build.

The editor modes map the grouped order to the editor-flavor order, where a
canvas is the first row in each gradient group. An applied file records that
index-flavor contract because adjacent grouped and editor indexes overlap.
"""
import json, os, re, sys

V1 = [("Float", 48), ("Int", 8), ("Bool", 16), ("Color", 12), ("Point", 12), ("Angle", 8)]
GROWTH = [("Layer", 4), ("Gradient", 2), ("Point3D", 8), ("Path", 2)]
BANKS = 12
BANK = [("Float", 8), ("Int", 2), ("Bool", 2), ("Color", 3), ("Point", 2), ("Angle", 1)]
HEADS = ["Language", "Source", "Compile", "Status", "StateToken"]
STOPS = 8
GRADS = 2


def old_order():
    order = list(HEADS)
    for kind, n in V1:
        order += [f"{kind}:{i}" for i in range(n)]
    order.append("Details")
    for kind, n in GROWTH:
        order += [f"{kind}:{i}" for i in range(n)]
    for g in range(GRADS):
        order.append(f"GCount:{g}")
        for s in range(STOPS):
            order += [f"GStop:{g}:{s}:{f}" for f in ("Pos", "Color", "Alpha")]
    order.append("PlanToken")
    return order


def new_order():
    order = ["SetupStart"] + list(HEADS) + ["Details", "PlanToken", "SetupEnd", "MainStart"]
    for kind, n in V1:
        order += [f"{kind}:{i}" for i in range(n)]
    for kind, n in GROWTH:
        if kind == "Gradient":
            continue
        order += [f"{kind}:{i}" for i in range(n)]
    for g in range(GRADS):
        order.append(f"GGroupStart:{g}")
        order.append(f"Gradient:{g}")
        order.append(f"GCount:{g}")
        for s in range(STOPS):
            order += [f"GStop:{g}:{s}:{f}" for f in ("Pos", "Color", "Alpha")]
        order.append(f"GGroupEnd:{g}")
    order.append("MainEnd")
    for p in range(BANKS):
        order.append(f"PGroupStart:{p}")
        for kind, n in BANK:
            order += [f"Bank:{p}:{kind}:{i}" for i in range(n)]
        order.append(f"PGroupEnd:{p}")
    return order


def editor_order():
    order = []
    for key in new_order():
        order.append(key)
        if key.startswith("GGroupStart:"):
            order.append(f"GCanvas:{key.rsplit(':', 1)[1]}")
    return order


def index_map():
    old = {key: i + 1 for i, key in enumerate(old_order())}
    new = {key: i + 1 for i, key in enumerate(new_order())}
    return {old[k]: new[k] for k in old}


def editor_index_map():
    new = {key: i + 1 for i, key in enumerate(new_order())}
    editor = {key: i + 1 for i, key in enumerate(editor_order())}
    return {new[k]: editor[k] for k in new}


# The ADR-0041 report's spot table (old -> new); `verify` fails loudly on drift.
REPORT_TABLE = {
    1: 2, 2: 3, 3: 4, 4: 5, 5: 6, 110: 7, 177: 8,
    6: 11, 7: 12, 8: 13, 53: 58, 54: 59, 62: 67, 78: 83, 90: 95, 102: 107,
    111: 115, 115: 130, 127: 131, 128: 132, 117: 119, 125: 127,
}

# Pins both unchanged growth pools and the two insertion boundaries.
EDITOR_REPORT_TABLE = {
    115: 115, 119: 119, 127: 127,
    129: 129, 130: 131, 131: 132, 156: 157,
    157: 158, 158: 160, 159: 161, 184: 186,
    185: 187, 187: 189,
}
EDITOR_FLAVOR_NOTE = "Numeric DynamicFx property indexes target the editor declaration order."

# Harness files that address effect params by numeric property index. The
# spike driver is deliberately absent: its dumps are recorded evidence of the
# 0.0.5 layout and must stay byte-true.
TARGETS = [
    "scripts/m2", "scripts/m3", "scripts/m4", "scripts/m5", "scripts/m6",
    "scripts/m7", "scripts/f003", "scripts/bind", "scripts/p05_probe.jsx",
    "scripts/p05_aerender_test.jsx", "scripts/p05_defaults_test.jsx",
    "scripts/p05_fresh_render.jsx", "scripts/p05_poke_render.jsx",
    "scripts/p05_reexpr_render.jsx", "scripts/regression_programmatic.jsx",
]
# Only rewrite indexes reached through an effect-typed receiver; masks,
# layers and comps also have .property(N) and must not shift.
SITE = re.compile(r"(?P<recv>\b(?:fx|eff|effect|inst|instA|instB)\w*\.property\()(?P<idx>\d{1,3})(?P<close>\))")


def repo_root():
    return os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))


def files():
    root = repo_root()
    for target in TARGETS:
        path = os.path.join(root, target)
        if os.path.isfile(path):
            yield path
        elif os.path.isdir(path):
            for name in sorted(os.listdir(path)):
                if name.endswith((".jsx", ".py")):
                    yield os.path.join(path, name)


def sweep(write, mapping=None, flavor_note=None):
    if mapping is None:
        mapping = index_map()
    total = 0
    for path in files():
        with open(path, encoding="utf-8") as f:
            text = f.read()
        hits = []
        already_pinned = flavor_note is not None and flavor_note in text

        def sub(m):
            old = int(m.group("idx"))
            new = old if already_pinned else mapping.get(old, old)
            hits.append((old, new))
            return f"{m.group('recv')}{new}{m.group('close')}"

        out = SITE.sub(sub, text)
        changed = [(o, n) for o, n in hits if o != n]
        if write and changed and flavor_note is not None:
            prefix = "#" if path.endswith(".py") else "//"
            out = f"{prefix} {flavor_note}\n" + out
        if hits:
            rel = os.path.relpath(path, repo_root())
            print(f"{rel}: {len(hits)} site(s), {len(changed)} shifted "
                  + json.dumps(changed[:12]))
            total += len(changed)
        if write and out != text:
            with open(path, "w", encoding="utf-8", newline="") as f:
                f.write(out)
    print(f"[{'APPLIED' if write else 'DRY'}] {total} shifted site(s)")


def verify():
    mapping = index_map()
    bad = {o: (mapping.get(o), n) for o, n in REPORT_TABLE.items() if mapping.get(o) != n}
    assert not bad, f"map disagrees with the ADR-0041 report: {bad}"
    assert len(old_order()) == 177 and mapping[1] == 2
    assert new_order().index("SetupStart") + 1 == 1, "Setup topic start"
    assert new_order().index("PGroupStart:0") + 2 == 187, "bank 1 first slot"
    print(f"map OK: {len(mapping)} params, old 177 -> new {len(new_order())} declared rows")


def verify_editor():
    mapping = editor_index_map()
    bad = {
        current: (mapping.get(current), editor)
        for current, editor in EDITOR_REPORT_TABLE.items()
        if mapping.get(current) != editor
    }
    assert not bad, f"map disagrees with the editor report: {bad}"

    first_shift = new_order().index("Gradient:0") + 1
    second_shift = new_order().index("Gradient:1") + 1
    assert first_shift == 130 and second_shift == 158
    assert all(mapping[i] == i for i in range(1, first_shift))
    assert all(mapping[i] == i + 1 for i in range(first_shift, second_shift))
    assert all(mapping[i] == i + 2 for i in range(second_shift, len(new_order()) + 1))
    assert GRADS == 2
    assert len(editor_order()) == len(new_order()) + 2
    print(
        f"editor map OK: {len(mapping)} params, grouped {len(new_order())} "
        f"-> editor {len(editor_order())} declared rows"
    )


def main():
    cmd = sys.argv[1] if len(sys.argv) > 1 else "verify"
    if cmd == "verify":
        verify()
    elif cmd == "dry":
        verify()
        sweep(write=False)
    elif cmd == "apply":
        verify()
        sweep(write=True)
    elif cmd == "verify-editor":
        verify_editor()
    elif cmd == "dry-editor":
        verify_editor()
        sweep(write=False, mapping=editor_index_map(), flavor_note=EDITOR_FLAVOR_NOTE)
    elif cmd == "apply-editor":
        verify_editor()
        sweep(write=True, mapping=editor_index_map(), flavor_note=EDITOR_FLAVOR_NOTE)
    else:
        raise SystemExit(
            "usage: repin_indexes.py "
            "verify|dry|apply|verify-editor|dry-editor|apply-editor"
        )


if __name__ == "__main__":
    main()
