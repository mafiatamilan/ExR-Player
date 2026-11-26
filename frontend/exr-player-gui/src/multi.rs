#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]
mod theme;

use eframe::egui::{self, vec2, Color32, Stroke};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

#[derive(Clone, Debug)]
struct InstanceInfo {
    name: String,
    dir: PathBuf,
    config_path: PathBuf,
    disk_path: PathBuf,
    memory: String,
    smp: u64,
    display: String,
    adb_port: u64,
    disk_bytes: u64,
    selected: bool,
}

#[derive(Clone)]
struct EditDraft {
    dir: PathBuf,
    name: String,
    memory_mb: u32,
    smp: u32,
    display: String,
    vgamem_mb: u32,
    adb_port: u32,
    resize_gb: u32,
}

struct MultiApp {
    instances_root: PathBuf,
    images_root: PathBuf,
    engine_bin: PathBuf,
    img_bin: PathBuf,
    player_bin: PathBuf,
    instances: Vec<InstanceInfo>,
    available_images: Vec<(String, PathBuf)>,
    running: HashMap<String, Child>,
    search: String,
    status_msg: String,
    new_open: bool,
    new_name: String,
    new_mem_mb: u32,
    new_smp: u32,
    new_disk_gb: u32,
    new_image_idx: usize,
    edit_draft: Option<EditDraft>,
    delete_confirm: Option<String>,
}

fn find_bin(candidates: &[&str]) -> PathBuf {
    for c in candidates {
        let p = PathBuf::from(c);
        if p.exists() { return p; }
    }
    PathBuf::from(candidates[0])
}

fn read_host_ram_mib() -> (u64, u64) {
    let Ok(txt) = std::fs::read_to_string("/proc/meminfo") else { return (0, 0); };
    let mut total = 0u64;
    let mut avail = 0u64;
    for line in txt.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            total = rest.split_whitespace().next().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0) / 1024;
        } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
            avail = rest.split_whitespace().next().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0) / 1024;
        }
    }
    (avail, total)
}

fn parse_instance(dir: &Path) -> Option<InstanceInfo> {
    let cfg_path = dir.join("config.exr");
    let raw = std::fs::read_to_string(&cfg_path).ok()?;
    let v: Value = serde_json::from_str(&raw).ok()?;
    let name = dir.file_name()?.to_string_lossy().into_owned();
    let engine = v.get("engine")?;
    let memory = engine.get("memory").and_then(|x| x.as_str()).unwrap_or("768M").to_string();
    let smp = engine.get("smp").and_then(|x| x.as_u64()).unwrap_or(1);
    let display = engine.get("display").and_then(|x| x.as_str()).unwrap_or("auto").to_string();
    let adb_port = v.get("networking").and_then(|n| n.get("adb-port")).and_then(|x| x.as_u64()).unwrap_or(5555);
    let disk_rel = v.get("storage").and_then(|s| s.get("path")).and_then(|x| x.as_str()).unwrap_or("system.exrimg");
    let disk_path = dir.join(disk_rel);
    let disk_bytes = std::fs::metadata(&disk_path).map(|m| m.len()).unwrap_or(0);
    Some(InstanceInfo {
        name,
        dir: dir.to_path_buf(),
        config_path: cfg_path,
        disk_path,
        memory,
        smp,
        display,
        adb_port,
        disk_bytes,
        selected: false,
    })
}

fn write_default_config(dir: &Path, mem_mb: u32, smp: u32, disk_gb: u32, adb_port: u32) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let cfg_path = dir.join("config.exr");
    let json = serde_json::json!({
        "schema-version": 1,
        "engine": {
            "architecture": "x86_64",
            "acceleration": "auto",
            "memory": format!("{}M", mem_mb),
            "cpu": "max",
            "machine": "pc",
            "smp": smp,
            "profile": if mem_mb <= 1024 { "low-mem-2gb" } else { "standard" },
            "balloon": true,
            "inputs": true,
            "display": "auto",
            "vgamem-mb": if mem_mb <= 1024 { 16 } else { 32 }
        },
        "storage": {
            "path": "system.exrimg",
            "format": "exrimg",
            "interface": "virtio",
            "size": format!("{}G", disk_gb)
        },
        "audio": { "backend": "auto" },
        "networking": { "backend": "auto", "adb-port": adb_port },
        "runtime": { "socket": "unix:exr-x86.sock", "pidfile": "exr-x86.pid", "serial": "stdio" }
    });
    std::fs::write(&cfg_path, serde_json::to_string_pretty(&json).unwrap())?;
    Ok(cfg_path)
}

