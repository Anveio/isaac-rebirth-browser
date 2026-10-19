#!/usr/bin/env python3
"""Cut the shortest DST tape prefixes that bring census keys into the corpus.

Every DST report records, per game case, the command at which the case first
reached each coverage key (`coverage.firstSteps`). Given a harvest run with
`--retain-all` and a baseline report of the corpus alone (`--units game
--cases 0 --corpus engine/dst/corpus`, what every run is sure to reach), this
picks harvested prefixes greedily, most new keys per command first, until the
keys the harvest reached but the baseline missed are covered, then writes them
as `coverage-<case seed>.json` tapes. With --verify it replays each written tape
and keeps it only if the replay reaches its keys.

  python3 tools/dst/cover.py --harvest H/report.json --tapes H/tapes \\
      --baseline B/report.json --prefix npc: --out engine/dst/corpus --verify

A corpus can be the harvest too: given a census of an earlier corpus and that
corpus's tapes, it carries the keys a rebuilt corpus lost from the old tapes'
prefixes, as `coverage-<old name>-carried.json`. `tools/dst/chase.py
--recipes` does this after every rebuild.
"""
import argparse
import json
import pathlib
import subprocess
import sys
import tempfile


def reached(report):
    return set(report["coverage"]["reached"])


CORPUS_LABEL = "game-replay-"


def harvested_tape(tapes, label):
    """A generated case's retained tape, or a replayed corpus tape by name."""
    if label.startswith(CORPUS_LABEL):
        return tapes / f"{label.removeprefix(CORPUS_LABEL)}.json"
    return tapes / f"{label}-case.json"


def written_name(label):
    if not label.startswith(CORPUS_LABEL):
        return f"coverage-{label.removeprefix('game-')}.json"
    name = label.removeprefix(CORPUS_LABEL).removesuffix("-carried")
    for kind in ("coverage-", "regression-"):
        name = name.removeprefix(kind)
    return f"coverage-{name}-carried.json"


def candidates(harvest, tapes, wanted, extend):
    """(label, tape, prefix length, keys) for every prefix worth cutting. A
    prefix runs `extend` commands past a key's first step, so an NPC's AI,
    contacts and death actually run rather than only its first frame."""
    for label, first in harvest["coverage"]["firstSteps"].items():
        tape = harvested_tape(tapes, label)
        if not tape.exists():
            continue
        length = len(json.loads(tape.read_text())["commands"])
        steps = sorted({step for key, step in first.items() if key in wanted})
        for step in steps:
            keys = {key for key, at in first.items() if key in wanted and at <= step}
            yield label, tape, min(step + 1 + extend, length), keys


def choose(options, wanted, budget):
    """Greedy prefixes, most new keys per added command first. A longer
    prefix of a case already chosen replaces it and costs only the commands
    it adds."""
    chosen, missing, total = {}, set(wanted), 0

    def cost(option):
        held = chosen.get(option[0])
        return option[2] - (held[2] if held else 0)

    while missing:
        best = max(
            (option for option in options if option[3] & missing and cost(option) > 0),
            key=lambda option: (len(option[3] & missing) / cost(option), -cost(option)),
            default=None,
        )
        if best is None or total + cost(best) > budget:
            break
        label, tape, length, keys = best
        total += cost(best)
        held = chosen.get(label, (label, tape, 0, set()))[3]
        chosen[label] = (label, tape, length, held | (keys & missing))
        missing -= keys
    return list(chosen.values()), missing


def replay_reaches(binary, tape, keys):
    """Whether the tape's replay reaches `keys`, and the properties it fails."""
    out = tape.with_suffix(".verify.json")
    subprocess.run(
        [binary, "--replay", str(tape), "--report", str(out), "--allow-unmet"],
        check=False,
        capture_output=True,
    )
    try:
        report = json.loads(out.read_text())
    finally:
        out.unlink(missing_ok=True)
    return keys <= reached(report), [p["message"] for p in report["properties"] if p["status"] == "failed"]


def cover(binary, harvest, tapes, baseline, out=None, prefixes=(), budget=6000, extend=300, verify=False):
    """Write the chosen prefixes to `out`; returns (written paths, keys left
    uncovered, whether a candidate failed its replay)."""
    wanted = reached(harvest) - reached(baseline)
    # A key every reaching case met before its first command comes from case
    # generation itself (the seed scout), which no tape carries.
    first_steps = harvest["coverage"]["firstSteps"].values()
    generated = {key for key in wanted if all(first[key] == 0 for first in first_steps if key in first)}
    if generated:
        print(f"generated cases alone reach {sorted(generated)}; no tape can carry them")
    wanted -= generated
    if prefixes:
        wanted = {key for key in wanted if key.startswith(tuple(prefixes))}
    options = list(candidates(harvest, tapes, wanted, extend))
    chosen, missing = choose(options, wanted, budget)
    for label, tape, length, keys in chosen:
        print(f"{label}: {length} commands -> {sorted(keys)}")
    if missing:
        print(f"not covered within the budget: {sorted(missing)}")
    written, failed = [], False
    if not out:
        return written, missing, failed
    for label, tape, length, keys in chosen:
        source = json.loads(tape.read_text())
        target = out / written_name(label)
        cut = {**source, "commands": source["commands"][:length]}
        encoded = json.dumps(cut, separators=(",", ":"))
        failures = []
        if verify:
            with tempfile.TemporaryDirectory(prefix=".cover-", dir=out) as scratch:
                candidate = pathlib.Path(scratch) / target.name
                candidate.write_text(encoded)
                verified, failures = replay_reaches(binary, candidate, keys)
            if not verified:
                print(f"{target}: replay misses {sorted(keys)}; candidate discarded", file=sys.stderr)
                failed = True
                continue
        target.write_text(encoded)
        written.append(target)
        print(f"wrote {target}")
        for message in failures:
            print(f"the tape fails: {message}")
    return written, missing, failed


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--harvest", required=True, type=pathlib.Path)
    parser.add_argument("--tapes", required=True, type=pathlib.Path)
    parser.add_argument("--baseline", required=True, type=pathlib.Path)
    parser.add_argument("--prefix", action="append", default=[], help="only keys starting with this (repeatable)")
    parser.add_argument("--budget", type=int, default=6000, help="most commands to add")
    parser.add_argument("--extend", type=int, default=300, help="commands kept past a key's first step")
    parser.add_argument("--out", type=pathlib.Path, help="write chosen prefixes here")
    parser.add_argument("--verify", action="store_true", help="replay written tapes and drop any that miss")
    parser.add_argument("--binary", default="engine/dst/target/release/rebirth-dst")
    args = parser.parse_args()
    _, _, failed = cover(
        args.binary,
        json.loads(args.harvest.read_text()),
        args.tapes,
        json.loads(args.baseline.read_text()),
        out=args.out,
        prefixes=args.prefix,
        budget=args.budget,
        extend=args.extend,
        verify=args.verify,
    )
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
