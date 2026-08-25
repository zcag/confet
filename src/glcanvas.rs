// Spike: does a GLArea composite with alpha over the transparent overlay?
use glow::HasContext;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::Rc;

fn load_gl() -> glow::Context {
    // GTK already links libepoxy, which exports every GL entry point, so the
    // symbols are resolvable from the running process -- no dlopen of a
    // platform-specific filename needed.
    let lib = libloading::os::unix::Library::this();
    unsafe {
        glow::Context::from_loader_function(|name| {
            lib.get::<*mut c_void>(name.as_bytes())
                .map(|s| s.into_raw() as *const c_void)
                .unwrap_or(std::ptr::null())
        })
    }
}

const VS: &str = r#"#version 150 core
in vec2 pos;
void main() { gl_Position = vec4(pos, 0.0, 1.0); }
"#;

const FS: &str = r#"#version 150 core
out vec4 frag;
void main() { frag = vec4(1.0, 0.0, 0.0, 1.0); }
"#;

pub fn build() -> gtk4::GLArea {
    let area = gtk4::GLArea::new();
    area.set_required_version(3, 2);
    area.set_has_depth_buffer(false);
    area.set_has_stencil_buffer(false);
    area.set_hexpand(true);
    area.set_vexpand(true);

    let state: Rc<RefCell<Option<(glow::Context, glow::Program, glow::VertexArray)>>> =
        Rc::new(RefCell::new(None));

    let s = state.clone();
    area.connect_realize(move |a| {
        a.make_current();
        if let Some(e) = a.error() { eprintln!("glarea error: {e}"); return }
        let gl = load_gl();
        unsafe {
            let vs = gl.create_shader(glow::VERTEX_SHADER).unwrap();
            gl.shader_source(vs, VS); gl.compile_shader(vs);
            if !gl.get_shader_compile_status(vs) { eprintln!("VS: {}", gl.get_shader_info_log(vs)); }
            let fs = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
            gl.shader_source(fs, FS); gl.compile_shader(fs);
            if !gl.get_shader_compile_status(fs) { eprintln!("FS: {}", gl.get_shader_info_log(fs)); }
            let prog = gl.create_program().unwrap();
            gl.attach_shader(prog, vs); gl.attach_shader(prog, fs); gl.link_program(prog);
            if !gl.get_program_link_status(prog) { eprintln!("link: {}", gl.get_program_info_log(prog)); }
            gl.delete_shader(vs); gl.delete_shader(fs);

            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));
            let vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            let verts: [f32; 6] = [-0.5, -0.5, 0.5, -0.5, 0.0, 0.5];
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER,
                std::slice::from_raw_parts(verts.as_ptr() as *const u8, 24), glow::STATIC_DRAW);
            let loc = gl.get_attrib_location(prog, "pos").unwrap();
            gl.enable_vertex_attrib_array(loc);
            gl.vertex_attrib_pointer_f32(loc, 2, glow::FLOAT, false, 8, 0);
            eprintln!("GL ready: {}", gl.get_parameter_string(glow::VERSION));
            *s.borrow_mut() = Some((gl, prog, vao));
        }
    });

    let s = state.clone();
    let frames = std::cell::Cell::new(0u32);
    area.connect_render(move |a, _| {
        frames.set(frames.get() + 1);
        if frames.get() <= 2 {
            eprintln!("render #{} size {}x{} scale {}",
                frames.get(), a.width(), a.height(), a.scale_factor());
        }
        if let Some((gl, prog, vao)) = s.borrow().as_ref() {
            unsafe {
                gl.clear_color(0.0, 0.0, 0.0, 0.0);
                gl.clear(glow::COLOR_BUFFER_BIT);
                gl.use_program(Some(*prog));
                gl.bind_vertex_array(Some(*vao));
                gl.draw_arrays(glow::TRIANGLES, 0, 3);
                let e = gl.get_error();
                if e != 0 { eprintln!("gl error {e:#x}"); }
            }
        }
        glib::Propagation::Stop
    });
    area
}
use gtk4::glib;
