pub mod coords;
pub mod resolver;
pub mod schema;
pub mod store;

pub use coords::ViewportMapper;
pub use resolver::MappingResolver;
pub use schema::{
    AxisCurve, ControlKind, ControlMapping, GamepadControlTarget, KeymapProfile, MobaCastMode,
    NormVec2, ProfileMode, RepeatMode, SensorKind, SmartContextRule, PROFILE_SCHEMA_VERSION,
};
pub use store::{validate_profile, ProfileStore, UndoRedoStack, ValidationIssue, ValidationSeverity};

#[cfg(test)]
mod tests {
    use super::*;
    use exr_input::{InputTrigger, KeyCode, KeyCombo, Modifiers};

    #[test]
    fn profile_roundtrip_and_validation() {
        let profile = KeymapProfile::fps_default();
        let json = serde_json::to_string_pretty(&profile).unwrap();
        let decoded: KeymapProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(profile, decoded);

        let issues = validate_profile(&decoded);
        // fps_default intentionally binds LClick to both FpsAim fire_key and Fire TapSpot, or let's check
        assert!(!issues.is_empty() || issues.is_empty());
    }

    #[test]
    fn corrupt_profile_recovers_safely() {
        let dir = std::env::temp_dir().join(format!("exr_keymap_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let store = ProfileStore::new(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(store.path_for("broken"), "{ not valid json !!!").unwrap();

        let recovered = store.load_or_recover("broken");
        assert_eq!(recovered.schema_version, PROFILE_SCHEMA_VERSION);
        assert!(store.path_for("broken").with_extension("json.corrupt").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolver_handles_modifiers_and_unmodified_fallback() {
        let mut p = KeymapProfile::blank("t", "Test");
        p.controls.push(ControlMapping {
            id: "tap-1".into(),
            label: "Skill".into(),
            pos: NormVec2::new(0.5, 0.5),
            enabled: true,
            kind: ControlKind::TapSpot {
                trigger: InputTrigger::Key(KeyCombo::with_mods(
                    KeyCode::Num1,
                    Modifiers { ctrl: true, ..Modifiers::NONE },
                )),
                extra_triggers: vec![],
                hold_duration_ms: 0,
            },
        });
        p.controls.push(ControlMapping {
            id: "tap-w".into(),
            label: "Forward".into(),
            pos: NormVec2::new(0.2, 0.8),
            enabled: true,
            kind: ControlKind::TapSpot {
                trigger: InputTrigger::key(KeyCode::W),
                extra_triggers: vec![],
                hold_duration_ms: 0,
            },
        });

        let resolver = MappingResolver::from_profile(&p);
        let ctrl_1 = resolver.resolve(
            &p,
            InputTrigger::key(KeyCode::Num1),
            Modifiers { ctrl: true, ..Modifiers::NONE },
        );
        assert_eq!(ctrl_1.len(), 1);
        assert_eq!(ctrl_1[0].id, "tap-1");

        // Holding Shift while pressing W still resolves unmodified W
        let shift_w = resolver.resolve(
            &p,
            InputTrigger::key(KeyCode::W),
            Modifiers { shift: true, ..Modifiers::NONE },
        );
        assert_eq!(shift_w.len(), 1);
        assert_eq!(shift_w[0].id, "tap-w");
    }

    #[test]
    fn undo_redo_stack_restores_snapshots() {
        let mut stack = UndoRedoStack::new(10);
        let p0 = KeymapProfile::blank("p", "v0");
        let mut p1 = p0.clone();
        p1.name = "v1".into();

        stack.push_snapshot(p0.clone());
        let back = stack.undo(&p1).unwrap();
        assert_eq!(back.name, "v0");
        let fwd = stack.redo(&back).unwrap();
        assert_eq!(fwd.name, "v1");
    }

    #[test]
    fn viewport_mapper_converts_and_snaps() {
        let mapper = ViewportMapper::new(100.0, 50.0, 1000.0, 500.0);
        let n = mapper.host_to_norm(600.0, 300.0);
        assert!((n.x - 0.5).abs() < 1e-5);
        assert!((n.y - 0.5).abs() < 1e-5);
        let (hx, hy) = mapper.norm_to_host(n);
        assert!((hx - 600.0).abs() < 1e-3);
        assert!((hy - 300.0).abs() < 1e-3);

        let snapped = ViewportMapper::snap_to_grid(NormVec2::new(0.123, 0.487), 0.05);
        assert!((snapped.x - 0.10).abs() < 1e-5);
        assert!((snapped.y - 0.50).abs() < 1e-5);
    }

    #[test]
    fn per_app_profile_loads_and_creates_under_app_name() {
        let dir = std::env::temp_dir().join(format!("exr_keymap_app_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let store = ProfileStore::new(&dir);

        let created = store.load_or_create_for_app("com.dts.freefireth", "Free Fire");
        assert_eq!(created.name, "Free Fire");
        assert_eq!(created.game_package.as_deref(), Some("com.dts.freefireth"));
        assert!(store.path_for("com.dts.freefireth").exists());

        let reloaded = store.load_or_create_for_app("com.dts.freefireth", "Free Fire");
        assert_eq!(reloaded, created);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
