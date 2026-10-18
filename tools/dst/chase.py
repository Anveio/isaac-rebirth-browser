#!/usr/bin/env python3
"""Branch DST game cases stage by stage until one reaches a deep scenario,
then write the tape that does.

Some coverage needs a chain of rare events: a boss room cleared, then a
descent, then a new run. Each stage branches `--cases` generated cases from
the prefix so far (`rebirth-dst --resume`). It keeps the cases that reached
all of the stage's census keys, earliest first, and cuts each tape
`--extend` commands past that point. The next stage's first round branches
from the earliest. A round that reaches nothing moves the next round to the
next-earliest prefix, so one dead end, such as a boss entry no branch
survives, spends a round rather than the chase.

An engine change that moves the shared RNG moves every tape off its branch.
A scenario tape is then rebuilt by rerunning its chase, not by hand. Keys
are the census keys of a report's `coverage.firstSteps`; a met property is
`property:<message>`.

  python3 tools/dst/chase.py --stage cleared:5 --stage stage:2 \\
      --stage 'new run from the menus after a descent' \\
      --out engine/dst/corpus/regression-new-run-resets-stage.json

A stage lists several keys to reach them together. `--from TAPE [--at N]`
starts from a tape's first N commands, such as a boss room entry. The
written tape is replayed, and kept only if the replay reaches every stage's
keys.

`--recipes engine/dst/chases.json` reruns the recorded chase of every
scenario tape the corpus holds (`--only NAME` for one), so one command
rebuilds what a shared-MT change moved. A recipe's first stage is the
scenario's precondition, such as `room:5` before a boss clear. Each chase
starts from its own tape, cut just after the replay first reaches that
stage, or from a fresh boot when drift moved the tape off it.
"""
import argparse
import json
import os
import pathlib
import subprocess
import sys
import time


def run(binary, arguments):
    return subprocess.run([binary, *arguments], check=False, capture_output=True, text=True)


def replay(binary, tape, report):
    """The replayed tape's report: reached keys and failed properties."""
    run(binary, ["--replay", str(tape), "--report", str(report), "--allow-unmet"])
    result = json.loads(report.read_text())
    failed = [p["message"] for p in result["properties"] if p["status"] == "failed"]
    return set(result["coverage"]["reached"]), failed


def precondition_cut(binary, tape, keys, report):
    """Commands of `tape` up to the one where its replay first reaches all of
    `keys`, or None when it never does."""
    run(binary, ["--replay", str(tape), "--report", str(report), "--allow-unmet"])
    (first,) = json.loads(report.read_text())["coverage"]["firstSteps"].values()
    if not set(keys) <= first.keys():
        return None
    return max(first[key] for key in keys) + 1


def write_tape(path, tape, length):
    path.write_text(json.dumps({**tape, "commands": tape["commands"][:length]}, separators=(",", ":")))


def chase(args, work):
    """The tape of the earliest case that reached every stage, or None."""
    prefixes = []
    if args.source:
        source = json.loads(args.source.read_text())
        prefixes = [work / "from.json"]
        write_tape(prefixes[0], source, args.at or len(source["commands"]))
    for index, keys in enumerate(args.stage):
        keys = set(keys)
        last = index == len(args.stage) - 1
        if prefixes:
            usable, done = [], []
            for rank, prefix in enumerate(prefixes):
                before, _ = replay(args.binary, prefix, work / f"stage-{index}-prefix-{rank}.json")
                if keys <= before:
                    done.append(prefix)
                elif keys & before:
                    print(f"stage {index}: prefix {rank} already reaches {sorted(keys & before)}; "
                          "their first steps cannot show a later hit", file=sys.stderr)
                else:
                    usable.append(prefix)
            if done:
                print(f"stage {index}: {len(done)} of {len(prefixes)} prefixes already reach {sorted(keys)}")
                prefixes = done
                continue
            if not usable:
                return None
            prefixes = usable
        found = []
        for attempt in range(args.rounds):
            seed = args.seed + 1000 * index + attempt
            out = work / f"stage-{index}-{attempt}"
            arguments = ["--units", "game", "--cases", str(args.cases), "--steps", str(args.steps),
                         "--seed", str(seed), "--tapes", str(out / "tapes"),
                         "--report", str(out / "report.json"), "--retain-all", "--allow-unmet"]
            origin = ""
            if prefixes:
                rank = attempt % len(prefixes)
                arguments += ["--resume", str(prefixes[rank])]
                origin = f", prefix {rank} of {len(prefixes)}"
            started = time.monotonic()
            run(args.binary, arguments)
            first_steps = json.loads((out / "report.json").read_text())["coverage"]["firstSteps"]
            hits = sorted(
                (max(first[key] for key in keys), label)
                for label, first in first_steps.items()
                if keys <= first.keys()
            )
            partial = {key: sum(key in first for first in first_steps.values()) for key in sorted(keys)}
            print(f"stage {index} round {attempt} (seed {seed}{origin}, {time.monotonic() - started:.0f}s): "
                  f"{len(hits)} of {len(first_steps)} cases reach {sorted(keys)}; per key {partial}")
            # A case that failed a property keeps only its violation tape: it
            # is a finding to look at, never a prefix to build on.
            violating = [label for _, label in hits if not (out / "tapes" / f"{label}-case.json").exists()]
            if violating:
                print(f"stage {index}: {len(violating)} reaching cases failed a property; "
                      f"see {out / 'tapes'} ({', '.join(violating)})", file=sys.stderr)
            hits = [hit for hit in hits if hit[1] not in violating]
            if hits:
                extend = args.final_extend if last else args.extend
                # The next stage spreads at most one round over each prefix.
                for rank, (step, label) in enumerate(hits[:args.rounds]):
                    tape = json.loads((out / "tapes" / f"{label}-case.json").read_text())
                    length = min(step + 1 + extend, len(tape["commands"]))
                    found.append(work / f"stage-{index}-{rank}.json")
                    write_tape(found[-1], tape, length)
                    if rank == 0:
                        print(f"stage {index}: {label} reaches it at command {step}; "
                              f"prefix is now {length} commands")
                break
        if not found:
            return None
        prefixes = found
    return prefixes[0]


