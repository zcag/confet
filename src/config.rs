use crate::types::{AnimType, Shape, DEFAULT_COLORS, ANIM_TYPE_NAMES, BUILTIN_PROFILE_NAMES};
use clap::{CommandFactory, FromArgMatches, Parser};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

// ── Global settings ──────────────────────────────────────────────

static SETTINGS: OnceLock<Settings> = OnceLock::new();

pub fn settings() -> &'static Settings {
    SETTINGS.get().expect("settings not initialized")
}

pub fn set_settings(s: Settings) {
    let _ = SETTINGS.set(s);
}

// ── CLI ──────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "confet", about = "GPU-rendered confetti overlay for Wayland")]
pub struct Cli {
    /// Profile name or animation type
    pub profile: Option<String>,
    /// Animation type (confetti, cannon, pop, fireworks, snow, rain, sparkle, drop)
    #[arg(short = 't', long = "type")]
    pub anim_type: Option<String>,
    /// Particle shape (rect, circle, mixed)
    #[arg(short = 's', long)]
    pub shape: Option<String>,
    /// Number of particles
    #[arg(short = 'n', long)]
    pub particles: Option<usize>,
    /// Animation duration in seconds
    #[arg(short, long)]
    pub duration: Option<f64>,
    /// Gravity strength
    #[arg(short, long)]
    pub gravity: Option<f64>,
    /// Air resistance (0-1)
    #[arg(long)]
    pub drag: Option<f64>,
    /// Minimum launch speed
    #[arg(long)]
    pub speed_min: Option<f64>,
    /// Maximum launch speed
    #[arg(long)]
    pub speed_max: Option<f64>,
    /// Horizontal spread
    #[arg(long)]
    pub spread: Option<f64>,
    /// Fade-out duration in seconds
    #[arg(long)]
    pub fade: Option<f64>,
    /// Comma-separated hex colors (e.g. '#ff0000,#00ff00,#0000ff')
    #[arg(short, long, value_delimiter = ',')]
    pub colors: Option<Vec<String>>,
    /// Create default config file at ~/.config/confet/config.toml
    #[arg(long)]
    pub init: bool,
}

pub fn parse_cli(file: &FileConfig) -> Cli {
    let mut profiles: Vec<&str> = BUILTIN_PROFILE_NAMES.to_vec();
    for k in file.profiles.keys() {
        if !profiles.iter().any(|&p| p == k) {
            profiles.push(k);
        }
    }
    let help = format!(
        "Types: {}\nProfiles: {}",
        ANIM_TYPE_NAMES.join(", "),
        profiles.join(", "),
    );
    let matches = Cli::command()
        .after_help(help)
        .get_matches();
    Cli::from_arg_matches(&matches).unwrap()
}

// ── Config structs ───────────────────────────────────────────────

#[derive(Deserialize, Default, Clone)]
pub struct ProfileConfig {
    #[serde(rename = "type")]
    pub anim_type: Option<String>,
    pub shape: Option<String>,
    pub particles: Option<usize>,
    pub duration: Option<f64>,
    pub gravity: Option<f64>,
    pub drag: Option<f64>,
    pub speed_min: Option<f64>,
    pub speed_max: Option<f64>,
    pub spread: Option<f64>,
    pub fade: Option<f64>,
    pub colors: Option<Vec<String>>,
}

#[derive(Deserialize, Default)]
pub struct FileConfig {
    #[serde(rename = "type")]
    pub anim_type: Option<String>,
    pub shape: Option<String>,
    pub particles: Option<usize>,
    pub duration: Option<f64>,
    pub gravity: Option<f64>,
    pub drag: Option<f64>,
    pub speed_min: Option<f64>,
    pub speed_max: Option<f64>,
    pub spread: Option<f64>,
    pub fade: Option<f64>,
    pub colors: Option<Vec<String>>,
    #[serde(default)]
    pub profiles: HashMap<String, ProfileConfig>,
}

pub struct Settings {
    pub anim_type: AnimType,
    pub shape: Shape,
    pub particles: usize,
    pub duration: f64,
    pub gravity: f64,
    pub drag: f64,
    pub speed_min: f64,
    pub speed_max: f64,
    pub spread: f64,
    pub fade: f64,
    pub colors: Vec<[f32; 3]>,
}

// ── Color parsing ────────────────────────────────────────────────

