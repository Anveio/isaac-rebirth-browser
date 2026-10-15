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
"""
import argparse
import json
import pathlib
import subprocess
import sys
import tempfile


def reached(report):
    return set(report["coverage"]["reached"])


def candidates(harvest, tapes, wanted, extend):
    """(label, tape, prefix length, keys) for every prefix worth cutting. A
    prefix runs `extend` commands past a key's first step, so an NPC's AI,
    contacts and death actually run rather than only its first frame."""
    for label, first in harvest["coverage"]["firstSteps"].items():
        tape = tapes / f"{label}-case.json"
        if not tape.exists():
            continue
        length = len(json.loads(tape.read_text())["commands"])
        steps = sorted({step for key, step in first.items() if key in wanted})
        for step in steps:
            keys = {key for key, at in first.items() if key in wanted and at <= step}
            yield label, tape, min(step + 1 + extend, length), keys


def choose(options, wanted, budget):
    chosen, missing, total = [], set(wanted), 0
    while missing:
        best = max(
            (option for option in options if option[3] & missing),
            key=lambda option: (len(option[3] & missing) / option[2], -option[2]),
            default=None,
        )
        if best is None or total + best[2] > budget:
            break
        label, tape, length, keys = best
        chosen.append((label, tape, length, keys & missing))
        missing -= keys
        total += length
    return chosen, missing


def replay_reaches(binary, tape, keys):
    out = tape.with_suffix(".verify.json")
    subprocess.run(
        [binary, "--replay", str(tape), "--report", str(out), "--allow-unmet"],
        check=False,
        capture_output=True,
    )
    try:
        got = reached(json.loads(out.read_text()))
    finally:
        out.unlink(missing_ok=True)
    return keys <= got


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

    harvest = json.loads(args.harvest.read_text())
    baseline = json.loads(args.baseline.read_text())
    wanted = reached(harvest) - reached(baseline)
    if args.prefix:
        wanted = {key for key in wanted if key.startswith(tuple(args.prefix))}
    options = list(candidates(harvest, args.tapes, wanted, args.extend))
    chosen, missing = choose(options, wanted, args.budget)
    for label, tape, length, keys in chosen:
        print(f"{label}: {length} commands -> {sorted(keys)}")
    if missing:
        print(f"not covered within the budget: {sorted(missing)}")
    if not args.out:
        return 0
    failed = False
    for label, tape, length, keys in chosen:
        source = json.loads(tape.read_text())
        target = args.out / f"coverage-{label.removeprefix('game-')}.json"
        cut = {**source, "commands": source["commands"][:length]}
        encoded = json.dumps(cut, separators=(",", ":"))
        if args.verify:
            with tempfile.TemporaryDirectory(prefix=".cover-", dir=args.out) as scratch:
                candidate = pathlib.Path(scratch) / target.name
                candidate.write_text(encoded)
                verified = replay_reaches(args.binary, candidate, keys)
            if not verified:
                print(f"{target}: replay misses {sorted(keys)}; candidate discarded", file=sys.stderr)
                failed = True
                continue
        target.write_text(encoded)
        print(f"wrote {target}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
