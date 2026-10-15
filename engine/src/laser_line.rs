//! Source Line577120/578760/579fa0 state used by the real Laser7 callbacks.
use crate::motion::WorldVector;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CurvePoint {
    pub position: WorldVector,
    pub incoming: WorldVector,
    pub outgoing: WorldVector,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LaserLine {
    pub origin: WorldVector,
    pub offset: WorldVector,
    ///Actual source+14 and+68, not independently inferred visual settings.
    pub subdivisions: Option<u32>,
    pub mode: Option<u32>,
    pub width: Option<f32>,
    ///Actual+70..78 cubic control vector; full rebuild clears this vector.
    pub controls: Vec<CurvePoint>,
    ///Actual+50..58 retains the unsimplified resampling/smoothing history.
    pub history: Vec<WorldVector>,
    ///Actual+20..28, including source578420 point simplification.
    pub points: Vec<WorldVector>,
    ///Actual+38..40 captures points before source578420 changes them.
    pub prior_points: Vec<WorldVector>,
    ///576F50 leaves these scalar words untouched. They become known only
    ///through577120/the actual Laser first callback/579FA0, respectively.
    pub length: Option<f32>,
}

impl Default for LaserLine {
    fn default() -> Self {
        Self {
            origin: WorldVector { x: 0.0, y: 0.0 },
            offset: WorldVector { x: 0.0, y: 0.0 },
            subdivisions: None,
            mode: None,
            width: None,
            controls: Vec::new(),
            history: Vec::new(),
            points: Vec::new(),
            prior_points: Vec::new(),
            length: None,
        }
    }
}