impl MultiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install(&cc.egui_ctx);
        let instances_root = PathBuf::from("instances");
        let images_root = PathBuf::from("images");
        let _ = std::fs::create_dir_all(&instances_root);
        let _ = std::fs::create_dir_all(&images_root);
        let engine_bin = find_bin(&["dist/engine/bin/exr-x86", "engine/bin/exr-x86", "../../dist/engine/bin/exr-x86", "engine/build/exr-x86"]);
        let img_bin = find_bin(&["dist/engine/bin/exr-img", "engine/bin/exr-img", "../../dist/engine/bin/exr-img", "engine/build/exr-img"]);
        let player_bin = find_bin(&["dist/exr-player", "./exr-player", "frontend/exr-player-gui/target/debug/exr-player"]);

        let mut app = Self {
            instances_root,
            images_root,
            engine_bin,
            img_bin,
            player_bin,
            instances: Vec::new(),
            available_images: Vec::new(),
            running: HashMap::new(),
            search: String::new(),
            status_msg: "Ready".into(),
            new_open: false,
            new_name: "Android-1".into(),
            new_mem_mb: 768,
            new_smp: 2,
            new_disk_gb: 16,
            new_image_idx: 0,
            edit_draft: None,
            delete_confirm: None,
        };
        app.refresh();
        app
    }

    fn next_adb_port(&self) -> u32 {
        let max_port = self.instances.iter().map(|i| i.adb_port as u32).max().unwrap_or(5553);
        (max_port + 2).max(5555)
    }

    fn refresh(&mut self) {
        let mut list = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&self.instances_root) {
            for entry in rd.flatten() {
                if entry.path().is_dir() {
                    if let Some(info) = parse_instance(&entry.path()) {
                        list.push(info);
                    }
                }
            }
        }
        list.sort_by(|a, b| a.name.cmp(&b.name));
        self.instances = list;

        let mut imgs = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&self.images_root) {
            for entry in rd.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("exrimg") {
                    let label = p.file_stem().and_then(|s| s.to_str()).unwrap_or("image").to_string();
                    imgs.push((label, p));
                } else if p.is_dir() && p.join("system.exrimg").exists() {
                    let label = p.file_name().and_then(|s| s.to_str()).unwrap_or("image").to_string();
                    imgs.push((label, p.join("system.exrimg")));
                }
            }
        }
        imgs.sort_by(|a, b| a.0.cmp(&b.0));
        self.available_images = imgs;
        if self.new_image_idx > self.available_images.len() {
            self.new_image_idx = 0;
        }
    }

    fn reap_children(&mut self) {
        let mut exited = Vec::new();
        for (name, child) in &mut self.running {
            if let Ok(Some(_)) = child.try_wait() {
                exited.push(name.clone());
            }
        }
        for name in exited {
            self.running.remove(&name);
        }
    }

    fn start_instance(&mut self, inst: &InstanceInfo) {
        if self.running.contains_key(&inst.name) { return; }
        match Command::new(&self.player_bin).arg("--config").arg(&inst.config_path).spawn() {
            Ok(child) => {
                self.running.insert(inst.name.clone(), child);
                self.status_msg = format!("Started {}", inst.name);
            }
            Err(e) => {
                self.status_msg = format!("Failed starting {}: {}", inst.name, e);
            }
        }
    }

    fn stop_instance(&mut self, name: &str) {
        if let Some(mut child) = self.running.remove(name) {
            let _ = child.kill();
            let _ = child.wait();
            self.status_msg = format!("Stopped {}", name);
        }
    }

    fn create_instance(&mut self, name: &str, mem_mb: u32, smp: u32, disk_gb: u32, img_idx: usize) {
        let clean = name.trim().replace('/', "_");
        if clean.is_empty() { return; }
        let dir = self.instances_root.join(&clean);
        if dir.exists() {
            self.status_msg = format!("Instance '{}' already exists", clean);
            return;
        }
        let port = self.next_adb_port();
        match write_default_config(&dir, mem_mb, smp, disk_gb, port) {
            Ok(cfg) => {
                if img_idx > 0 {
                    if let Some((label, src_img)) = self.available_images.get(img_idx - 1) {
                        match std::fs::copy(src_img, dir.join("system.exrimg")) {
                            Ok(_) => self.status_msg = format!("Created '{}' from image '{}'", clean, label),
                            Err(e) => self.status_msg = format!("Image copy failed: {}", e),
                        }
                        self.refresh();
                        return;
                    }
                }
                let out = Command::new(&self.engine_bin).arg("create").arg(&cfg).output();
                match out {
                    Ok(o) if o.status.success() => {
                        self.status_msg = format!("Created blank instance '{}'", clean);
                    }
                    Ok(o) => {
                        self.status_msg = format!("Disk create failed: {}", String::from_utf8_lossy(&o.stderr));
                    }
                    Err(e) => {
                        self.status_msg = format!("Engine error: {}", e);
                    }
                }
            }
            Err(e) => self.status_msg = format!("Write config failed: {}", e),
        }
        self.refresh();
    }

    fn clone_instance(&mut self, inst: &InstanceInfo) {
        let mut n = 2u32;
        let mut target_name = format!("{}-clone", inst.name);
        while self.instances_root.join(&target_name).exists() {
            target_name = format!("{}-clone-{}", inst.name, n);
            n += 1;
        }
        let target_dir = self.instances_root.join(&target_name);
        if let Err(e) = std::fs::create_dir_all(&target_dir) {
            self.status_msg = format!("Clone failed: {}", e);
            return;
        }
        if inst.disk_path.exists() {
            if let Err(e) = std::fs::copy(&inst.disk_path, target_dir.join("system.exrimg")) {
                self.status_msg = format!("Disk copy failed: {}", e);
                return;
            }
        }
        if let Ok(raw) = std::fs::read_to_string(&inst.config_path) {
            if let Ok(mut v) = serde_json::from_str::<Value>(&raw) {
                if let Some(net) = v.get_mut("networking").and_then(|n| n.as_object_mut()) {
                    net.insert("adb-port".into(), Value::from(self.next_adb_port()));
                }
                let _ = std::fs::write(target_dir.join("config.exr"), serde_json::to_string_pretty(&v).unwrap());
            }
        }
        self.status_msg = format!("Cloned '{}' -> '{}'", inst.name, target_name);
        self.refresh();
    }
}

