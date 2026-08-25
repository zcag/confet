use std::path::PathBuf;
use std::process::{Command, Stdio};

// Sounds are embedded rather than installed alongside the binary so that
// `cargo install` / `cargo binstall` stay a single self-contained artifact.
// They are played by whatever player the platform already has, which keeps
// the build free of an audio library (and of ALSA headers on Linux).
const SOUNDS: &[(&str, &[u8])] = &[
    ("cork", include_bytes!("../assets/sounds/cork.wav")),
];

pub const SOUND_NAMES: &[&str] = &["cork"];

/// Players in preference order; each takes the file path as its last argument.
#[cfg(target_os = "macos")]
const PLAYERS: &[(&str, &[&str])] = &[("afplay", &[])];
#[cfg(not(target_os = "macos"))]
const PLAYERS: &[(&str, &[&str])] = &[
    ("pw-play", &[]),
    ("paplay", &[]),
    ("aplay", &["-q"]),
    ("ffplay", &["-nodisp", "-autoexit", "-loglevel", "quiet"]),
];

/// Fire and forget: spawn a player and let it outlive us if it needs to.
/// A missing player or a missing file is silence, not an error -- the sound
/// is a garnish on the animation, never the point.
pub fn play(spec: &str) {
    let Some(path) = resolve(spec) else { return };
    for (bin, args) in PLAYERS {
        let ok = Command::new(bin)
            .args(*args)
            .arg(&path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_ok();
        if ok { return }
    }
}

fn resolve(spec: &str) -> Option<PathBuf> {
    if let Some((name, bytes)) = SOUNDS.iter().find(|(n, _)| *n == spec) {
        return cached(name, bytes);
    }
    let path = match spec.strip_prefix("~/") {
        Some(rest) => dirs::home_dir()?.join(rest),
        None => PathBuf::from(spec),
    };
    if !path.exists() {
        // Worth a word: a typo'd name would otherwise be indistinguishable
        // from a working install that happens to be quiet.
        eprintln!("confet: no sound '{spec}' (built-in: {})", SOUND_NAMES.join(", "));
        return None;
    }
    Some(path)
}

/// The players all want a path, so an embedded sound is written out once and
/// replayed from the cache. The name carries a hash of the bytes, so a changed
/// asset can never be shadowed by a stale copy from an older build.
fn cached(name: &str, bytes: &[u8]) -> Option<PathBuf> {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    let dir = dirs::cache_dir()?.join("confet");
    let path = dir.join(format!("{name}-{hash:x}.wav"));
    if !path.exists() {
        std::fs::create_dir_all(&dir).ok()?;
        std::fs::write(&path, bytes).ok()?;
    }
    Some(path)
}
