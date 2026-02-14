use gtk4::prelude::*;
use gtk4_layer_shell::LayerShell;

pub fn setup_window(win: &gtk4::Window) {
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

    win.connect_realize(|w| {
        if let Some(surface) = w.surface() {
            let region = cairo::Region::create_rectangle(
                &cairo::RectangleInt::new(0, 0, 0, 0),
            );
            surface.set_input_region(&region);
        }
    });
}