impl eframe::App for MultiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _f: &mut eframe::Frame) {
        self.reap_children();
        let ctx = ui.ctx().clone();

        egui::Panel::top("multi_header").exact_size(52.0)
            .frame(egui::Frame::new().fill(theme::BG_PANEL)).show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(16.0);
                    ui.label(egui::RichText::new("ExR Multi-Instance Manager").size(16.0).family(theme::semibold()).color(theme::TEXT));
                    ui.add_space(16.0);
                    ui.add(egui::TextEdit::singleline(&mut self.search).hint_text("Filter instances...").desired_width(180.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(16.0);
                        if ui.button("Refresh").clicked() {
                            self.refresh();
                            self.status_msg = format!("{} instance(s), {} image(s) in images/", self.instances.len(), self.available_images.len());
                        }
                        if ui.button("+ New").clicked() {
                            self.refresh();
                            self.new_name = format!("Android-{}", self.instances.len() + 1);
                            self.new_open = true;
                        }
                        if ui.button("Stop All").clicked() {
                            let keys: Vec<String> = self.running.keys().cloned().collect();
                            for k in keys { self.stop_instance(&k); }
                        }
                        if ui.button("Start Selected").clicked() {
                            let selected: Vec<InstanceInfo> = self.instances.iter().filter(|i| i.selected).cloned().collect();
                            for inst in selected { self.start_instance(&inst); }
                        }
                    });
                });
            });

        egui::Panel::bottom("multi_footer").exact_size(34.0)
            .frame(egui::Frame::new().fill(theme::BG_PANEL)).show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(16.0);
                    let (avail, total) = read_host_ram_mib();
                    ui.label(egui::RichText::new(format!("Host RAM: {} / {} MiB free   |   Running: {}   |   {}", avail, total, self.running.len(), self.status_msg))
                        .size(12.0).color(theme::TEXT_DIM));
                });
            });

        egui::CentralPanel::default().frame(egui::Frame::new().fill(theme::BG_BASE).inner_margin(egui::Margin::same(16))).show(ui, |ui| {
            let filter = self.search.to_lowercase();
            let list: Vec<InstanceInfo> = self.instances.iter()
                .filter(|i| filter.is_empty() || i.name.to_lowercase().contains(&filter))
                .cloned().collect();

            if list.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new("No instances yet. Click '+ New' or 'Import ISO' to create one.").color(theme::TEXT_DIM));
                });
                return;
            }

            let mut to_start = None;
            let mut to_stop = None;
            let mut to_clone = None;
            let mut to_check = None;

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (idx, inst) in list.iter().enumerate() {
                    let is_running = self.running.contains_key(&inst.name);
                    egui::Frame::new().fill(theme::BG_PANEL).stroke(Stroke::new(1.0, theme::BORDER)).corner_radius(8.0)
                        .inner_margin(egui::Margin::symmetric(14, 10)).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                if let Some(orig) = self.instances.iter_mut().find(|x| x.name == inst.name) {
                                    ui.checkbox(&mut orig.selected, "");
                                }
                                let dot_col = if is_running { theme::ACCENT_GREEN } else { theme::TEXT_DIM };
                                ui.label(egui::RichText::new("●").color(dot_col));
                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new(&inst.name).size(14.0).family(theme::semibold()).color(theme::TEXT));
                                    ui.label(egui::RichText::new(format!(
                                        "{} RAM  |  {} vCPU  |  GPU: {}  |  ADB :{}  |  Disk: {:.1} MiB",
                                        inst.memory, inst.smp, inst.display, inst.adb_port, inst.disk_bytes as f64 / (1024.0 * 1024.0)
                                    )).size(11.5).color(theme::TEXT_DIM));
                                });
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if !is_running && ui.button("Delete").clicked() {
                                        self.delete_confirm = Some(inst.name.clone());
                                    }
                                    if !is_running && ui.button("Check").clicked() {
                                        to_check = Some(inst.clone());
                                    }
                                    if !is_running && ui.button("Clone").clicked() {
                                        to_clone = Some(inst.clone());
                                    }
                                    if !is_running && ui.button("Settings").clicked() {
                                        let mem_num = inst.memory.trim_end_matches(|c: char| !c.is_ascii_digit()).parse().unwrap_or(768);
                                        self.edit_draft = Some(EditDraft {
                                            dir: inst.dir.clone(),
                                            name: inst.name.clone(),
                                            memory_mb: mem_num,
                                            smp: inst.smp as u32,
                                            display: inst.display.clone(),
                                            vgamem_mb: 32,
                                            adb_port: inst.adb_port as u32,
                                            resize_gb: 16,
                                        });
                                    }
                                    if is_running {
                                        if ui.button("Stop").clicked() { to_stop = Some(inst.name.clone()); }
                                    } else if ui.button("Start").clicked() {
                                        to_start = Some(inst.clone());
                                    }
                                });
                            });
                        });
                    if idx + 1 < list.len() { ui.add_space(6.0); }
                }
            });

            if let Some(inst) = to_start { self.start_instance(&inst); }
            if let Some(name) = to_stop { self.stop_instance(&name); }
            if let Some(inst) = to_clone { self.clone_instance(&inst); }
            if let Some(inst) = to_check {
                let ok = Command::new(&self.img_bin).args(["check", "-q"]).arg(&inst.disk_path).status().map(|s| s.success()).unwrap_or(false);
                self.status_msg = if ok { format!("{}: .exrimg integrity OK", inst.name) } else { format!("{}: .exrimg check failed", inst.name) };
            }
        });

        /* New Instance Modal */
        if self.new_open {
            let mut close = false;
            egui::Window::new("New Instance").collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0)).show(&ctx, |ui| {
                ui.horizontal(|ui| { ui.label("Name:"); ui.text_edit_singleline(&mut self.new_name); });
                ui.horizontal(|ui| {
                    ui.label("Image (images/):");
                    let cur_label = if self.new_image_idx == 0 {
                        "Blank Disk".to_string()
                    } else {
                        self.available_images.get(self.new_image_idx - 1).map(|(l, _)| l.clone()).unwrap_or_else(|| "Blank Disk".into())
                    };
                    egui::ComboBox::from_id_salt("new_image_sel").selected_text(cur_label).show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.new_image_idx, 0, "Blank Disk");
                        for (idx, (label, _)) in self.available_images.iter().enumerate() {
                            ui.selectable_value(&mut self.new_image_idx, idx + 1, label);
                        }
                    });
                });
                ui.add(egui::Slider::new(&mut self.new_mem_mb, 256..=8192).step_by(128.0).text("RAM (MB)"));
                ui.add(egui::Slider::new(&mut self.new_smp, 1..=16).text("CPU cores"));
                if self.new_image_idx == 0 {
                    ui.add(egui::Slider::new(&mut self.new_disk_gb, 2..=128).text("Disk (GB)"));
                }
                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        let (n, m, s, d, img) = (self.new_name.clone(), self.new_mem_mb, self.new_smp, self.new_disk_gb, self.new_image_idx);
                        self.create_instance(&n, m, s, d, img);
                        close = true;
                    }
                    if ui.button("Cancel").clicked() { close = true; }
                });
            });
            if close { self.new_open = false; }
        }

        /* Configure Instance Modal */
        if let Some(mut d) = self.edit_draft.clone() {
            let mut close = false;
            let mut save = false;
            egui::Window::new(format!("Configure {}", d.name)).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0)).show(&ctx, |ui| {
                ui.add(egui::Slider::new(&mut d.memory_mb, 256..=16384).step_by(128.0).text("RAM (MB)"));
                ui.add(egui::Slider::new(&mut d.smp, 1..=16).text("CPU cores"));
                ui.horizontal(|ui| {
                    ui.label("GPU:");
                    for mode in ["auto", "virtio-gpu", "virtio-gpu-gl", "none"] {
                        ui.selectable_value(&mut d.display, mode.into(), mode);
                    }
                });
                ui.add(egui::Slider::new(&mut d.vgamem_mb, 16..=256).step_by(16.0).text("VRAM (MB)"));
                ui.add(egui::DragValue::new(&mut d.adb_port).range(1024..=65535).prefix("ADB port: "));
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut d.resize_gb).range(2..=512).suffix(" GB"));
                    if ui.button("Grow Disk").clicked() {
                        let cfg = d.dir.join("config.exr");
                        let ok = Command::new(&self.engine_bin).arg("resize").arg(&cfg).arg(format!("{}G", d.resize_gb)).status().map(|s| s.success()).unwrap_or(false);
                        self.status_msg = if ok { format!("Resized {} to {}G", d.name, d.resize_gb) } else { "Resize failed (can only grow)".into() };
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("Save config.exr").clicked() { save = true; close = true; }
                    if ui.button("Cancel").clicked() { close = true; }
                });
            });
            if save {
                let cfg_path = d.dir.join("config.exr");
                if let Ok(raw) = std::fs::read_to_string(&cfg_path) {
                    if let Ok(mut v) = serde_json::from_str::<Value>(&raw) {
                        if let Some(eng) = v.get_mut("engine").and_then(|e| e.as_object_mut()) {
                            eng.insert("memory".into(), Value::from(format!("{}M", d.memory_mb)));
                            eng.insert("smp".into(), Value::from(d.smp));
                            eng.insert("display".into(), Value::from(d.display.clone()));
                            eng.insert("vgamem-mb".into(), Value::from(d.vgamem_mb));
                        }
                        if let Some(net) = v.get_mut("networking").and_then(|n| n.as_object_mut()) {
                            net.insert("adb-port".into(), Value::from(d.adb_port));
                        }
                        let _ = std::fs::write(&cfg_path, serde_json::to_string_pretty(&v).unwrap());
                        self.status_msg = format!("Saved {}", cfg_path.display());
                        self.refresh();
                    }
                }
            }
            self.edit_draft = if close { None } else { Some(d) };
        }

        /* Delete Confirmation */
        if let Some(name) = self.delete_confirm.clone() {
            let mut close = false;
            egui::Window::new("Delete Instance").collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0)).show(&ctx, |ui| {
                ui.label(egui::RichText::new(format!("Permanently delete '{}' and its .exrimg disk?", name)).color(Color32::from_rgb(0xFF, 0x80, 0x80)));
                ui.horizontal(|ui| {
                    if ui.button("Delete").clicked() {
                        let _ = std::fs::remove_dir_all(self.instances_root.join(&name));
                        self.status_msg = format!("Deleted '{}'", name);
                        self.refresh();
                        close = true;
                    }
                    if ui.button("Cancel").clicked() { close = true; }
                });
            });
            if close { self.delete_confirm = None; }
        }
    }
}

fn main() -> eframe::Result<()> {
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ExR Multi-Instance Manager")
            .with_inner_size(vec2(860.0, 560.0))
            .with_min_inner_size(vec2(640.0, 420.0)),
        ..Default::default()
    };
    eframe::run_native("ExR Multi-Instance Manager", opts, Box::new(|cc| Ok(Box::new(MultiApp::new(cc)))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_and_parse_instance_config() {
        let dir = std::env::temp_dir().join("exr-multi-test-inst");
        let _ = std::fs::remove_dir_all(&dir);
        let cfg = write_default_config(&dir, 768, 2, 16, 5555).unwrap();
        assert!(cfg.exists());
        let parsed = parse_instance(&dir).unwrap();
        assert_eq!(parsed.memory, "768M");
        assert_eq!(parsed.smp, 2);
        assert_eq!(parsed.display, "auto");
        assert_eq!(parsed.adb_port, 5555);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
