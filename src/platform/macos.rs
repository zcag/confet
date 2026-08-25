use gtk4::prelude::*;
use gtk4::glib;
use objc2::encode::{Encode, Encoding};
use objc2::runtime::{AnyClass, AnyObject, Bool};
use objc2::{msg_send, msg_send_id};
use objc2::rc::Retained;

// msg_send! marshals arguments and return values through objc2's Encode, so any
// struct crossing that boundary needs an encoding that matches the Objective-C
// type. Declaring the #[repr(C)] layout alone is not enough -- without these
// impls `screen.frame` and `setFrame:display:` do not compile. The encodings
// mirror objc2-foundation's own NSPoint/NSSize/NSRect, which is why no extra
// dependency is needed.

#[repr(C)]
#[derive(Copy, Clone)]
struct CGPoint { x: f64, y: f64 }

unsafe impl Encode for CGPoint {
    const ENCODING: Encoding = Encoding::Struct("CGPoint", &[f64::ENCODING, f64::ENCODING]);
}

#[repr(C)]
#[derive(Copy, Clone)]
struct CGSize { width: f64, height: f64 }

unsafe impl Encode for CGSize {
    const ENCODING: Encoding = Encoding::Struct("CGSize", &[f64::ENCODING, f64::ENCODING]);
}

#[repr(C)]
#[derive(Copy, Clone)]
struct CGRect { origin: CGPoint, size: CGSize }

unsafe impl Encode for CGRect {
    const ENCODING: Encoding = Encoding::Struct("CGRect", &[CGPoint::ENCODING, CGSize::ENCODING]);
}

pub fn setup_window(win: &gtk4::Window) {
    win.set_decorated(false);

    // Do this before anything is shown: NSApplicationActivationPolicyAccessory
    // (1) keeps confet out of the Dock and out of the activation order, so
    // firing it after a build doesn't pull focus off whatever you're typing in.
    // The default policy is Regular, which makes the overlay the front app.
    unsafe {
        let app: Retained<AnyObject> = msg_send_id![
            AnyClass::get("NSApplication").unwrap(),
            sharedApplication
        ];
        let _: Bool = msg_send![&*app, setActivationPolicy: 1_isize];
    }

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

        // Take the window off -[NSApp windows] rather than -keyWindow: with the
        // accessory policy the app never activates, so there is no key window.
        let windows: Retained<AnyObject> = msg_send_id![&*app, windows];
        let count: usize = msg_send![&*windows, count];
        if count == 0 { return }
        let ns_win: *mut AnyObject = msg_send![&*windows, objectAtIndex: count - 1];
        if ns_win.is_null() { return }
        let ns_win = &*ns_win;

        // Overlay level (kCGScreenSaverWindowLevel = 1000)
        let _: () = msg_send![ns_win, setLevel: 1000_i64];

        // Transparent background
        let _: () = msg_send![ns_win, setOpaque: Bool::NO];
        let clear: Retained<AnyObject> = msg_send_id![
            AnyClass::get("NSColor").unwrap(),
            clearColor
        ];
        let _: () = msg_send![ns_win, setBackgroundColor: &*clear];

        // Click-through
        let _: () = msg_send![ns_win, setIgnoresMouseEvents: Bool::YES];

        // No shadow
        let _: () = msg_send![ns_win, setHasShadow: Bool::NO];

        // Borderless (NSWindowStyleMaskBorderless = 0)
        let _: () = msg_send![ns_win, setStyleMask: 0_u64];

        // Show it without taking key/main away from the focused app
        let _: () = msg_send![ns_win, orderFrontRegardless];

        // Cover full screen
        let Some(screen): Option<Retained<AnyObject>> = msg_send_id![ns_win, screen]
        else { return };
        let frame: CGRect = msg_send![&*screen, frame];
        let _: () = msg_send![ns_win, setFrame: frame display: Bool::YES];
    }
}
