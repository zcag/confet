use clap::Parser;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{gdk, glib};
use gtk4_layer_shell::LayerShell;
use graphene;
use rand::Rng;
use serde::Deserialize;
use std::cell::RefCell;
use std::sync::OnceLock;
use std::time::Instant;

// ── Configuration ─────────────────────────────────────────────────

const DEFAULT_COLORS: [[f32; 3]; 9] = [
    [0.95, 0.26, 0.26], [0.26, 0.80, 0.36], [0.20, 0.45, 1.00],
    [1.00, 0.82, 0.10], [1.00, 0.45, 0.10], [0.75, 0.25, 1.00],
    [0.10, 0.82, 0.82], [1.00, 0.42, 0.70], [0.40, 1.00, 0.40],
];

static SETTINGS: OnceLock<Settings> = OnceLock::new();

fn settings() -> &'static Settings {
    SETTINGS.get().expect("settings not initialized")
}

#[derive(Parser)]
#[command(name = "confet", about = "GPU-rendered confetti overlay for Wayland")]
struct Cli {
    /// Number of particles
    #[arg(short = 'n', long)]
    particles: Option<usize>,
    /// Animation duration in seconds
    #[arg(short, long)]
    duration: Option<f64>,
    /// Gravity strength
    #[arg(short, long)]
    gravity: Option<f64>,
    /// Air resistance (0-1)
    #[arg(long)]
    drag: Option<f64>,
    /// Minimum launch speed
    #[arg(long)]
    speed_min: Option<f64>,
    /// Maximum launch speed
    #[arg(long)]
    speed_max: Option<f64>,
    /// Horizontal spread
    #[arg(long)]
    spread: Option<f64>,
    /// Fade-out duration in seconds
    #[arg(long)]
    fade: Option<f64>,
    /// Comma-separated hex colors (e.g. '#ff0000,#00ff00,#0000ff')
    #[arg(short, long, value_delimiter = ',')]
    colors: Option<Vec<String>>,
    /// Create default config file at ~/.config/confet/config.toml
    #[arg(long)]
    init: bool,
}

#[derive(Deserialize, Default)]
struct FileConfig {
    particles: Option<usize>,
    duration: Option<f64>,
    gravity: Option<f64>,
    drag: Option<f64>,
    speed_min: Option<f64>,
    speed_max: Option<f64>,
    spread: Option<f64>,
    fade: Option<f64>,
    colors: Option<Vec<String>>,
}

struct Settings {
    particles: usize,
    duration: f64,
    gravity: f64,
    drag: f64,
    speed_min: f64,
    speed_max: f64,
    spread: f64,
    fade: f64,
    colors: Vec<[f32; 3]>,
}

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

const DEFAULT_CONFIG: &str = r##"particles = 1200
duration = 3.5
gravity = 1000
drag = 0.985
speed_min = 1200
speed_max = 3000
spread = 200
fade = 0.6
colors = ["#f24242", "#42cc5c", "#3373ff", "#ffd11a", "#ff731a", "#bf40ff", "#1ad1d1", "#ff6bb3", "#66ff66"]
"##;

fn config_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join("confet").join("config.toml"))
}

