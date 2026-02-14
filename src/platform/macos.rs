use gtk4::prelude::*;
use gtk4::glib;
use objc2::runtime::{AnyClass, AnyObject, Bool};
use objc2::{msg_send, msg_send_id};
use objc2::rc::Retained;

#[repr(C)]
#[derive(Copy, Clone)]
struct CGPoint { x: f64, y: f64 }

#[repr(C)]
#[derive(Copy, Clone)]
struct CGSize { width: f64, height: f64 }

#[repr(C)]
#[derive(Copy, Clone)]
struct CGRect { origin: CGPoint, size: CGSize }

pub fn setup_window(win: &gtk4::Window) {
    win.set_decorated(false);

    // Configure the underlying NSWindow after GTK realizes it
    win.connect_realize(|_| {
        glib::idle_add_once(configure_ns_window);
    });
}

fn configure_ns_window() {
    unsafe {
        let app: Retained<AnyObject> = msg_send_id![
            AnyClass::get("NSApplication").unwrap(),
            sharedApplication
        ];

        let Some(ns_win): Option<Retained<AnyObject>> = msg_send_id![&*app, keyWindow]
        else { return };

        // Overlay level (kCGScreenSaverWindowLevel = 1000)
        let _: () = msg_send![&*ns_win, setLevel: 1000_i64];

        // Transparent background
        let _: () = msg_send![&*ns_win, setOpaque: Bool::NO];
        let clear: Retained<AnyObject> = msg_send_id![
            AnyClass::get("NSColor").unwrap(),
            clearColor
        ];
        let _: () = msg_send![&*ns_win, setBackgroundColor: &*clear];

        // Click-through
        let _: () = msg_send![&*ns_win, setIgnoresMouseEvents: Bool::YES];

        // No shadow
        let _: () = msg_send![&*ns_win, setHasShadow: Bool::NO];

        // Borderless (NSWindowStyleMaskBorderless = 0)
        let _: () = msg_send![&*ns_win, setStyleMask: 0_u64];

        // Cover full screen
        let Some(screen): Option<Retained<AnyObject>> = msg_send_id![&*ns_win, screen]
        else { return };
        let frame: CGRect = msg_send![&*screen, frame];
        let _: () = msg_send![&*ns_win, setFrame: frame display: Bool::YES];
    }
}
