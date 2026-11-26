//! The ONLY boundary between GUI and emulator. Spec: docs/06 §3, §5.
use crate::settings::model::Settings;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
pub enum InstanceState { NoInstance, Booting(String), Running, Crashed(String) }

pub struct Frame { pub id: u64, pub w: usize, pub h: usize, pub rgba: Vec<u8> }

#[derive(Clone, Copy, Debug)]
pub enum NavKey { Back, Home, Recents, VolumeUp, VolumeDown }

pub trait EmulatorBackend: Send {
    fn state(&self) -> InstanceState;
    /// Returns a frame only if newer than `last_id` (avoid redundant texture uploads).
    fn frame_if_new(&mut self, last_id: u64) -> Option<Frame>;
    fn touch_down(&mut self, finger: u8, nx: f32, ny: f32);
    fn touch_move(&mut self, finger: u8, nx: f32, ny: f32);
    fn touch_up(&mut self, finger: u8);
    fn nav(&mut self, key: NavKey);
    fn install_apk(&mut self, path: PathBuf);
    fn restart(&mut self);
    fn apply_settings(&mut self, _settings: &Settings) {}
    fn set_repaint_hook(&mut self, hook: Arc<dyn Fn() + Send + Sync>);
    /// Optional foreground Android app `(package_id, display_name)` reported by the guest.
    fn foreground_app(&self) -> Option<(String, String)> { None }
}

/* --- C ABI bindings to libexr-x86.so (engine/include/exr/api.h) --- */
#[repr(C)]
struct ExRMachineOpaque { _priv: [u8; 0] }
#[repr(C)]
struct ExRConfigOpaque { _priv: [u8; 0] }

#[allow(dead_code)]
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum ExrAccel { Auto = 0, Tcg = 1, Kvm = 2, Whpx = 3, Hvf = 4 }

#[allow(dead_code)]
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum ExrProfile { Standard = 0, LowMem2Gb = 1, Minimal = 2 }

#[allow(dead_code)]
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum ExrMouseBtn { Left = 0, Middle = 1, Right = 2 }

#[allow(dead_code)]
extern "C" {
    fn exr_config_create() -> *mut ExRConfigOpaque;
    fn exr_config_load(exr_path: *const c_char, out_cfg: *mut *mut ExRConfigOpaque) -> c_int;
    fn exr_config_save(cfg: *const ExRConfigOpaque, exr_path: *const c_char) -> c_int;
    fn exr_config_free(cfg: *mut ExRConfigOpaque);
    fn exr_config_set_engine_path(cfg: *mut ExRConfigOpaque, path: *const c_char) -> c_int;
    fn exr_config_set_accel(cfg: *mut ExRConfigOpaque, accel: ExrAccel) -> c_int;
    fn exr_config_set_memory(cfg: *mut ExRConfigOpaque, memory_str: *const c_char) -> c_int;
    fn exr_config_set_profile(cfg: *mut ExRConfigOpaque, profile: ExrProfile) -> c_int;
    fn exr_config_set_smp(cfg: *mut ExRConfigOpaque, smp: u16) -> c_int;
    fn exr_config_set_balloon(cfg: *mut ExRConfigOpaque, enabled: bool) -> c_int;
    fn exr_config_set_display(cfg: *mut ExRConfigOpaque, display: *const c_char) -> c_int;
    fn exr_config_set_vgamem(cfg: *mut ExRConfigOpaque, vgamem_mb: u16) -> c_int;
    fn exr_config_set_inputs(cfg: *mut ExRConfigOpaque, enabled: bool) -> c_int;
    fn exr_config_set_audio(cfg: *mut ExRConfigOpaque, backend: *const c_char, wav_path: *const c_char) -> c_int;
    fn exr_config_set_network(cfg: *mut ExRConfigOpaque, backend: *const c_char) -> c_int;
    fn exr_config_set_adb_port(cfg: *mut ExRConfigOpaque, port: u16) -> c_int;
    fn exr_config_set_shared_folder(cfg: *mut ExRConfigOpaque, host_path: *const c_char, readonly: bool) -> c_int;

    fn exr_machine_create(cfg: *const ExRConfigOpaque, out_machine: *mut *mut ExRMachineOpaque) -> c_int;
    fn exr_machine_start(machine: *mut ExRMachineOpaque) -> c_int;
    fn exr_machine_stop(machine: *mut ExRMachineOpaque) -> c_int;
    fn exr_machine_reset(machine: *mut ExRMachineOpaque) -> c_int;
    fn exr_machine_destroy(machine: *mut ExRMachineOpaque);

    fn exr_machine_send_key(machine: *mut ExRMachineOpaque, qcode: *const c_char, down: bool) -> c_int;
    fn exr_machine_send_pointer_abs(machine: *mut ExRMachineOpaque, x: i32, y: i32) -> c_int;
    fn exr_machine_send_pointer_btn(machine: *mut ExRMachineOpaque, btn: ExrMouseBtn, down: bool) -> c_int;
    fn exr_machine_set_balloon_target(machine: *mut ExRMachineOpaque, target_bytes: u64) -> c_int;
    fn exr_machine_capture_screen(machine: *mut ExRMachineOpaque, ppm_path: *const c_char) -> c_int;
    fn exr_get_last_error() -> *const c_char;
}

