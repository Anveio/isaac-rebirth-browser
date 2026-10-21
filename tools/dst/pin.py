#!/usr/bin/env python3
"""Pin a room a `rooms` finding was met in, so every corpus run replays it.

Game tapes reach only the floors a game case descends to, so a finding a
`rooms` case met on a later floor has no tape. A pin in
`engine/dst/pinned-rooms.json` names the room instead: its run code, stage,
room index and template, the seed of its boot, descent and play, and the
census keys the room reaches. `rebirth-dst --pins` descends that run with
that seed and plays only that room (`docs/dst.md`).

`search` tries `--tries` seeds for one room in a single run. For a finding
not yet fixed, `--signature TEXT` keeps the first seed whose case fails a
property with exactly that signature (`npc 24` is not `npc 240`), and
`--fails TEXT` the first with TEXT anywhere in a failure's signature or
details. Without either it keeps the first whose room reaches every
`--reaches` key with nothing failed. The kept seed is replayed alone to
confirm it, then printed, or written with `--write`.

  python3 tools/dst/pin.py --binary B search --name regression-depths-champion-11 \\
      --code 7WEVL3X9 --stage 5 --room 19 --template 1.82 \\
      --reaches 'champion:11 on stage 5' --fails 'champion color 11'

`check` replays each pin alone and prints its verdict and drift, failing if
any pin fails a property or drifted. A pin drifts when its run no longer
reaches its stage, its room is another template or is not entered, or the
room misses a key; a pin that fails a property is judged by its failures,
so what it did not reach is not drift. `refresh --only NAME` searches a new seed for a drifted
pin with its recorded fields and writes it; a room generation moved
elsewhere needs a new search from the finding's `dst.at`.
"""
import argparse
import json
import os
import pathlib
import subprocess
import sys
import tempfile

PINS = pathlib.Path("engine/dst/pinned-rooms.json")


def run_pins(binary, pins, work):
    """The report of one run playing exactly `pins`."""
    path = work / "pins.json"
    report = work / "report.json"
    expected = [pin["name"] for pin in pins]
    if len(set(expected)) != len(expected):
        sys.exit("pinned room names must be unique")
    path.write_text(json.dumps(pins, indent=2) + "\n")
    # Every call must produce its own report, including isolated replays.
    report.unlink(missing_ok=True)
    result = subprocess.run(
        [binary, "--units", "rooms", "--cases", "0", "--pins", str(path), "--report", str(report), "--allow-unmet"],
        check=False,
        capture_output=True,
        text=True,
    )
    # Exit 1 is a property failure, which --fails searches deliberately keep.
    # Startup, input, report-write and process failures are not pin verdicts.
    if result.returncode not in (0, 1):
        sys.exit(f"rebirth-dst exited {result.returncode}:\n{result.stderr}")
    if not report.exists():
        sys.exit(f"rebirth-dst wrote no report:\n{result.stderr}")
    data = json.loads(report.read_text())
    census = (data.get("coverage") or {}).get("pins", {})
    cases = [case.get("pin") for case in data.get("cases", [])]
    if set(census) != set(expected) or len(cases) != len(expected) or set(cases) != set(expected):
        sys.exit("rebirth-dst did not report every requested pinned room")
    return data


def failures(report):
    """(property, signature, pin, details) of each failed property's
    signatures; a signature keeps the first case that failed it."""
    found = []
    for prop in report["properties"]:
        if prop["status"] != "failed":
            continue
        signed = prop.get("signatures") or [{"signature": "", "firstFailure": prop.get("firstFailure")}]
        for signature in signed:
            details = signature.get("firstFailure") or {}
            pin = (details.get("dst") or {}).get("pin")
            found.append((prop["message"], signature.get("signature", ""), pin, details))
    return found


def verdict(binary, pin, work):
    """The pin's drift, or None, and the (property, signature, details) it
    failed."""
    report = run_pins(binary, [pin], work)
    drift = report["coverage"]["pins"][pin["name"]]
    return drift, [(message, signature, details) for message, signature, _, details in failures(report)]


def failing(options):
    """Whether a failure's (signature, details) is the finding `search`
    looks for, or None when it looks for a passing seed."""
    if options.signature is not None:
        return lambda signature, details: signature == options.signature
    if options.fails is not None:
        return lambda signature, details: options.fails in signature or options.fails in json.dumps(details)
    return None


