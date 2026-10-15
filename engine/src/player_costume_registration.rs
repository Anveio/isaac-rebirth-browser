//! Actual collectible costume-config registrations, not a pose/null cache.
//! A known empty constructor, real presence, and unknown restore are distinct.
use crate::{items, State};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CostumeRegistrationHistory {
    ///Ordered distinct config identities. One config may own multiple source
    ///entries; selected layer/pose consumers must prove their own readsets.
    pub configuration_ids: Vec<u32>,
}
impl CostumeRegistrationHistory {
    pub(crate) fn fresh() -> Self {
        //50EABA..50EAD2 constructs the empty AC0/AC8/AD0 vector.
        //Initial item grants are then recorded through their real Add hook.
        Self {
            configuration_ids: Vec::new(),
        }
    }
    fn register(&mut self, id: u32) {
        //5110B0/511400 erases every matching config entry, then5110D1
        //and51133A append its new entry/group at the tail. Never sort by id.
        self.configuration_ids.retain(|previous| *previous != id);
        self.configuration_ids.push(id);
    }
}

///Called only by the real52CC90 grant callback. ANM presence alone never
///admits a source type/skip path. Unknown histories never become empty.
pub(crate) fn on_collectible_added(state: &mut State, id: u32) {
    let normal = state.weapon_admission.source_player_type == Some(0);
    let Some(owner) = state.player_items.as_mut() else {
        return;
    };
    if !items::collectible_add_may_register_costume(id) {
        return;
    }
    //These actual passive4/152/169/229 registration paths are source-closed. Other
    //costume producers remain unknown until their event path is implemented.
    //This is an event producer set, never a loadout/controller permission.
    if !normal
        || !matches!(id, 4 | 152 | 169 | 229)
        || owner.trinket_slots != [0, 0]
        || !owner.temporary_collectibles.is_empty()
        || owner
            .owned_counts
            .get(id as usize)
            .is_none_or(|count| *count <= 0)
    {
        owner.costume_registrations = None;
        return;
    }
    if let Some(history) = &mut owner.costume_registrations {
        history.register(id);
    }
}
