#[derive(Clone, Copy, Default, PartialEq)]
pub enum AnimType {
    #[default]
    Confetti,
    Cannon,
    Pop,
    Fireworks,
    Snow,
    Rain,
    Sparkle,
    Drop,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum Shape {
    #[default]
    Rect,
    Circle,
    Mixed,
}

pub const DEFAULT_COLORS: [[f32; 3]; 9] = [
    [0.95, 0.26, 0.26], [0.26, 0.80, 0.36], [0.20, 0.45, 1.00],
    [1.00, 0.82, 0.10], [1.00, 0.45, 0.10], [0.75, 0.25, 1.00],
    [0.10, 0.82, 0.82], [1.00, 0.42, 0.70], [0.40, 1.00, 0.40],
];

pub const ANIM_TYPE_NAMES: &[&str] = &[
    "confetti", "cannon", "pop", "fireworks", "snow", "rain", "sparkle", "drop",
];

pub const BUILTIN_PROFILE_NAMES: &[&str] = &[
    "lava", "matrix", "sakura", "aurora", "gold", "balloon",
];

impl AnimType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "confetti" => Some(Self::Confetti),
            "cannon" => Some(Self::Cannon),
            "pop" => Some(Self::Pop),
            "fireworks" => Some(Self::Fireworks),
            "snow" => Some(Self::Snow),
            "rain" => Some(Self::Rain),
            "sparkle" => Some(Self::Sparkle),
            "drop" => Some(Self::Drop),
            _ => None,
        }
    }

    /// Returns (particles, duration, gravity, drag, speed_min, speed_max, spread, fade)
    pub fn defaults(self) -> (usize, f64, f64, f64, f64, f64, f64, f64) {
        match self {
            Self::Confetti => (1500, 2.5, 800.0, 0.98, 900.0, 2500.0, 150.0, 0.4),
            Self::Cannon   => (1500, 2.5, 800.0, 0.98, 900.0, 2500.0, 100.0, 0.4),
            Self::Pop      => (1000, 2.0, 300.0, 0.98, 600.0, 1500.0, 0.0, 0.4),
            Self::Fireworks=> (600, 3.0, 400.0, 0.99, 400.0, 1200.0, 0.0, 0.5),
            Self::Snow     => (200, 8.0, 40.0, 0.995, 20.0, 80.0, 30.0, 1.0),
            Self::Rain     => (800, 4.0, 200.0, 0.999, 1500.0, 3000.0, 20.0, 0.3),
            Self::Sparkle  => (100, 4.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.5),
            Self::Drop     => (1500, 3.0, 600.0, 0.99, 50.0, 200.0, 400.0, 0.5),
        }
    }

    pub fn default_shape(self) -> Shape {
        match self {
            Self::Fireworks | Self::Snow | Self::Sparkle => Shape::Circle,
            Self::Drop | Self::Pop => Shape::Mixed,
            Self::Rain => Shape::Rect,
            _ => Shape::Rect,
        }
    }

    pub fn default_colors(self) -> &'static [[f32; 3]] {
        match self {
            Self::Snow => &[
                [1.0, 1.0, 1.0], [0.88, 0.91, 1.0], [0.75, 0.83, 1.0], [0.82, 0.88, 1.0],
            ],
            Self::Rain => &[
                [0.27, 0.53, 0.80], [0.20, 0.40, 0.67], [0.33, 0.60, 0.87], [0.40, 0.53, 0.67],
            ],
            Self::Sparkle => &[
                [1.0, 1.0, 1.0], [1.0, 1.0, 0.82], [1.0, 0.84, 0.0], [1.0, 0.97, 0.86],
            ],
            Self::Fireworks => &[
                [1.0, 0.27, 0.27], [1.0, 0.67, 0.0], [1.0, 1.0, 0.27], [1.0, 1.0, 1.0], [1.0, 0.42, 0.18],
            ],
            _ => &DEFAULT_COLORS,
        }
    }
}

impl Shape {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "rect" => Some(Self::Rect),
            "circle" => Some(Self::Circle),
            "mixed" => Some(Self::Mixed),
            _ => None,
        }
    }
}