def ordered(fields, seed):
    """A pin's fields in the order the pins file keeps them."""
    keys = ["name", "code", "stage", "room", "template", "seed", "reaches"]
    return {key: seed if key == "seed" else fields[key] for key in keys}


def search(binary, fields, tries, finding, work):
    """A pin for `fields` from the first of `tries` seeds that fails with
    `finding`, or passes and reaches its keys; None if no seed does."""
    candidates = []
    for index in range(tries):
        seed = int.from_bytes(os.urandom(8), "little")
        candidates.append(ordered({**fields, "name": f"{fields['name']}-{index}"}, seed))
    report = run_pins(binary, candidates, work)
    by_name = {pin["name"]: pin for pin in candidates}
    if finding:
        chosen = sorted(
            (by_name[pin] for _, signature, pin, details in failures(report) if pin in by_name and finding(signature, details)),
            key=lambda pin: candidates.index(pin),
        )
    else:
        failed = {pin for _, _, pin, _ in failures(report)}
        drifts = report["coverage"]["pins"]
        chosen = [pin for pin in candidates if drifts[pin["name"]] is None and pin["name"] not in failed]
    for candidate in chosen:
        pin = {**candidate, "name": fields["name"]}
        drift, failed = verdict(binary, pin, work)
        if finding and any(finding(signature, details) for _, signature, details in failed):
            return pin
        if not finding and drift is None and not failed:
            return pin
    return None


def write(pins_path, pin):
    pins = json.loads(pins_path.read_text()) if pins_path.exists() else []
    pins = [existing for existing in pins if existing["name"] != pin["name"]] + [pin]
    pins.sort(key=lambda existing: existing["name"])
    pins_path.write_text(json.dumps(pins, indent=2) + "\n")
    print(f"wrote {pin['name']} to {pins_path}")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--binary", default="engine/dst/target/release/rebirth-dst")
    parser.add_argument("--pins", type=pathlib.Path, default=PINS)
    commands = parser.add_subparsers(dest="command", required=True)
    find = commands.add_parser("search")
    find.add_argument("--name", required=True)
    find.add_argument("--code", required=True)
    find.add_argument("--stage", type=int, required=True)
    find.add_argument("--room", type=int, required=True)
    find.add_argument("--template", required=True, help="TYPE.VARIANT, as dst.at names it")
    find.add_argument("--reaches", action="append", default=[])
    finding = find.add_mutually_exclusive_group()
    finding.add_argument("--fails")
    finding.add_argument("--signature")
    find.add_argument("--tries", type=int, default=32)
    find.add_argument("--write", action="store_true")
    check = commands.add_parser("check")
    check.add_argument("--only", action="append")
    refresh = commands.add_parser("refresh")
    refresh.add_argument("--only", action="append", required=True)
    refresh.add_argument("--tries", type=int, default=32)
    options = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="rebirth-pin-") as directory:
        work = pathlib.Path(directory)
        if options.command == "search":
            room_type, variant = (int(part) for part in options.template.split("."))
            fields = {
                "name": options.name,
                "code": options.code,
                "stage": options.stage,
                "room": options.room,
                "template": [room_type, variant],
                "reaches": options.reaches,
            }
            pin = search(options.binary, fields, options.tries, failing(options), work)
            if pin is None:
                sys.exit(f"none of {options.tries} seeds pinned {options.name}")
            print(json.dumps(pin, indent=2))
            if options.write:
                write(options.pins, pin)
            return
        pins = json.loads(options.pins.read_text())
        selected = [pin for pin in pins if not options.only or pin["name"] in options.only]
        if options.only and len(selected) != len(set(options.only)):
            sys.exit(f"no pin named {sorted(set(options.only) - {pin['name'] for pin in selected})}")
        if options.command == "refresh":
            for old in selected:
                fields = {key: value for key, value in old.items() if key != "seed"}
                pin = search(options.binary, fields, options.tries, None, work)
                if pin is None:
                    sys.exit(f"none of {options.tries} seeds pinned {old['name']}; search it again from its finding")
                write(options.pins, pin)
            return
        bad = 0
        for pin in selected:
            drift, failed = verdict(options.binary, pin, work)
            bad += bool(drift or failed)
            state = "drifted: " + drift if drift else "in place"
            print(f"{pin['name']}: {state}; {len(failed)} failed")
            for message, signature, _ in failed:
                print(f"  {message}" + (f" [{signature}]" if signature else ""))
        sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