struct SendMachine(*mut ExRMachineOpaque);
unsafe impl Send for SendMachine {}
unsafe impl Sync for SendMachine {}

/// Live backend backed by `libexr-x86.so` + `config.exr`.
pub struct ExrEngineBackend {
    machine: Arc<Mutex<Option<SendMachine>>>,
    state: Arc<Mutex<InstanceState>>,
    latest_frame: Arc<Mutex<Option<Frame>>>,
    hook: Arc<Mutex<Option<Arc<dyn Fn() + Send + Sync>>>>,
    stop_flag: Arc<AtomicBool>,
    capture_thread: Option<JoinHandle<()>>,
    config_path: PathBuf,
    engine_bin: PathBuf,
    adb_port: u16,
}

fn parse_ppm_p6(data: &[u8]) -> Option<(usize, usize, Vec<u8>)> {
    if !data.starts_with(b"P6") { return None; }
    let mut i = 2usize;
    let mut nums = [0usize; 3];
    let mut idx = 0usize;
    while idx < 3 && i < data.len() {
        while i < data.len() && data[i].is_ascii_whitespace() { i += 1; }
        if i < data.len() && data[i] == b'#' {
            while i < data.len() && data[i] != b'\n' { i += 1; }
            continue;
        }
        let start = i;
        while i < data.len() && data[i].is_ascii_digit() { i += 1; }
        if start == i { return None; }
        nums[idx] = std::str::from_utf8(&data[start..i]).ok()?.parse().ok()?;
        idx += 1;
    }
    if idx != 3 || nums[2] != 255 || i >= data.len() { return None; }
    i += 1; // single whitespace byte after maxval
    let (w, h) = (nums[0], nums[1]);
    let expected = w.checked_mul(h)?.checked_mul(3)?;
    if data.len() < i + expected { return None; }
    let rgb = &data[i..i + expected];
    let mut rgba = Vec::with_capacity(w * h * 4);
    for px in rgb.chunks_exact(3) {
        rgba.extend_from_slice(&[px[0], px[1], px[2], 255]);
    }
    Some((w, h, rgba))
}

impl ExrEngineBackend {
    pub fn find_paths(config_override: Option<PathBuf>) -> Option<(PathBuf, PathBuf)> {
        let candidates = [
            PathBuf::from("dist/engine/bin/exr-x86"),
            PathBuf::from("../../dist/engine/bin/exr-x86"),
            PathBuf::from("engine/build/exr-x86"),
            PathBuf::from("../../engine/build/exr-x86"),
        ];
        let bin = candidates.into_iter().find(|p| p.exists())?;
        let cfg = config_override.unwrap_or_else(|| PathBuf::from("config.exr"));
        Some((bin, cfg))
    }