def build(args, work):
    """Chase one scenario and write its verified tape to args.out."""
    work.mkdir(parents=True, exist_ok=True)
    print(f"chase seed {args.seed}, stage runs in {work}")
    tape = chase(args, work)
    if not tape:
        print("no case reached the stage; raise --cases, --steps or --rounds", file=sys.stderr)
        return 1
    wanted = set().union(*map(set, args.stage))
    reached, failed = replay(args.binary, tape, work / "verify.json")
    if not wanted <= reached:
        print(f"{tape}: replay misses {sorted(wanted - reached)}; not written", file=sys.stderr)
        return 1
    args.out.write_text(tape.read_text())
    commands = len(json.loads(tape.read_text())["commands"])
    print(f"wrote {args.out} ({commands} commands)")
    for message in failed:
        print(f"the tape fails: {message}")
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--stage", action="append", nargs="+", metavar="KEY",
                        help="census keys one stage must reach together (repeat per stage)")
    parser.add_argument("--from", dest="source", type=pathlib.Path, help="start from this tape")
    parser.add_argument("--at", type=int, help="with --from, keep only its first N commands")
    parser.add_argument("--cases", type=int, default=64, help="cases branched per round")
    parser.add_argument("--steps", type=int, default=3000, help="generated commands per branched case")
    parser.add_argument("--rounds", type=int, default=4, help="rounds a stage gets before giving up")
    parser.add_argument("--extend", type=int, default=30, help="commands kept past a stage before branching")
    parser.add_argument("--final-extend", type=int, default=300, help="commands kept past the last stage")
    parser.add_argument("--seed", type=int, default=int.from_bytes(os.urandom(6), "little"))
    parser.add_argument("--work", type=pathlib.Path, help="keep stage runs here (default .cache/dst/chase-*)")
    parser.add_argument("--out", type=pathlib.Path, help="where the verified tape goes")
    parser.add_argument("--recipes", type=pathlib.Path,
                        help="rebuild every corpus tape a recipe file names (see engine/dst/chases.json)")
    parser.add_argument("--only", action="append", default=[], help="with --recipes, only these tapes")
    parser.add_argument("--binary", default="engine/dst/target/release/rebirth-dst")
    args = parser.parse_args()
    if not args.recipes:
        if not args.stage or not args.out:
            parser.error("--stage and --out are required without --recipes")
        return build(args, args.work or pathlib.Path(f".cache/dst/chase-{args.seed}"))
    # A recipe names a corpus tape, the prefix it branches from (by default
    # its own first `at` commands) and its stages; other keys override flags.
    corpus = args.recipes.parent / "corpus"
    failed = 0
    for recipe in json.loads(args.recipes.read_text()):
        if args.only and recipe["out"] not in args.only:
            continue
        one = argparse.Namespace(**vars(args))
        one.out = corpus / recipe["out"]
        # `"from": null` chases from a fresh boot rather than a prefix.
        source = recipe.get("from", recipe["out"])
        one.source = corpus / source if source else None
        one.stage = recipe["stages"]
        for key in ("cases", "steps", "rounds", "extend", "final_extend"):
            setattr(one, key, recipe.get(key, getattr(args, key)))
        print(f"== {recipe['out']}")
        work = (args.work or pathlib.Path(f".cache/dst/chase-{args.seed}")) / recipe["out"].removesuffix(".json")
        work.mkdir(parents=True, exist_ok=True)
        if one.source:
            one.at = precondition_cut(args.binary, one.source, one.stage[0], work / "precondition.json")
            if one.at is None:
                print(f"{source} no longer reaches {one.stage[0]}; chasing from a fresh boot")
                one.source = None
            else:
                print(f"{source} reaches {one.stage[0]} at command {one.at - 1}")
        failed += build(one, work)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
