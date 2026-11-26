//! ExR macro grammar (our own). Spec: docs/05 §7. Line-based, `#` comments.
#[derive(Debug, PartialEq, Clone)]
pub enum Op {
    Touch { x: f32, y: f32, hold_ms: u32 }, Swipe { x1: f32, y1: f32, x2: f32, y2: f32, ms: u32 }, Delay(u32),
    KeyDown(String), KeyUp(String), KeyPress(String), KeyMove { dx: f32, dy: f32 }, Input(String), MapKey(String),
    LoopStart(Option<u32>), LoopStop, OnKeyDown, OnKeyUp, EnterShoot, ExitShoot,
}
#[derive(Debug, PartialEq)] pub struct LineError { pub line: usize, pub msg: String }

pub fn parse(src: &str) -> Result<Vec<Op>, Vec<LineError>> {
    let (mut ops, mut errs) = (Vec::new(), Vec::new());
    for (i, raw) in src.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() { continue; }
        let mut it = line.splitn(2, char::is_whitespace);
        let cmd = it.next().unwrap(); let rest = it.next().unwrap_or("").trim();
        let nums = || -> Result<Vec<f32>, String> { rest.split_whitespace().map(|n| n.parse::<f32>().map_err(|_| format!("'{n}' is not a number"))).collect() };
        let r: Result<Op, String> = match cmd {
            "touch" => nums().and_then(|n| match n.as_slice() { [x, y] => Ok(Op::Touch { x: *x, y: *y, hold_ms: 30 }), [x, y, h] => Ok(Op::Touch { x: *x, y: *y, hold_ms: *h as u32 }), _ => Err("touch X Y [hold_ms]".into()) }),
            "swipe" => nums().and_then(|n| match n.as_slice() { [a, b, c, d] => Ok(Op::Swipe { x1: *a, y1: *b, x2: *c, y2: *d, ms: 180 }), [a, b, c, d, m] => Ok(Op::Swipe { x1: *a, y1: *b, x2: *c, y2: *d, ms: *m as u32 }), _ => Err("swipe X1 Y1 X2 Y2 [ms]".into()) }),
            "delay" => rest.parse::<u32>().map(Op::Delay).map_err(|_| "delay MS".into()),
            "keydown" if !rest.is_empty() => Ok(Op::KeyDown(rest.into())), "keyup" if !rest.is_empty() => Ok(Op::KeyUp(rest.into())),
            "keypress" if !rest.is_empty() => Ok(Op::KeyPress(rest.into())), "mapkey" if !rest.is_empty() => Ok(Op::MapKey(rest.into())),
            "keymove" => nums().and_then(|n| match n.as_slice() { [x, y] => Ok(Op::KeyMove { dx: *x, dy: *y }), _ => Err("keymove DX DY".into()) }),
            "input" => Ok(Op::Input(rest.trim_matches('"').into())),
            "loop" => if rest.is_empty() { Ok(Op::LoopStart(None)) } else { rest.parse().map(|n| Op::LoopStart(Some(n))).map_err(|_| "loop [N]".into()) },
            "loopstop" => Ok(Op::LoopStop), "onkeydown" => Ok(Op::OnKeyDown), "onkeyup" => Ok(Op::OnKeyUp),
            "entershoot" => Ok(Op::EnterShoot), "exitshoot" => Ok(Op::ExitShoot),
            other => Err(format!("unknown command '{other}'")),
        };
        match r { Ok(o) => ops.push(o), Err(m) => errs.push(LineError { line: i + 1, msg: m }) }
    }
    if errs.is_empty() { Ok(ops) } else { Err(errs) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn ok() { assert_eq!(parse("touch 0.9 0.4\n# c\ndelay 100").unwrap().len(), 2); }
    #[test] fn err_line() { let e = parse("delay 5\nfoo 1").unwrap_err(); assert_eq!(e[0].line, 2); }
}
