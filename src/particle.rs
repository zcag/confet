use crate::config::settings;
use crate::types::{AnimType, Shape};
use gtk4::prelude::*;
use gtk4::{gdk, glib, gsk};
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
    kind: Vec<Shape>,
    /// One antialiased triangle per colour. GSK has no polygon primitive
    /// before GTK 4.14's Path API, and append_cairo silently blanks the whole
    /// frame on macOS -- a texture per colour draws in one node per particle,
    /// on the GPU, on every GTK 4 version.
    tri_tex: Vec<gdk::Texture>,
    w: f64, h: f64,
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
        let z = s.size as f32;
        let (pw_lo, pw_hi, ph_lo, ph_hi) = (pw_lo * z, pw_hi * z, ph_lo * z, ph_hi * z);
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

        // Mixed is roughly half strips, the rest split between discs and
        // triangles -- thrown paper is not all one shape.
        let kind: Vec<Shape> = match s.shape {
            Shape::Mixed => (0..n).map(|_| match r.gen_range(0..4) {
                0 => Shape::Circle,
                1 => Shape::Triangle,
                _ => Shape::Rect,
            }).collect(),
            fixed => vec![fixed; n],
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
            tri_tex: if kind.contains(&Shape::Triangle) {
                triangle_textures(&s.colors)
            } else {
                Vec::new()
            },
            kind, w, h,
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

        // GSK has no polygon primitive before GTK 4.14's Path API, and
        // requiring that would drop every distro still on GTK 4.6-4.8. So
        // triangles are collected here and filled in a single cairo node at
        // the end -- one extra node per frame rather than one per particle.
        // (x, y, rotation, half width, half height, rgb, alpha)
        let mut tris: Vec<(f64, f64, f64, f64, f64, usize, f32)> = Vec::new();

        if s.anim_type == AnimType::Sparkle {
            for i in 0..n {
                if tf < self.delay[i] { continue; }
                let phase = (self.wobble[i].sin() * 0.5 + 0.5) as f32;
                let pa = alpha * phase;
                if pa < 0.01 { continue; }
                let c = &s.colors[self.color[i] as usize];
                let sz = self.pw[i];
                let half = sz / 2.0;
                if self.kind[i] == Shape::Triangle {
                    tris.push((self.x[i], self.y[i], 0.0,
                               half as f64, half as f64, self.color[i] as usize, pa));
                    continue;
                }
                let rgba = gdk::RGBA::new(c[0], c[1], c[2], pa);
                let rect = graphene::Rect::new(-half, -half, sz, sz);
                snap.save();
                snap.translate(&graphene::Point::new(self.x[i] as f32, self.y[i] as f32));
                if self.kind[i] == Shape::Circle {
                    let corner = graphene::Size::new(half, half);
                    snap.push_rounded_clip(&gsk::RoundedRect::new(rect, corner, corner, corner, corner));
                    snap.append_color(&rgba, &rect);
                    snap.pop();
                } else {
                    snap.append_color(&rgba, &rect);
                }
                snap.restore();
            }
            self.fill_triangles(snap, &tris, None);
            return;
        }

        let rgba: Vec<gdk::RGBA> = s.colors.iter()
            .map(|c| gdk::RGBA::new(c[0], c[1], c[2], alpha))
            .collect();
        let use_wobble = !matches!(s.anim_type, AnimType::Rain);
        // A tumbling paper disc reads as an ellipse, not a circle -- and being
        // asymmetric is also the only way its rotation is visible at all.
        // Snow, fireworks and sparkle are flakes and sparks rather than paper,
        // so their discs stay round.
        let discs_tumble = matches!(s.anim_type,
            AnimType::Confetti | AnimType::Cannon | AnimType::Pop | AnimType::Drop);
        // Cheap per-frame visibility test, not a retirement: a particle thrown
        // above the top is skipped while it is up there and drawn again on the
        // way back down. Most of a big burst is off-screen most of the time.
        let (mx, my) = (self.w + 60.0, self.h + 60.0);
        for i in 0..n {
            if tf < self.delay[i] { continue; }
            if self.x[i] < -60.0 || self.x[i] > mx || self.y[i] < -60.0 || self.y[i] > my { continue; }
            // The wobble squash is what makes a particle look like paper
            // turning edge-on, so triangles and discs get it too.
            let wob = if use_wobble && (self.kind[i] != Shape::Circle || discs_tumble) {
                self.wobble[i].sin().abs().max(0.15) as f32
            } else {
                1.0
            };
            let sw = wob * self.pw[i];
            if self.kind[i] == Shape::Triangle {
                tris.push((
                    self.x[i], self.y[i], self.rot[i],
                    (sw / 2.0) as f64, (self.ph[i] / 2.0) as f64,
                    self.color[i] as usize, alpha,
                ));
                continue;
            }
            let color = &rgba[self.color[i] as usize];
            snap.save();
            snap.translate(&graphene::Point::new(self.x[i] as f32, self.y[i] as f32));
            snap.rotate(self.rot[i].to_degrees() as f32);
            if self.kind[i] == Shape::Circle {
                let sz = (self.pw[i] + self.ph[i]) / 2.0;
                let w = wob * sz;
                // Corner radii at half the extents turn a rounded rect into an
                // ellipse -- round when face-on, a sliver when edge-on.
                let rect = graphene::Rect::new(-w / 2.0, -sz / 2.0, w, sz);
                let corner = graphene::Size::new(w / 2.0, sz / 2.0);
                snap.push_rounded_clip(&gsk::RoundedRect::new(rect, corner, corner, corner, corner));
                snap.append_color(color, &rect);
                snap.pop();
            } else {
                snap.append_color(color, &graphene::Rect::new(-sw / 2.0, -self.ph[i] / 2.0, sw, self.ph[i]));
            }
            snap.restore();
        }

        self.fill_triangles(snap, &tris, Some(alpha));
    }

    /// One opacity node around the whole batch rather than one per particle:
    /// alpha only varies per particle in sparkle mode, which pushes its own.
    fn fill_triangles(&self, snap: &gtk4::Snapshot,
                      tris: &[(f64, f64, f64, f64, f64, usize, f32)], batch_alpha: Option<f32>) {
        if tris.is_empty() { return }
        if let Some(a) = batch_alpha { snap.push_opacity(a as f64); }
        for &(x, y, rot, hw, hh, ci, a) in tris {
            if batch_alpha.is_none() { snap.push_opacity(a as f64); }
            snap.save();
            snap.translate(&graphene::Point::new(x as f32, y as f32));
            snap.rotate(rot.to_degrees() as f32);
            snap.append_texture(
                &self.tri_tex[ci],
                &graphene::Rect::new(-hw as f32, -hh as f32, (hw * 2.0) as f32, (hh * 2.0) as f32),
            );
            snap.restore();
            if batch_alpha.is_none() { snap.pop(); }
        }
        if batch_alpha.is_some() { snap.pop(); }
    }
}

