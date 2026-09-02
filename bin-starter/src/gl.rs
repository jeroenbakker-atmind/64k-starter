#![cfg_attr(windows, allow(clippy::missing_safety_doc))]

use core::ffi::c_void;

#[cfg(windows)]
use alloc::vec::Vec;
#[cfg(target_os = "macos")]
use std::vec::Vec;

use once_cell::sync::OnceCell;

#[allow(dead_code)]
pub enum ShaderType {
    Fragment,
    Vertex,
    Geometry,
    Compute,
}

impl Into<i32> for ShaderType {
    fn into(self) -> i32 {
        match self {
            ShaderType::Fragment => 0x8b30,
            ShaderType::Vertex => 0x8b31,
            ShaderType::Geometry => 0x8dd9,
            ShaderType::Compute => 0x91b9,
        }
    }
}

macro_rules! glcall {
    ($t:ty, $fn:literal) => {{
        static CELL: OnceCell<$t> = OnceCell::new();
        CELL.get_or_init(|| core::mem::transmute(load(concat!($fn, "\0").as_bytes())))
    }};
}

pub struct Program {
    idx: u32,
    #[cfg(target_os = "macos")]
    pipeline: u32,
    uniforms: Vec<(*const u8, i32)>,
}

impl Program {
    #[allow(unused_variables)]
    pub unsafe fn new(shader_type: ShaderType, source: &'static [u8]) -> Program {
        #[cfg(windows)]
        let idx = glcall!(unsafe extern "C" fn(i32, u32, &*const u8) -> u32, "glCreateShaderProgramv")(shader_type.into(), 1, &source.as_ptr());

        #[cfg(target_os = "macos")]
        let (idx, pipeline) = {
            // macOS ships OpenGL 4.1 core (no fixed-function pipeline), so the
            // fragment-only program needs a trivial vertex stage to render the
            // fullscreen quad. Build a separable vertex+fragment program
            // pipeline; the fragment source is the minified shader with
            // `gl_FragColor` (GLSL 1.x) rewritten for a GLSL 4.10 core output.
            let vert = b"#version 410 core\nin vec2 aPos;out gl_PerVertex{vec4 gl_Position;};void main(){gl_Position=vec4(aPos,0.0,1.0);}\0";
            let vert_idx = glcall!(unsafe extern "C" fn(i32, u32, &*const u8) -> u32, "glCreateShaderProgramv")(0x8b31, 1, &vert.as_ptr());

            let mut frag: Vec<u8> = Vec::with_capacity(source.len() + 64);
            frag.extend_from_slice(b"#version 410 core\nout vec4 fragColor;\n");
            const FIND: &[u8] = b"gl_FragColor";
            const REPLACE: &[u8] = b"fragColor";
            let mut rest = source;
            while let Some(pos) = rest.windows(FIND.len()).position(|w| w == FIND) {
                frag.extend_from_slice(&rest[..pos]);
                frag.extend_from_slice(REPLACE);
                rest = &rest[pos + FIND.len()..];
            }
            frag.extend_from_slice(rest);
            frag.push(0);
            let frag_idx = glcall!(unsafe extern "C" fn(i32, u32, &*const u8) -> u32, "glCreateShaderProgramv")(0x8b30, 1, &frag.as_ptr());

let mut pipeline: u32 = 0;
            glcall!(unsafe extern "C" fn(i32, *mut u32) , "glGenProgramPipelines")(1, &mut pipeline);
            // GL_VERTEX_SHADER_BIT | GL_FRAGMENT_SHADER_BIT
            glcall!(unsafe extern "C" fn(u32, u32, u32) , "glUseProgramStages")(pipeline, 0x1, vert_idx);
            glcall!(unsafe extern "C" fn(u32, u32, u32) , "glUseProgramStages")(pipeline, 0x2, frag_idx);

            debug_assert!(!debug_program_issue("vertex", vert_idx), "vertex program failed to link");
            debug_assert!(!debug_program_issue("fragment", frag_idx), "fragment program failed to link");

            (frag_idx, pipeline)
        };

        Program {
            idx,
            #[cfg(target_os = "macos")]
            pipeline,
            uniforms: Vec::new(),
        }
    }

    pub unsafe fn bind(&self) {
        #[cfg(windows)]
        glcall!(unsafe extern "C" fn(u32) , "glUseProgram")(self.idx);

        #[cfg(target_os = "macos")]
        glcall!(unsafe extern "C" fn(u32) , "glBindProgramPipeline")(self.pipeline);
    }

    pub unsafe fn set_uniform_f32(&mut self, name: &'static [u8], value: f32) {
        glcall!(unsafe extern "C" fn(u32, i32, f32) , "glProgramUniform1f")(self.idx, self.get_uniform_location(name), value);
    }

