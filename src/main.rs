mod canvas;
mod config;
mod particle;
mod types;

use canvas::Canvas;
use clap::Parser;
use config::{Cli, Settings};
use gtk4::prelude::*;
use gtk4::gdk;
use gtk4_layer_shell::LayerShell;

fn main() {
    let cli = Cli::parse();
    if cli.init {
        config::init_config();
        return;
    }
    let file = config::load_file_config();
    config::set_settings(Settings::resolve(cli, file));

    let app = gtk4::Application::builder()
        .application_id("dev.confetti.overlay")
        .flags(gtk4::gio::ApplicationFlags::NON_UNIQUE)
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
