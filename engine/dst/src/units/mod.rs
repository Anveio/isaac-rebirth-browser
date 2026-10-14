//! The registry of logical engine units under simulation. Each unit owns the
//! Antithesis properties declared in its module; `props::verdict` scopes the
//! must-hit catalog to the units a run selected.
use crate::harness::Unit;

mod abi;
mod actors;
mod floor;
mod game;
mod grid;
mod mt;
mod phase;
mod seed_codec;

pub const UNITS: &[Unit] = &[
    Unit {
        name: "seed_codec",
        summary: "public seed codes, canonical decoding and the run-seed xorshift handoff",
        run: seed_codec::run,
    },
    Unit {
        name: "mt",
        summary: "shared MT19937 stream, lazy descriptor seed tables and boot lifecycle",
        run: mt::run,
    },
    Unit {
        name: "phase",
        summary: "system update/interpolation phases against a counter model",
        run: phase::run,
    },
    Unit {
        name: "actors",
        summary: "actor admission order and room identities against a list model",
        run: actors::run,
    },
    Unit {
        name: "floor",
        summary: "Basement generation structure, retries and per-descriptor room admission",
        run: floor::run,
    },
    Unit {
        name: "grid",
        summary: "grid queries and ground-collision solvers on generated rooms",
        run: grid::run,
    },
    Unit {
        name: "game",
        summary: "the whole engine through public commands: invariants, contracts, replay, ABI",
        run: game::run,
    },
    Unit {
        name: "abi",
        summary: "the exported C ABI transport the browser bridge drives",
        run: abi::run,
    },
];
