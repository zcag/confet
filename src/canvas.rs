use crate::config::settings;
use crate::particle::Particles;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::glib;
use std::cell::RefCell;
use std::time::Instant;

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
    pub fn new() -> Self { glib::Object::builder().build() }

    pub fn start(&self, w: f64, h: f64) {
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