    pub fn start_with(engine_bin: PathBuf, config_path: PathBuf, settings: &Settings) -> Self {
        let state = Arc::new(Mutex::new(InstanceState::Booting("Starting ExR x86...".into())));
        let machine_arc = Arc::new(Mutex::new(None));
        let latest_frame = Arc::new(Mutex::new(None));
        let hook: Arc<Mutex<Option<Arc<dyn Fn() + Send + Sync>>>> = Arc::new(Mutex::new(None));
        let stop_flag = Arc::new(AtomicBool::new(false));
        let adb_port = 5555u16;

        unsafe {
            let mut cfg: *mut ExRConfigOpaque = std::ptr::null_mut();
            let c_cfg_path = CString::new(config_path.to_string_lossy().as_ref()).unwrap();
            if config_path.exists() {
                let _ = exr_config_load(c_cfg_path.as_ptr(), &mut cfg);
            }
            if cfg.is_null() {
                cfg = exr_config_create();
                let mem = CString::new(format!("{}M", settings.memory_mb)).unwrap();
                let disp = CString::new("auto").unwrap();
                let audio = CString::new("none").unwrap();
                let net = CString::new("auto").unwrap();
                exr_config_set_accel(cfg, ExrAccel::Auto);
                exr_config_set_memory(cfg, mem.as_ptr());
                exr_config_set_smp(cfg, settings.cpu_cores.max(1) as u16);
                exr_config_set_profile(cfg, if settings.gpu_mem_opt { ExrProfile::LowMem2Gb } else { ExrProfile::Standard });
                exr_config_set_balloon(cfg, true);
                exr_config_set_display(cfg, disp.as_ptr());
                exr_config_set_vgamem(cfg, if settings.gpu_mem_opt { 16 } else { 32 });
                exr_config_set_inputs(cfg, true);
                exr_config_set_audio(cfg, audio.as_ptr(), std::ptr::null());
                exr_config_set_network(cfg, net.as_ptr());
                exr_config_set_adb_port(cfg, adb_port);
                let _ = exr_config_save(cfg, c_cfg_path.as_ptr());
            }
            let c_bin = CString::new(engine_bin.to_string_lossy().as_ref()).unwrap();
            exr_config_set_engine_path(cfg, c_bin.as_ptr());

            let mut m: *mut ExRMachineOpaque = std::ptr::null_mut();
            if exr_machine_create(cfg, &mut m) == 0 && exr_machine_start(m) == 0 {
                *machine_arc.lock().unwrap() = Some(SendMachine(m));
                *state.lock().unwrap() = InstanceState::Running;
            } else {
                let err = CStr::from_ptr(exr_get_last_error()).to_string_lossy().into_owned();
                *state.lock().unwrap() = InstanceState::Crashed(err);
                if !m.is_null() { exr_machine_destroy(m); }
            }
            exr_config_free(cfg);
        }

        let m_clone = machine_arc.clone();
        let f_clone = latest_frame.clone();
        let h_clone = hook.clone();
        let s_clone = stop_flag.clone();
        let capture_thread = thread::spawn(move || {
            let ppm_path = format!("/tmp/exr-gui-fb-{}.ppm", std::process::id());
            let c_ppm = CString::new(ppm_path.as_str()).unwrap();
            let mut next_id = 1u64;
            while !s_clone.load(Ordering::Relaxed) {
                let ptr = { m_clone.lock().unwrap().as_ref().map(|sm| sm.0) };
                if let Some(m_ptr) = ptr {
                    let rc = unsafe { exr_machine_capture_screen(m_ptr, c_ppm.as_ptr()) };
                    if rc == 0 {
                        if let Ok(bytes) = std::fs::read(&ppm_path) {
                            if let Some((w, h, rgba)) = parse_ppm_p6(&bytes) {
                                *f_clone.lock().unwrap() = Some(Frame { id: next_id, w, h, rgba });
                                next_id += 1;
                                if let Some(cb) = h_clone.lock().unwrap().as_ref() {
                                    cb();
                                }
                            }
                        }
                    }
                }
                thread::sleep(Duration::from_millis(33));
            }
            let _ = std::fs::remove_file(&ppm_path);
        });

        Self {
            machine: machine_arc,
            state,
            latest_frame,
            hook,
            stop_flag,
            capture_thread: Some(capture_thread),
            config_path,
            engine_bin,
            adb_port,
        }
    }
}

impl Drop for ExrEngineBackend {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(t) = self.capture_thread.take() {
            let _ = t.join();
        }
        if let Some(SendMachine(m)) = self.machine.lock().unwrap().take() {
            unsafe {
                exr_machine_stop(m);
                exr_machine_destroy(m);
            }
        }
    }
}

impl EmulatorBackend for ExrEngineBackend {
    fn state(&self) -> InstanceState { self.state.lock().unwrap().clone() }

    fn frame_if_new(&mut self, last_id: u64) -> Option<Frame> {
        let mut guard = self.latest_frame.lock().unwrap();
        if guard.as_ref().map_or(false, |f| f.id > last_id) {
            guard.take()
        } else {
            None
        }
    }

    fn touch_down(&mut self, _finger: u8, nx: f32, ny: f32) {
        if let Some(sm) = self.machine.lock().unwrap().as_ref() {
            let x = (nx.clamp(0.0, 1.0) * 32767.0) as i32;
            let y = (ny.clamp(0.0, 1.0) * 32767.0) as i32;
            unsafe {
                exr_machine_send_pointer_abs(sm.0, x, y);
                exr_machine_send_pointer_btn(sm.0, ExrMouseBtn::Left, true);
            }
        }
    }

    fn touch_move(&mut self, _finger: u8, nx: f32, ny: f32) {
        if let Some(sm) = self.machine.lock().unwrap().as_ref() {
            let x = (nx.clamp(0.0, 1.0) * 32767.0) as i32;
            let y = (ny.clamp(0.0, 1.0) * 32767.0) as i32;
            unsafe { exr_machine_send_pointer_abs(sm.0, x, y); }
        }
    }

    fn touch_up(&mut self, _finger: u8) {
        if let Some(sm) = self.machine.lock().unwrap().as_ref() {
            unsafe { exr_machine_send_pointer_btn(sm.0, ExrMouseBtn::Left, false); }
        }
    }