/// An isoceles triangle, apex up, 4x4 supersampled for smooth edges and stored
/// premultiplied. Built once per run, one per colour -- the coverage mask is
/// shape, not colour, so it is rasterized once and tinted N times.
fn triangle_textures(colors: &[[f32; 3]]) -> Vec<gdk::Texture> {
    const S: usize = 48;
    let mut cov = vec![0.0f32; S * S];
    for py in 0..S {
        for px in 0..S {
            let mut hits = 0;
            for sy in 0..4 {
                for sx in 0..4 {
                    let u = (px as f32 + (sx as f32 + 0.5) / 4.0) / S as f32;
                    let v = (py as f32 + (sy as f32 + 0.5) / 4.0) / S as f32;
                    if (u - 0.5).abs() <= v * 0.5 { hits += 1; }
                }
            }
            cov[py * S + px] = hits as f32 / 16.0;
        }
    }

    colors.iter().map(|c| {
        let mut buf = vec![0u8; S * S * 4];
        for (i, &a) in cov.iter().enumerate() {
            let o = i * 4;
            buf[o]     = (c[0] * a * 255.0) as u8;
            buf[o + 1] = (c[1] * a * 255.0) as u8;
            buf[o + 2] = (c[2] * a * 255.0) as u8;
            buf[o + 3] = (a * 255.0) as u8;
        }
        gdk::MemoryTexture::new(
            S as i32, S as i32,
            gdk::MemoryFormat::R8g8b8a8Premultiplied,
            &glib::Bytes::from_owned(buf), S * 4,
        ).upcast()
    }).collect()
}
