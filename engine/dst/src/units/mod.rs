//! The registry of logical engine units under simulation. Each unit owns the
//! Antithesis properties declared in its module; `props::verdict` scopes the
//! must-hit catalog to the units a run selected.
use crate::harness::Unit;

mod abi;
mod actors;
mod contact;
mod floor;
mod game;
mod grid;
mod item_pools;
mod layout;
mod motion;
mod mt;
mod pathfinder;
mod phase;
mod room_spawns;
mod room_templates;
mod seed_codec;
mod tears;

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
        name: "layout",
        summary: "rectangular floor growth over every accepted seed, room count and shape mask",
        run: layout::run,
    },
    Unit {
        name: "room_spawns",
        summary: "authored spawn-group selection over every authored room and synthetic edge rooms",
        run: room_spawns::run,
    },
    Unit {
        name: "room_templates",
        summary: "authored room-template picks and weight decay against a weight model",
        run: room_templates::run,
    },
    Unit {
        name: "floor",
        summary: "Basement generation structure, retries and per-descriptor room admission",
        run: floor::run,
    },
    Unit {
        name: "motion",
        summary: "player motion steps and held-input runs against an along/across velocity model",
        run: motion::run,
    },
    Unit {
        name: "tears",
        summary: "tear velocity, eye origin, size and flight against f64 and lifecycle models",
        run: tears::run,
    },
    Unit {
        name: "grid",
        summary: "grid queries and ground-collision solvers on generated rooms",
        run: grid::run,
    },
    Unit {
        name: "contact",
        summary: "mass contact responses against momentum, centre-of-mass and overlap oracles",
        run: contact::run,
    },
    Unit {
        name: "pathfinder",
        summary: "FindGridPath's flood against an independent node-weighted Dijkstra",
        run: pathfinder::run,
    },
    Unit {
        name: "item_pools",
        summary: "ItemPool selection, depletion, fallbacks, routing and continuation",
        run: item_pools::run,
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