    unsafe fn get_uniform_location(&mut self, name: &'static [u8]) -> i32 {
        if let Some(cached) = self.uniforms.iter().find(|x| x.0 == name.as_ptr()) {
            return cached.1;
        }

        let location = glcall!(unsafe extern "C" fn(u32, *const u8) -> i32, "glGetUniformLocation")(self.idx, name.as_ptr());
        self.uniforms.push((name.as_ptr(), location));
        location
    }
}

/// Debug-only: returns true and prints the info log if the program is invalid
/// or failed to link.
#[cfg(all(target_os = "macos", debug_assertions))]
unsafe fn debug_program_issue(name: &str, prog: u32) -> bool {
    if prog == 0 {
        eprintln!("[gl] {name}: glCreateShaderProgramv returned 0");
        return true;
    }
    // GL_LINK_STATUS
    let mut status: i32 = 0;
    glcall!(unsafe extern "C" fn(u32, u32, *mut i32), "glGetProgramiv")(prog, 0x8b82, &mut status);
    if status != 0 {
        return false;
    }
    // GL_INFO_LOG_LENGTH
    let mut len: i32 = 0;
    glcall!(unsafe extern "C" fn(u32, u32, *mut i32), "glGetProgramiv")(prog, 0x8b84, &mut len);
    let mut buf: Vec<u8> = vec![0u8; len.max(1) as usize];
    let mut written: i32 = 0;
    glcall!(unsafe extern "C" fn(u32, i32, *mut i32, *mut u8), "glGetProgramInfoLog")(
        prog,
        len,
        &mut written,
        buf.as_mut_ptr(),
    );
    let log = std::str::from_utf8(&buf[..written.max(0) as usize]).unwrap_or("<invalid utf8>");
    eprintln!("[gl] {name} link failed: {log}");
    true
}

#[cfg(all(target_os = "macos", not(debug_assertions)))]
unsafe fn debug_program_issue(_name: &str, _prog: u32) -> bool {
    false
}

/// Renders a fullscreen quad. Windows uses fixed-function immediate mode
/// (`glRects`); macOS runs a 4.1 core profile, so it draws a fullscreen
/// triangle through the VAO/pipeline set up in `Program::new`.
pub unsafe fn draw_fullscreen_quad() {
    #[cfg(windows)]
    windows_sys::Win32::Graphics::OpenGL::glRects(-1, -1, 1, 1);

    #[cfg(target_os = "macos")]
    {
        // Cache is a crate-private static in this module; OnceCell keyed by
        // the DSO handle. Vertex data: a single fullscreen triangle.
        static mut SETUP: bool = false;
        static mut VAO: u32 = 0;
        static mut VBO: u32 = 0;
        if !SETUP {
            let verts: [f32; 6] = [-1.0, -1.0, 3.0, -1.0, -1.0, 3.0];
            // GL_ARRAY_BUFFER
glcall!(unsafe extern "C" fn(i32, *mut u32), "glGenVertexArrays")(1, core::ptr::addr_of_mut!(VAO));
            glcall!(unsafe extern "C" fn(u32), "glBindVertexArray")(VAO);
            glcall!(unsafe extern "C" fn(i32, *mut u32), "glGenBuffers")(1, core::ptr::addr_of_mut!(VBO));
            glcall!(unsafe extern "C" fn(u32, u32) , "glBindBuffer")(0x8892, VBO);
            // GL_STATIC_DRAW
            glcall!(unsafe extern "C" fn(u32, isize, *const c_void, u32) , "glBufferData")(0x8892, (verts.len() * 4) as isize, verts.as_ptr() as *const c_void, 0x88e4);
            // GL_FLOAT
            glcall!(unsafe extern "C" fn(u32, i32, u32, u8, i32, *const c_void) , "glVertexAttribPointer")(0, 2, 0x1406, 0, 0, core::ptr::null());
            glcall!(unsafe extern "C" fn(u32) , "glEnableVertexAttribArray")(0);
            SETUP = true;
        }

        glcall!(unsafe extern "C" fn(u32) , "glBindVertexArray")(VAO);
        // GL_TRIANGLES
        glcall!(unsafe extern "C" fn(u32, i32, i32) , "glDrawArrays")(0x0004, 0, 3);
    }
}

#[cfg(windows)]
unsafe fn load(name: &'static [u8]) -> unsafe extern "system" fn() -> isize {
    use windows_sys::Win32::Graphics::OpenGL::wglGetProcAddress;
    wglGetProcAddress(name.as_ptr()).unwrap()
}

#[cfg(target_os = "macos")]
unsafe fn load(name: &'static [u8]) -> unsafe extern "C" fn() -> isize {
    unsafe extern "C" {
        fn dlsym(handle: *mut c_void, sym: *const u8) -> *mut c_void;
    }
    const RTLD_DEFAULT: *mut c_void = -1isize as *mut c_void;
    let c = core::ffi::CStr::from_bytes_with_nul_unchecked(name);
    core::mem::transmute(dlsym(RTLD_DEFAULT, c.as_ptr() as *const u8))
}