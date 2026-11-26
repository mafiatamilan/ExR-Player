use crate::schema::{ControlMapping, KeymapProfile};
use exr_input::{InputTrigger, Modifiers};
use std::collections::HashMap;

/// Event-driven binding index built from a `KeymapProfile`.
/// Separates lookup from UI so hot-path input resolution requires zero profile scans.
#[derive(Debug, Clone, Default)]
pub struct MappingResolver {
    by_trigger: HashMap<InputTrigger, Vec<usize>>,
    /// Secondary lookup by base key (ignoring modifiers) for unmodified fallback.
    by_unmodified_trigger: HashMap<InputTrigger, Vec<usize>>,
}

impl MappingResolver {
    pub fn from_profile(profile: &KeymapProfile) -> Self {
        let mut by_trigger: HashMap<InputTrigger, Vec<usize>> = HashMap::new();
        let mut by_unmodified_trigger: HashMap<InputTrigger, Vec<usize>> = HashMap::new();

        for (idx, ctrl) in profile.controls.iter().enumerate() {
            if !ctrl.enabled {
                continue;
            }
            for trig in ctrl.kind.bound_triggers() {
                by_trigger.entry(trig).or_default().push(idx);
                if let InputTrigger::Key(combo) = trig {
                    if combo.modifiers.is_empty() {
                        by_unmodified_trigger.entry(trig).or_default().push(idx);
                    }
                }
            }
        }

        Self {
            by_trigger,
            by_unmodified_trigger,
        }
    }

    /// Resolves control indices matching a trigger.
    /// Exact modifier combos take precedence; if none match and modifiers are held,
    /// falls back to unmodified bindings so movement keys (WASD) work while holding Shift/Ctrl.
    pub fn resolve<'a>(
        &self,
        profile: &'a KeymapProfile,
        trigger: InputTrigger,
        held_mods: Modifiers,
    ) -> Vec<&'a ControlMapping> {
        let effective_trigger = match trigger {
            InputTrigger::Key(mut combo) => {
                combo.modifiers = held_mods;
                InputTrigger::Key(combo)
            }
            other => other,
        };

        if let Some(indices) = self.by_trigger.get(&effective_trigger) {
            return indices
                .iter()
                .filter_map(|&i| profile.controls.get(i))
                .collect();
        }

        if let InputTrigger::Key(combo) = trigger {
            let base = InputTrigger::key(combo.key);
            if let Some(indices) = self.by_unmodified_trigger.get(&base) {
                return indices
                    .iter()
                    .filter_map(|&i| profile.controls.get(i))
                    .collect();
            }
        }

        Vec::new()
    }
}