fn parse_hex_color(s: &str) -> Option<[f32; 3]> {
    let s = s.strip_prefix('#').unwrap_or(s);
    if s.len() != 6 { return None; }
    let r = u8::from_str_radix(&s[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&s[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&s[4..6], 16).ok()? as f32 / 255.0;
    Some([r, g, b])
}

fn parse_colors(input: &[String]) -> Vec<[f32; 3]> {
    let parsed: Vec<_> = input.iter().filter_map(|s| parse_hex_color(s.trim())).collect();
    if parsed.is_empty() { DEFAULT_COLORS.to_vec() } else { parsed }
}

// ── Built-in profiles ────────────────────────────────────────────

fn builtin_profiles() -> HashMap<&'static str, ProfileConfig> {
    let mut m = HashMap::new();
    m.insert("lava", ProfileConfig {
        anim_type: Some("pop".into()),
        particles: Some(1000),
        duration: Some(2.0),
        gravity: Some(500.0),
        colors: Some(vec!["#ff2200".into(), "#ff6600".into(), "#ffaa00".into(), "#ffdd00".into(), "#ff4400".into()]),
        ..Default::default()
    });
    m.insert("matrix", ProfileConfig {
        anim_type: Some("rain".into()),
        particles: Some(600),
        duration: Some(5.0),
        colors: Some(vec!["#00ff00".into(), "#00cc00".into(), "#00ff44".into(), "#33ff33".into()]),
        ..Default::default()
    });
    m.insert("sakura", ProfileConfig {
        anim_type: Some("snow".into()),
        particles: Some(250),
        duration: Some(8.0),
        gravity: Some(30.0),
        colors: Some(vec!["#ffb7c5".into(), "#ff69b4".into(), "#ffc0cb".into(), "#ffffff".into(), "#ffd1dc".into()]),
        ..Default::default()
    });
    m.insert("aurora", ProfileConfig {
        anim_type: Some("sparkle".into()),
        particles: Some(150),
        duration: Some(5.0),
        colors: Some(vec!["#00ff87".into(), "#00d4ff".into(), "#a855f7".into(), "#ff2d87".into(), "#22d3ee".into()]),
        ..Default::default()
    });
    m.insert("gold", ProfileConfig {
        anim_type: Some("cannon".into()),
        shape: Some("circle".into()),
        particles: Some(1200),
        colors: Some(vec!["#ffd700".into(), "#ffb800".into(), "#fff1a8".into(), "#daa520".into(), "#ffe066".into()]),
        ..Default::default()
    });
    m.insert("balloon", ProfileConfig {
        anim_type: Some("drop".into()),
        particles: Some(1200),
        duration: Some(3.5),
        colors: Some(vec!["#ff2d87".into(), "#2d8cff".into(), "#2dff6d".into(), "#ffd02d".into(), "#a12dff".into(), "#ff6b2d".into()]),
        ..Default::default()
    });
    m
}

// ── Default config file ──────────────────────────────────────────

const DEFAULT_CONFIG: &str = r##"# Default settings
# type = "confetti"
# shape = "rect"
particles = 1500
duration = 2.5
gravity = 800
drag = 0.98
speed_min = 900
speed_max = 2500
spread = 150
fade = 0.4
colors = ["#ff2d87", "#2d8cff", "#2dff6d", "#ffd02d", "#a12dff", "#ff6b2d", "#2dfff6", "#ff2dca"]

# Available types: confetti, cannon, pop, fireworks, snow, rain, sparkle, drop
# Available shapes: rect, circle, mixed
# Run a profile: confet <name>

[profiles.lava]
type = "pop"
particles = 1000
duration = 2.0
gravity = 500
colors = ["#ff2200", "#ff6600", "#ffaa00", "#ffdd00", "#ff4400"]

[profiles.matrix]
type = "rain"
particles = 600
duration = 5.0
colors = ["#00ff00", "#00cc00", "#00ff44", "#33ff33"]

[profiles.sakura]
type = "snow"
particles = 250
duration = 8.0
gravity = 30
colors = ["#ffb7c5", "#ff69b4", "#ffc0cb", "#ffffff", "#ffd1dc"]

[profiles.aurora]
type = "sparkle"
particles = 150
duration = 5.0
colors = ["#00ff87", "#00d4ff", "#a855f7", "#ff2d87", "#22d3ee"]

[profiles.gold]
type = "cannon"
shape = "circle"
particles = 1200
colors = ["#ffd700", "#ffb800", "#fff1a8", "#daa520", "#ffe066"]

[profiles.balloon]
type = "drop"
particles = 1200
duration = 3.5
colors = ["#ff2d87", "#2d8cff", "#2dff6d", "#ffd02d", "#a12dff", "#ff6b2d"]
"##;

// ── File config ──────────────────────────────────────────────────

fn config_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join("confet").join("config.toml"))
}