    fn nav(&mut self, key: NavKey) {
        let qcode = match key {
            NavKey::Back => "esc",
            NavKey::Home => "meta_l",
            NavKey::Recents => "tab",
            NavKey::VolumeUp => "volumeup",
            NavKey::VolumeDown => "volumedown",
        };
        if let Some(sm) = self.machine.lock().unwrap().as_ref() {
            let c_key = CString::new(qcode).unwrap();
            unsafe {
                exr_machine_send_key(sm.0, c_key.as_ptr(), true);
                exr_machine_send_key(sm.0, c_key.as_ptr(), false);
            }
        }
    }

    fn install_apk(&mut self, path: PathBuf) {
        let target = format!("127.0.0.1:{}", self.adb_port);
        let _ = Command::new("adb").args(["-s", &target, "install", "-r"]).arg(path).spawn();
    }

    fn restart(&mut self) {
        if let Some(sm) = self.machine.lock().unwrap().as_ref() {
            unsafe { exr_machine_reset(sm.0); }
        }
    }

    fn apply_settings(&mut self, s: &Settings) {
        unsafe {
            let mut cfg: *mut ExRConfigOpaque = std::ptr::null_mut();
            let c_cfg_path = CString::new(self.config_path.to_string_lossy().as_ref()).unwrap();
            if self.config_path.exists() {
                let _ = exr_config_load(c_cfg_path.as_ptr(), &mut cfg);
            }
            if cfg.is_null() {
                cfg = exr_config_create();
            }
            let mem = CString::new(format!("{}M", s.memory_mb)).unwrap();
            exr_config_set_memory(cfg, mem.as_ptr());
            exr_config_set_smp(cfg, s.cpu_cores.max(1) as u16);
            exr_config_set_vgamem(cfg, if s.gpu_mem_opt { 16 } else { 32 });
            let _ = exr_config_save(cfg, c_cfg_path.as_ptr());
            exr_config_free(cfg);

            if let Some(sm) = self.machine.lock().unwrap().as_ref() {
                let target_bytes = (s.memory_mb as u64) * 1024 * 1024;
                let _ = exr_machine_set_balloon_target(sm.0, target_bytes);
            }
        }
        let _ = &self.engine_bin;
    }

    fn set_repaint_hook(&mut self, h: Arc<dyn Fn() + Send + Sync>) {
        *self.hook.lock().unwrap() = Some(h);
    }
}

/// Animated gradient test pattern at 1280x720 @ ~30 fps — exercises the real texture path.
pub struct MockBackend { id: u64, t0: Instant, hook: Option<Arc<dyn Fn() + Send + Sync>>, pub log: Arc<Mutex<Vec<String>>> }
impl MockBackend {
    pub fn new() -> Self { Self { id: 0, t0: Instant::now(), hook: None, log: Default::default() } }
    fn push(&self, s: String) { if let Ok(mut l) = self.log.lock() { l.push(s); if l.len() > 200 { l.remove(0); } } }
}
impl EmulatorBackend for MockBackend {
    fn state(&self) -> InstanceState { InstanceState::Running }
    fn frame_if_new(&mut self, last_id: u64) -> Option<Frame> {
        let id = (self.t0.elapsed().as_millis() / 33) as u64 + 1;
        if id == last_id { return None; }
        self.id = id;
        let (w, h) = (320usize, 180usize);
        let ph = (id % 256) as u8;
        let mut rgba = Vec::with_capacity(w * h * 4);
        for y in 0..h { for x in 0..w {
            rgba.extend_from_slice(&[(x * 255 / w) as u8 ^ ph, (y * 255 / h) as u8, 160, 255]);
        } }
        Some(Frame { id, w, h, rgba })
    }
    fn touch_down(&mut self, f: u8, x: f32, y: f32) { self.push(format!("down f{f} {x:.3},{y:.3}")); }
    fn touch_move(&mut self, f: u8, x: f32, y: f32) { self.push(format!("move f{f} {x:.3},{y:.3}")); }
    fn touch_up(&mut self, f: u8) { self.push(format!("up f{f}")); }
    fn nav(&mut self, k: NavKey) { self.push(format!("nav {k:?}")); }
    fn install_apk(&mut self, p: PathBuf) { self.push(format!("apk {}", p.display())); }
    fn restart(&mut self) { self.push("restart".into()); }
    fn set_repaint_hook(&mut self, h: Arc<dyn Fn() + Send + Sync>) { self.hook = Some(h); }
}

pub fn create_backend(settings: &Settings, config_override: Option<PathBuf>) -> Box<dyn EmulatorBackend> {
    if std::env::var("EXR_MOCK").as_deref() == Ok("1") {
        return Box::new(MockBackend::new());
    }
    if let Some((bin, cfg)) = ExrEngineBackend::find_paths(config_override) {
        return Box::new(ExrEngineBackend::start_with(bin, cfg, settings));
    }
    Box::new(MockBackend::new())
}
