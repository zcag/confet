mod canvas;
mod config;
mod particle;
mod platform;
mod types;

use canvas::Canvas;
use config::Settings;
use gtk4::prelude::*;
use gtk4::gdk;

fn main() {
    let file = config::load_file_config();
    let cli = config::parse_cli(&file);
    if cli.init {
        config::init_config();
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

        let display = gdk::Display::default().unwrap();
        let monitor: gdk::Monitor = display.monitors()
            .item(0).unwrap().downcast().unwrap();
        let geom = monitor.geometry();
        canvas.start(geom.width() as f64, geom.height() as f64);
    });

    app.run_with_args::<&str>(&[]);
}
