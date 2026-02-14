use crate::config::settings;
use crate::types::{AnimType, Shape};
use gtk4::prelude::*;
use gtk4::{gdk, gsk};
use rand::Rng;

fn randn(rng: &mut impl Rng, mean: f64, std: f64) -> f64 {
    let u1: f64 = rng.gen_range(1e-10..1.0);
    let u2: f64 = rng.gen();
    mean + std * (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

pub struct Particles {
    x: Vec<f64>, y: Vec<f64>, vx: Vec<f64>, vy: Vec<f64>,
    delay: Vec<f32>, color: Vec<u8>,
    pw: Vec<f32>, ph: Vec<f32>,
    rot: Vec<f64>, rot_speed: Vec<f64>,
    wobble: Vec<f64>, wobble_speed: Vec<f64>,
    is_circle: Vec<bool>,
}

impl Particles {
    pub fn new(w: f64, h: f64) -> Self {
        let s = settings();
        let n = s.particles;
        let mut r = rand::thread_rng();
        let pi = std::f64::consts::PI;
        let tau = std::f64::consts::TAU;
        let (mut x, mut y, mut vx, mut vy) =
            (Vec::with_capacity(n), Vec::with_capacity(n),
             Vec::with_capacity(n), Vec::with_capacity(n));

        match s.anim_type {
            AnimType::Confetti => {
                let half = n / 2;
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
            }
            AnimType::Cannon => {
                for _ in 0..n {
                    x.push(randn(&mut r, w / 2.0, 20.0));
                    y.push(randn(&mut r, h, 10.0));
                    let ang = r.gen_range(-pi * 0.85..-pi * 0.15);
                    let spd = r.gen_range(s.speed_min..s.speed_max);
                    vx.push(ang.cos() * spd + r.gen_range(-s.spread..s.spread));
                    vy.push(ang.sin() * spd);
                }
            }
            AnimType::Pop => {
                for _ in 0..n {
                    x.push(w / 2.0);
                    y.push(h / 2.0);
                    let ang = r.gen_range(0.0..tau);
                    let spd = r.gen_range(s.speed_min..s.speed_max);
                    vx.push(ang.cos() * spd);
                    vy.push(ang.sin() * spd);
                }
            }
            AnimType::Fireworks => {
                let cx = w / 2.0 + r.gen_range(-w * 0.15..w * 0.15);
                let cy = h * 0.3 + r.gen_range(-h * 0.1..h * 0.1);
                for _ in 0..n {
                    x.push(cx + r.gen_range(-5.0..5.0));
                    y.push(cy + r.gen_range(-5.0..5.0));
                    let ang = r.gen_range(0.0..tau);
                    let spd = r.gen_range(s.speed_min..s.speed_max);
                    vx.push(ang.cos() * spd);
                    vy.push(ang.sin() * spd);
                }
            }
            AnimType::Snow => {
                for _ in 0..n {
                    x.push(r.gen_range(0.0..w));
                    y.push(r.gen_range(-h * 0.3..h * 0.1));
                    vx.push(r.gen_range(-s.spread..s.spread));
                    vy.push(r.gen_range(s.speed_min..s.speed_max));
                }
            }
            AnimType::Rain => {
                for _ in 0..n {
                    x.push(r.gen_range(0.0..w));
                    y.push(r.gen_range(-h..0.0));
                    vx.push(r.gen_range(-s.spread..s.spread));
                    vy.push(r.gen_range(s.speed_min..s.speed_max));
                }
            }
            AnimType::Sparkle => {
                for _ in 0..n {
                    x.push(r.gen_range(0.0..w));
                    y.push(r.gen_range(0.0..h));
                    vx.push(0.0);
                    vy.push(0.0);
                }
            }
            AnimType::Drop => {
                for _ in 0..n {
                    x.push(randn(&mut r, w / 2.0, s.spread));
                    y.push(r.gen_range(-80.0..20.0));
                    vx.push(r.gen_range(-s.spread * 0.3..s.spread * 0.3));
                    vy.push(r.gen_range(s.speed_min..s.speed_max));
                }
            }
        }

        let nc = s.colors.len();
        let (pw_lo, pw_hi, ph_lo, ph_hi): (f32, f32, f32, f32) = match s.anim_type {
            AnimType::Fireworks => (3.0, 6.0, 3.0, 6.0),
            AnimType::Snow      => (4.0, 8.0, 4.0, 8.0),
            AnimType::Rain      => (1.5, 3.0, 15.0, 30.0),
            AnimType::Sparkle   => (2.0, 5.0, 2.0, 5.0),
            _                   => (5.0, 12.0, 3.0, 8.0),
        };
        let (rot_lo, rot_hi): (f64, f64) = match s.anim_type {
            AnimType::Rain | AnimType::Sparkle => (0.0, 0.0),
            AnimType::Snow => (-3.0, 3.0),
            _ => (-12.0, 12.0),
        };
        let delay_max: f32 = match s.anim_type {
            AnimType::Snow | AnimType::Rain => 1.5,
            AnimType::Sparkle => s.duration as f32 * 0.6,
            AnimType::Drop => 0.5,
            _ => 0.3,
        };

        let is_circle: Vec<bool> = match s.shape {
            Shape::Rect => vec![false; n],
            Shape::Circle => vec![true; n],
            Shape::Mixed => (0..n).map(|_| r.gen_bool(0.5)).collect(),
        };

        Self {
            x, y, vx, vy,
            delay: (0..n).map(|_| r.gen_range(0.0..delay_max)).collect(),
            color: (0..n).map(|_| r.gen_range(0..nc as u8)).collect(),
            pw: (0..n).map(|_| r.gen_range(pw_lo..pw_hi)).collect(),
            ph: (0..n).map(|_| r.gen_range(ph_lo..ph_hi)).collect(),
            rot: (0..n).map(|_| r.gen_range(0.0..tau)).collect(),
            rot_speed: (0..n).map(|_| if rot_lo == rot_hi { rot_lo } else { r.gen_range(rot_lo..rot_hi) }).collect(),
            wobble: (0..n).map(|_| r.gen_range(0.0..tau)).collect(),
            wobble_speed: (0..n).map(|_| r.gen_range(3.0..8.0)).collect(),
            is_circle,
        }
    }

    pub fn step(&mut self, dt: f64, t: f64) {
        let s = settings();
        let n = s.particles;
        let tf = t as f32;
        for i in 0..n {
            if tf < self.delay[i] { continue; }
            match s.anim_type {
                AnimType::Sparkle => {
                    self.wobble[i] += self.wobble_speed[i] * dt;
                }
                AnimType::Snow => {
                    self.x[i] += self.wobble[i].sin() * 30.0 * dt;
                    self.wobble[i] += self.wobble_speed[i] * dt;
                    self.vy[i] += s.gravity * dt;
                    self.y[i] += self.vy[i] * dt;
                    self.rot[i] += self.rot_speed[i] * dt;
                }
                _ => {
                    self.vy[i] += s.gravity * dt;
                    self.vx[i] *= s.drag;
                    self.x[i] += self.vx[i] * dt;
                    self.y[i] += self.vy[i] * dt;
                    self.rot[i] += self.rot_speed[i] * dt;
                    self.wobble[i] += self.wobble_speed[i] * dt;
                }
            }
        }
    }

    pub fn draw(&self, snap: &gtk4::Snapshot, alpha: f32, t: f64) {
        let s = settings();
        let n = s.particles;
        let tf = t as f32;

        if s.anim_type == AnimType::Sparkle {
            for i in 0..n {
                if tf < self.delay[i] { continue; }
                let phase = (self.wobble[i].sin() * 0.5 + 0.5) as f32;
                let pa = alpha * phase;
                if pa < 0.01 { continue; }
                let c = &s.colors[self.color[i] as usize];
                let rgba = gdk::RGBA::new(c[0], c[1], c[2], pa);
                let sz = self.pw[i];
                let half = sz / 2.0;
                let rect = graphene::Rect::new(-half, -half, sz, sz);
                snap.save();
                snap.translate(&graphene::Point::new(self.x[i] as f32, self.y[i] as f32));
                if self.is_circle[i] {
                    let corner = graphene::Size::new(half, half);
                    snap.push_rounded_clip(&gsk::RoundedRect::new(rect, corner, corner, corner, corner));
                    snap.append_color(&rgba, &rect);
                    snap.pop();
                } else {
                    snap.append_color(&rgba, &rect);
                }
                snap.restore();
            }
            return;
        }

        let rgba: Vec<gdk::RGBA> = s.colors.iter()
            .map(|c| gdk::RGBA::new(c[0], c[1], c[2], alpha))
            .collect();
        let use_wobble = !matches!(s.anim_type, AnimType::Rain);
        for i in 0..n {
            if tf < self.delay[i] { continue; }
            let color = &rgba[self.color[i] as usize];
            snap.save();
            snap.translate(&graphene::Point::new(self.x[i] as f32, self.y[i] as f32));
            snap.rotate(self.rot[i].to_degrees() as f32);
            if self.is_circle[i] {
                let sz = (self.pw[i] + self.ph[i]) / 2.0;
                let half = sz / 2.0;
                let rect = graphene::Rect::new(-half, -half, sz, sz);
                let corner = graphene::Size::new(half, half);
                snap.push_rounded_clip(&gsk::RoundedRect::new(rect, corner, corner, corner, corner));
                snap.append_color(color, &rect);
                snap.pop();
            } else {
                let sw = if use_wobble {
                    self.wobble[i].sin().abs().max(0.15) as f32 * self.pw[i]
                } else {
                    self.pw[i]
                };
                let sh = self.ph[i];
                snap.append_color(color, &graphene::Rect::new(-sw / 2.0, -sh / 2.0, sw, sh));
            }
            snap.restore();
        }
    }
}
