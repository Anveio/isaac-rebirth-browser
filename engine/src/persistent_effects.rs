//! Inline Persistent callbacks over the one active Profile; no shadow stats.
use crate::mt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PlatformBackend {
    ///The port has no Steam wrapper. This is an actual target backend fact,
    ///not a claim about the independent Original wrapper's readiness/gates.
    Unavailable,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlatformAttempt {
    pub achievement: u32,
    pub backend: PlatformBackend,
    pub succeeded: bool,
}

struct Effects<'a> {
    notification: &'a mut Option<crate::unlock_notifications::Runtime>,
    popups: Option<bool>,
    loading: bool,
    pre_counter: Option<u32>,
    backend: Option<PlatformBackend>,
    attempts: &'a mut Vec<PlatformAttempt>,
    audio: &'a mut crate::audio::AudioState,
    shared: &'a mut mt::Mt19937,
}

impl crate::persistent::AwardCallbacks for Effects<'_> {
    fn validate_awards(&self, ids: &[u32]) -> Result<(), String> {
        self.backend
            .ok_or("Persistent platform backend ownership is unknown")?;
        self.pre_counter
            .ok_or("Persistent callback has no actual System pre-counter")?;
        if self.popups == Some(false) {
            return Ok(());
        }
        self.notification
            .as_ref()
            .ok_or("Persistent unlock notification owner is missing")?
            .validate_awards(ids, self.popups)
    }

    fn report_platform(&mut self, id: u32) -> Result<(), String> {
        //Keep the unavailable target backend explicit; never manufacture
        //SetAchievement/StoreStats success or infer Original Steam readiness.
        self.attempts.push(PlatformAttempt {
            achievement: id,
            backend: self
                .backend
                .ok_or("Persistent platform backend ownership is unknown")?,
            succeeded: false,
        });
        Ok(())
    }

    fn enqueue_and_update_notification(&mut self, id: u32) -> Result<(), String> {
        match self.popups {
            Some(false) => Ok(()),
            Some(true) => self
                .notification
                .as_mut()
                .ok_or("Persistent unlock notification owner is missing")?
                .enqueue_and_update(
                    id,
                    self.pre_counter
                        .ok_or("Persistent callback System phase is unknown")?,
                    self.loading,
                    self.audio,
                    self.shared,
                ),
            None => Err("Persistent Popups setting ownership is unknown".into()),
        }
    }
}

fn apply(state: &mut crate::State, mutation: crate::persistent::Mutation) -> Result<(), String> {
    let slot = usize::from(
        state
            .profile
            .checked_sub(1)
            .ok_or("Persistent profile slot is invalid")?,
    );
    let profile = state
        .profiles
        .get_mut(slot)
        .ok_or("Persistent profile slot is absent")?;
    let runtime = state
        .persistent_runtime
        .as_mut()
        .ok_or("Persistent live dirty/inhibit history is unknown")?;
    let mut effects = Effects {
        notification: &mut state.unlock_notifications,
        popups: state.unlock_popups,
        loading: state.unlock_notifications_loading,
        pre_counter: state.persistent_callback_pre_counter,
        backend: state.platform_achievement_backend,
        attempts: &mut state.platform_achievement_attempts,
        audio: &mut state.audio,
        shared: &mut state.boot_lifecycle.shared,
    };
    mutation.apply(
        &mut profile.achievements,
        &mut profile.counters,
        &mut profile.collected_history,
        runtime,
        &mut effects,
    )
}

pub(crate) fn reset_counter21(
    state: &mut crate::State,
    catalog: &crate::challenges::Catalog,
) -> Result<(), String> {
    let profile = state
        .profiles
        .get(usize::from(state.profile - 1))
        .ok_or("Persistent profile slot is absent")?;
    let mutation = crate::persistent::prepare_restart_counter21(
        profile,
        &profile.collected_history,
        state
            .persistent_runtime
            .ok_or("Persistent live dirty/inhibit history is unknown")?,
        catalog,
    )?;
    apply(state, mutation)
}

///52CDBC precedes the permanent AddCollectible count. The immutable catalogue
///cache is rehydrated from Game's actual resource owner, never guessed data.
pub(crate) fn mark_collected(state: &mut crate::State, id: u32) -> Result<(), String> {
    let catalog = state
        .persistent_catalog
        .as_ref()
        .cloned()
        .ok_or("Persistent Challenge catalogue owner is unknown")?;
    let profile = state
        .profiles
        .get(usize::from(state.profile - 1))
        .ok_or("Persistent profile slot is absent")?;
    let mutation = crate::persistent::prepare_collected(
        profile,
        &profile.collected_history,
        state
            .persistent_runtime
            .ok_or("Persistent live dirty/inhibit history is unknown")?,
        &catalog,
        usize::try_from(id).map_err(|_| "collected id exceeds target usize")?,
    )?;
    apply(state, mutation)
}

pub(crate) fn add_counter(
    state: &mut crate::State,
    index: usize,
    delta: i32,
) -> Result<(), String> {
    let catalog = state
        .persistent_catalog
        .as_ref()
        .cloned()
        .ok_or("Persistent Challenge catalogue owner is unknown")?;
    let profile = state
        .profiles
        .get(usize::from(state.profile - 1))
        .ok_or("Persistent profile slot is absent")?;
    let mutation = crate::persistent::prepare_counter(
        profile,
        &profile.collected_history,
        state
            .persistent_runtime
            .ok_or("Persistent live dirty/inhibit history is unknown")?,
        &catalog,
        index,
        delta,
    )?;
    apply(state, mutation)
}
