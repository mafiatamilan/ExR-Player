//! Pure validation. Spec: docs/04 §3.3–3.4, §5.

/// MEmu 9.2.5 rule: only F1–F12 or combos containing Ctrl/Shift/Alt.
pub fn shortcut_ok(s: &str) -> bool {
    let parts: Vec<&str> = s.split('+').map(str::trim).collect();
    if parts.is_empty() || parts.iter().any(|p| p.is_empty()) { return false; }
    let is_fn = |k: &str| k.strip_prefix('F').and_then(|n| n.parse::<u8>().ok()).map_or(false, |n| (1..=12).contains(&n));
    if parts.len() == 1 { return is_fn(parts[0]); }
    parts[..parts.len() - 1].iter().any(|m| matches!(*m, "Ctrl" | "Shift" | "Alt"))
}

pub fn imei_ok(s: &str) -> bool {
    if s.len() != 15 || !s.bytes().all(|b| b.is_ascii_digit()) { return false; }
    let sum: u32 = s.bytes().rev().enumerate().map(|(i, b)| { let d = (b - b'0') as u32; if i % 2 == 1 { let x = d * 2; x / 10 + x % 10 } else { d } }).sum();
    sum % 10 == 0
}

pub fn resolution_ok(w: u32, h: u32, dpi: u32) -> bool { (320..=3840).contains(&w) && (240..=2160).contains(&h) && (120..=640).contains(&dpi) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn shortcuts() { assert!(shortcut_ok("F5")); assert!(shortcut_ok("Ctrl+Shift+S")); assert!(!shortcut_ok("S")); assert!(!shortcut_ok("F13")); assert!(!shortcut_ok("Ctrl+")); }
    #[test] fn imei() { assert!(imei_ok("490154203237518")); assert!(!imei_ok("490154203237519")); assert!(!imei_ok("123")); }
    #[test] fn res() { assert!(resolution_ok(1280, 720, 240)); assert!(!resolution_ok(100, 720, 240)); }
}
