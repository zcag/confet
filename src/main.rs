mod canvas;
mod config;
mod particle;
mod platform;
mod sound;
mod types;

use canvas::Canvas;
use config::Settings;
use gtk4::prelude::*;
use gtk4::gdk;
use std::process::{Command, Stdio};

const BACKGROUND_GUARD: &str = "CONFET_BACKGROUND";

/// Re-exec self with stdio detached, so the calling shell returns right away.
/// This is the default: confet has no result to wait for -- it always exits 0
/// and prints nothing, so blocking the caller protects nothing and costs them
/// the length of the animation. `--wait` opts back in, for the rare caller
/// that treats the animation itself as the artifact (see scripts/record-demos.sh).
///
/// Spawns (fork+exec) instead of a bare fork because GTK is Cocoa-backed on
/// macOS, where forking without exec is not safe. Returns false if the respawn
/// didn't happen, in which case we just animate in the foreground.
fn respawn_in_background() -> bool {
    if std::env::var_os(BACKGROUND_GUARD).is_some() { return false; }
    let Ok(exe) = std::env::current_exe() else { return false };
    Command::new(exe)
        .args(std::env::args_os().skip(1))
        .env(BACKGROUND_GUARD, "1")
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
        .spawn()
        .is_ok()
}

fn main() {
    let file = config::load_file_config();
    let cli = config::parse_cli(&file);
    if cli.init {
        config::init_config();
        return;
    }
    if !cli.wait && respawn_in_background() {
        return;
    }
    config::set_settings(Settings::resolve(cli, file));

    let app = gtk4::Application::builder()
        .application_id("dev.confetti.overlay")
        .flags(gtk4::gio::ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_activate(|app| {
        let win = gtk4::Window::builder().application(app).build();

        platform::setup_window(&win);

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

        win.present();

        if let Some(spec) = &config::settings().sound {
            sound::play(spec);
        }

        let display = gdk::Display::default().unwrap();
        let monitor: gdk::Monitor = display.monitors()
            .item(0).unwrap().downcast().unwrap();
        let geom = monitor.geometry();
        canvas.start(geom.width() as f64, geom.height() as f64);
    });

    app.run_with_args::<&str>(&[]);
}