fn init_config() {
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

fn load_file_config() -> FileConfig {
    let Some(config_dir) = dirs::config_dir() else { return FileConfig::default() };
    let path = config_dir.join("confet").join("config.toml");
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

impl Settings {
    fn resolve(cli: Cli, file: FileConfig) -> Self {
        macro_rules! pick {
            ($cli:expr, $file:expr, $default:expr) => {
                $cli.unwrap_or_else(|| $file.unwrap_or($default))
            };
        }
        let colors = if let Some(ref c) = cli.colors {
            parse_colors(c)
        } else if let Some(ref c) = file.colors {
            parse_colors(c)
        } else {
            DEFAULT_COLORS.to_vec()
        };
        Self {
            particles: pick!(cli.particles, file.particles, 1200),
            duration:  pick!(cli.duration,  file.duration,  3.5),
            gravity:   pick!(cli.gravity,   file.gravity,   1000.0),
            drag:      pick!(cli.drag,      file.drag,      0.985),
            speed_min: pick!(cli.speed_min, file.speed_min, 1200.0),
            speed_max: pick!(cli.speed_max, file.speed_max, 3000.0),
            spread:    pick!(cli.spread,    file.spread,    200.0),
            fade:      pick!(cli.fade,      file.fade,      0.6),
            colors,
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────

fn randn(rng: &mut impl Rng, mean: f64, std: f64) -> f64 {
    let u1: f64 = rng.gen_range(1e-10..1.0);
    let u2: f64 = rng.gen();
    mean + std * (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

// ── Particles ──────────────────────────────────────────────────────

struct Particles {
    x: Vec<f64>, y: Vec<f64>, vx: Vec<f64>, vy: Vec<f64>,
    delay: Vec<f32>, color: Vec<u8>,
    pw: Vec<f32>, ph: Vec<f32>,
    rot: Vec<f64>, rot_speed: Vec<f64>,
    wobble: Vec<f64>, wobble_speed: Vec<f64>,
}

impl Particles {
    fn new(w: f64, h: f64) -> Self {
        let s = settings();
        let n = s.particles;
        let mut r = rand::thread_rng();
        let half = n / 2;
        let pi = std::f64::consts::PI;
        let (mut x, mut y, mut vx, mut vy) =
            (Vec::with_capacity(n), Vec::with_capacity(n),
             Vec::with_capacity(n), Vec::with_capacity(n));
        for i in 0..n {
            let (cx, ang) = if i < half {
                (randn(&mut r, 0.0, 20.0), r.gen_range(-pi * 0.9..-pi * 0.1))
            } else {
                (randn(&mut r, w, 20.0), pi - r.gen_range(-pi * 0.9..-pi * 0.1))
            };
            x.push(cx);
            y.push(randn(&mut r, h, 10.0));
            let spd = r.gen_range(s.speed_min..s.speed_max);
            vx.push(ang.cos() * spd + r.gen_range(-s.spread..s.spread));
            vy.push(ang.sin() * spd);
        }
        let nc = s.colors.len();
        Self {
            x, y, vx, vy,
            delay: (0..n).map(|_| r.gen_range(0.0..0.3f32)).collect(),
            color: (0..n).map(|_| r.gen_range(0..nc as u8)).collect(),
            pw: (0..n).map(|_| r.gen_range(5.0..12.0f32)).collect(),
            ph: (0..n).map(|_| r.gen_range(3.0..8.0f32)).collect(),
            rot: (0..n).map(|_| r.gen_range(0.0..std::f64::consts::TAU)).collect(),
            rot_speed: (0..n).map(|_| r.gen_range(-12.0..12.0)).collect(),
            wobble: (0..n).map(|_| r.gen_range(0.0..std::f64::consts::TAU)).collect(),
            wobble_speed: (0..n).map(|_| r.gen_range(3.0..8.0)).collect(),
        }
    }

    fn step(&mut self, dt: f64, t: f64) {
        let s = settings();
        let n = s.particles;
        let tf = t as f32;
        for i in 0..n {
            if tf < self.delay[i] { continue; }
            self.vy[i] += s.gravity * dt;
            self.vx[i] *= s.drag;
            self.x[i] += self.vx[i] * dt;
            self.y[i] += self.vy[i] * dt;
            self.rot[i] += self.rot_speed[i] * dt;
            self.wobble[i] += self.wobble_speed[i] * dt;
        }
    }

    fn draw(&self, snap: &gtk4::Snapshot, alpha: f32, t: f64) {
        let s = settings();
        let n = s.particles;
        let rgba: Vec<gdk::RGBA> = s.colors.iter()
            .map(|c| gdk::RGBA::new(c[0], c[1], c[2], alpha))
            .collect();
        let tf = t as f32;
        for i in 0..n {
            if tf < self.delay[i] { continue; }
            let sw = self.wobble[i].sin().abs().max(0.15) as f32 * self.pw[i];
            let sh = self.ph[i];
            snap.save();
            snap.translate(&graphene::Point::new(self.x[i] as f32, self.y[i] as f32));
            snap.rotate(self.rot[i].to_degrees() as f32);
            snap.append_color(
                &rgba[self.color[i] as usize],
                &graphene::Rect::new(-sw / 2.0, -sh / 2.0, sw, sh),
            );
            snap.restore();
        }
    }
}

// ── Canvas widget ──────────────────────────────────────────────────

mod imp {
    use super::*;

    pub struct State {
        pub(crate) ps: Particles,
        pub t0: Instant,
        pub last: Instant,
    }

    #[derive(Default)]
    pub struct Canvas {
        pub state: RefCell<Option<State>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Canvas {
        const NAME: &'static str = "ConfettiCanvas";
        type Type = super::Canvas;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for Canvas {}

    impl WidgetImpl for Canvas {
        fn snapshot(&self, snap: &gtk4::Snapshot) {
            let s = settings();
            let st = self.state.borrow();
            let Some(state) = st.as_ref() else { return };
            let t = state.t0.elapsed().as_secs_f64();
            let alpha = if t > s.duration - s.fade {
                ((s.duration - t) / s.fade).clamp(0.0, 1.0) as f32
            } else {
                1.0
            };
            state.ps.draw(snap, alpha, t);
        }
    }
}

glib::wrapper! {
    pub struct Canvas(ObjectSubclass<imp::Canvas>)
        @extends gtk4::Widget;
}

impl Canvas {
    fn new() -> Self { glib::Object::builder().build() }

    fn start(&self, w: f64, h: f64) {
        let now = Instant::now();
        *self.imp().state.borrow_mut() = Some(imp::State {
            ps: Particles::new(w, h), t0: now, last: now,
        });
        self.add_tick_callback(|widget, _| {
            let s = settings();
            let canvas: &Canvas = widget.downcast_ref().unwrap();
            let mut st = canvas.imp().state.borrow_mut();
            let Some(state) = st.as_mut() else { return glib::ControlFlow::Break };
            let now = Instant::now();
            let t = (now - state.t0).as_secs_f64();
            if t > s.duration {
                drop(st);
                if let Some(app) = canvas.root()
                    .and_then(|r| r.downcast::<gtk4::Window>().ok())
                    .and_then(|w| w.application())
                {
                    app.quit();
                }
                return glib::ControlFlow::Break;
            }
            let dt = (now - state.last).as_secs_f64();
            state.last = now;
            state.ps.step(dt, t);
            drop(st);
            canvas.queue_draw();
            glib::ControlFlow::Continue
        });
    }
}

// ── Main ───────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();
    if cli.init {
        init_config();
        return;
    }
    let file = load_file_config();
    let _ = SETTINGS.set(Settings::resolve(cli, file));

    let app = gtk4::Application::builder()
        .application_id("dev.confetti.overlay")
        .build();

    app.connect_activate(|app| {
        let win = gtk4::Window::builder().application(app).build();

        win.init_layer_shell();
        win.set_layer(gtk4_layer_shell::Layer::Overlay);
        for edge in [
            gtk4_layer_shell::Edge::Top, gtk4_layer_shell::Edge::Bottom,
            gtk4_layer_shell::Edge::Left, gtk4_layer_shell::Edge::Right,
        ] {
            win.set_anchor(edge, true);
        }
        win.set_exclusive_zone(-1);
        win.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);
        win.set_namespace(Some("confetti"));

        let css = gtk4::CssProvider::new();
        css.load_from_data(
            "window.background, window.background * { background: unset; background-color: rgba(0,0,0,0); }",
        );
        gtk4::style_context_add_provider_for_display(
            &gdk::Display::default().unwrap(), &css, gtk4::STYLE_PROVIDER_PRIORITY_USER,
        );

        let canvas = Canvas::new();
        canvas.set_hexpand(true);
        canvas.set_vexpand(true);
        win.set_child(Some(&canvas));

        win.connect_realize(|w| {
            if let Some(surface) = w.surface() {
                let region = cairo::Region::create_rectangle(
                    &cairo::RectangleInt::new(0, 0, 0, 0),
                );
                surface.set_input_region(&region);
            }
        });

        win.present();

        let display = gdk::Display::default().unwrap();
        let monitor: gdk::Monitor = display.monitors()
            .item(0).unwrap().downcast().unwrap();
        let geom = monitor.geometry();
        canvas.start(geom.width() as f64, geom.height() as f64);
    });

    app.run_with_args::<&str>(&[]);
}
