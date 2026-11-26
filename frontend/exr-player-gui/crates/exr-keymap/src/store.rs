use crate::schema::{KeymapProfile, PROFILE_SCHEMA_VERSION};
use exr_input::InputTrigger;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub control_id: String,
    pub message: String,
}

/// Validates a `KeymapProfile` for duplicate trigger conflicts, out-of-bounds coordinates, and schema errors.
pub fn validate_profile(profile: &KeymapProfile) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    let mut seen_triggers: HashMap<InputTrigger, Vec<String>> = HashMap::new();

    if profile.schema_version != PROFILE_SCHEMA_VERSION {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Warning,
            control_id: String::new(),
            message: format!(
                "Profile schema version {} differs from current {}",
                profile.schema_version, PROFILE_SCHEMA_VERSION
            ),
        });
    }

    for ctrl in &profile.controls {
        if ctrl.pos.x < 0.0 || ctrl.pos.x > 1.0 || ctrl.pos.y < 0.0 || ctrl.pos.y > 1.0 {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                control_id: ctrl.id.clone(),
                message: format!("Control '{}' position ({:.2}, {:.2}) out of [0,1] bounds", ctrl.label, ctrl.pos.x, ctrl.pos.y),
            });
        }
        if !ctrl.enabled {
            continue;
        }
        for trig in ctrl.kind.bound_triggers() {
            seen_triggers.entry(trig).or_default().push(ctrl.id.clone());
        }
    }

    for (trig, ids) in seen_triggers {
        if ids.len() > 1 {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                control_id: ids[0].clone(),
                message: format!("Trigger '{trig}' is bound to multiple controls: {}", ids.join(", ")),
            });
        }
    }

    issues
}

/// Undo/Redo stack for non-destructive profile editing.
#[derive(Debug, Clone)]
pub struct UndoRedoStack {
    undo: Vec<KeymapProfile>,
    redo: Vec<KeymapProfile>,
    max_depth: usize,
}

impl UndoRedoStack {
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            max_depth: max_depth.max(1),
        }
    }

    pub fn push_snapshot(&mut self, snapshot: KeymapProfile) {
        if self.undo.last() == Some(&snapshot) {
            return;
        }
        if self.undo.len() >= self.max_depth {
            self.undo.remove(0);
        }
        self.undo.push(snapshot);
        self.redo.clear();
    }

    pub fn undo(&mut self, current: &KeymapProfile) -> Option<KeymapProfile> {
        let prev = self.undo.pop()?;
        self.redo.push(current.clone());
        Some(prev)
    }

    pub fn redo(&mut self, current: &KeymapProfile) -> Option<KeymapProfile> {
        let next = self.redo.pop()?;
        self.undo.push(current.clone());
        Some(next)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}

/// Disk persistence manager for `KeymapProfile`s with corrupt-file recovery.
#[derive(Debug, Clone)]
pub struct ProfileStore {
    dir: PathBuf,
}

impl ProfileStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn path_for(&self, id: &str) -> PathBuf {
        let safe_id: String = id
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        self.dir.join(format!("{safe_id}.json"))
    }

    pub fn save(&self, profile: &KeymapProfile) -> Result<PathBuf, String> {
        fs::create_dir_all(&self.dir).map_err(|e| format!("create profile dir: {e}"))?;
        let path = self.path_for(&profile.id);
        let json = serde_json::to_string_pretty(profile).map_err(|e| format!("serialize profile: {e}"))?;
        fs::write(&path, json).map_err(|e| format!("write profile: {e}"))?;
        Ok(path)
    }

    pub fn load(&self, id: &str) -> Result<KeymapProfile, String> {
        let path = self.path_for(id);
        self.import_from_path(&path)
    }

    /// Loads a profile; if JSON is corrupted, backs up the corrupt file to `.corrupt` and returns a clean default.
    pub fn load_or_recover(&self, id: &str) -> KeymapProfile {
        let path = self.path_for(id);
        match self.import_from_path(&path) {
            Ok(p) => p,
            Err(_) if path.exists() => {
                let backup = path.with_extension("json.corrupt");
                let _ = fs::rename(&path, backup);
                let fallback = KeymapProfile::fps_default();
                let _ = self.save(&fallback);
                fallback
            }
            Err(_) => KeymapProfile::fps_default(),
        }
    }

    pub fn import_from_path(&self, path: &Path) -> Result<KeymapProfile, String> {
        let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let mut profile: KeymapProfile =
            serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))?;
        for ctrl in &mut profile.controls {
            ctrl.pos = ctrl.pos.clamped();
        }
        Ok(profile)
    }

    pub fn export_to_path(&self, profile: &KeymapProfile, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create dir: {e}"))?;
        }
        let json = serde_json::to_string_pretty(profile).map_err(|e| format!("serialize: {e}"))?;
        fs::write(path, json).map_err(|e| format!("write {}: {e}", path.display()))
    }

    pub fn delete(&self, id: &str) -> Result<(), String> {
        let path = self.path_for(id);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| format!("delete {}: {e}", path.display()))?;
        }
        Ok(())
    }

    pub fn list_ids(&self) -> Vec<String> {
        let Ok(entries) = fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut ids = Vec::new();
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                    ids.push(stem.to_string());
                }
            }
        }
        ids.sort();
        ids
    }

    /// Finds an existing profile matching `package` (either by `game_package` or `id`).
    pub fn find_by_package(&self, package: &str) -> Option<KeymapProfile> {
        if let Ok(direct) = self.load(package) {
            return Some(direct);
        }
        for id in self.list_ids() {
            if let Ok(p) = self.load(&id) {
                if p.game_package.as_deref() == Some(package) {
                    return Some(p);
                }
            }
        }
        None
    }

    /// Loads the `.json` profile for `package`, or creates & saves a new one under `app_name`.
    pub fn load_or_create_for_app(&self, package: &str, app_name: &str) -> KeymapProfile {
        if let Some(existing) = self.find_by_package(package) {
            return existing;
        }
        let mut profile = KeymapProfile::blank(package, app_name);
        profile.game_package = Some(package.to_string());
        let _ = self.save(&profile);
        profile
    }
}