pub fn init_config() {
    let Some(path) = config_path() else {
        eprintln!("could not determine config directory");
        std::process::exit(1);
    };
    if path.exists() {
        eprintln!("config already exists: {}", path.display());
        std::process::exit(1);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap_or_else(|e| {
            eprintln!("failed to create {}: {e}", parent.display());
            std::process::exit(1);
        });
    }
    std::fs::write(&path, DEFAULT_CONFIG).unwrap_or_else(|e| {
        eprintln!("failed to write {}: {e}", path.display());
        std::process::exit(1);
    });
    println!("created {}", path.display());
}

pub fn load_file_config() -> FileConfig {
    let Some(config_dir) = dirs::config_dir() else { return FileConfig::default() };
    let path = config_dir.join("confet").join("config.toml");
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

// ── Settings resolution ──────────────────────────────────────────

impl Settings {
    pub fn resolve(cli: Cli, file: FileConfig) -> Self {
        let builtins = builtin_profiles();

        let profile = if let Some(ref name) = cli.profile {
            if let Some(p) = file.profiles.get(name) {
                p.clone()
            } else if let Some(p) = builtins.get(name.as_str()) {
                p.clone()
            } else if AnimType::from_str(name).is_some() {
                ProfileConfig::default()
            } else {
                let mut profiles: Vec<&str> = BUILTIN_PROFILE_NAMES.to_vec();
                for k in file.profiles.keys() {
                    if !profiles.iter().any(|&p| p == k) {
                        profiles.push(k);
                    }
                }
                eprintln!("error: unknown name '{name}'\n");
                eprintln!("Types: {}", ANIM_TYPE_NAMES.join(", "));
                eprintln!("Profiles: {}", profiles.join(", "));
                std::process::exit(1);
            }
        } else {
            ProfileConfig::default()
        };

        let anim_type = cli.anim_type.as_deref()
            .and_then(AnimType::from_str)
            .or_else(|| profile.anim_type.as_deref().and_then(AnimType::from_str))
            .or_else(|| file.anim_type.as_deref().and_then(AnimType::from_str))
            .or_else(|| cli.profile.as_deref().and_then(AnimType::from_str))
            .unwrap_or_default();

        let shape = cli.shape.as_deref()
            .and_then(Shape::from_str)
            .or_else(|| profile.shape.as_deref().and_then(Shape::from_str))
            .or_else(|| file.shape.as_deref().and_then(Shape::from_str))
            .unwrap_or_else(|| anim_type.default_shape());

        let (dp, dd, dg, ddr, dsn, dsx, dsp, df) = anim_type.defaults();

        macro_rules! pick {
            ($cli:expr, $prof:expr, $file:expr, $default:expr) => {
                $cli.unwrap_or_else(|| $prof.unwrap_or_else(|| $file.unwrap_or($default)))
            };
        }

        let colors = if let Some(ref c) = cli.colors {
            parse_colors(c)
        } else if let Some(ref c) = profile.colors {
            parse_colors(c)
        } else if let Some(ref c) = file.colors {
            parse_colors(c)
        } else {
            anim_type.default_colors().to_vec()
        };

        Self {
            anim_type, shape,
            particles: pick!(cli.particles, profile.particles, file.particles, dp),
            duration:  pick!(cli.duration,  profile.duration,  file.duration,  dd),
            gravity:   pick!(cli.gravity,   profile.gravity,   file.gravity,   dg),
            drag:      pick!(cli.drag,      profile.drag,      file.drag,      ddr),
            speed_min: pick!(cli.speed_min, profile.speed_min, file.speed_min, dsn),
            speed_max: pick!(cli.speed_max, profile.speed_max, file.speed_max, dsx),
            spread:    pick!(cli.spread,    profile.spread,    file.spread,    dsp),
            fade:      pick!(cli.fade,      profile.fade,      file.fade,      df),
            colors,
        }
    }
}
