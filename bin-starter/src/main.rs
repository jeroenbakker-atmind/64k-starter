#![cfg_attr(windows, no_std)]
#![cfg_attr(windows, no_main)]
#![cfg_attr(windows, windows_subsystem = "windows")]
#![allow(unexpected_cfgs)]

#[cfg(windows)]
#[global_allocator]
static ALLOCATOR: libc_alloc::LibcAlloc = libc_alloc::LibcAlloc;

#[cfg(windows)]
#[link(name = "libcmt")]
extern "C" {}
#[cfg(windows)]
#[link(name = "ucrt")]
extern "C" {}
#[cfg(windows)]
#[link(name = "uuid")]
extern "C" {}
#[cfg(windows)]
#[link(name = "vcruntime")]
extern "C" {}

#[cfg(windows)]
extern crate alloc;

mod audio;
#[cfg(windows)]
mod critical;
mod gl;
mod glsl;
mod platform;
mod time;

static SONG_BLOB: &'static [u8] = include_bytes!("song.bin");

unsafe fn run() {
    platform::enter_fullscreen();
    let device = platform::create_device();
    let mut program = gl::Program::new(gl::ShaderType::Fragment, glsl::SHADER_FRAG);
    program.bind();

    let length = audio::song_length(SONG_BLOB);
    let _player = audio::play(SONG_BLOB);

    loop {
        platform::pump_events();
        if platform::should_exit() {
            break;
        }
        let elapsed = time::elapsed();
        if elapsed > length {
            break;
        }

        program.set_uniform_f32(glsl::VAR_ITIME, elapsed.as_secs_f32());
        gl::draw_fullscreen_quad();
        platform::swap_buffers(device);
    }

    platform::exit(0);
}

#[cfg(windows)]
#[no_mangle]
extern "C" fn mainCRTStartup() {
    unsafe {
        run();
    }
}

#[cfg(target_os = "macos")]
fn main() {
    unsafe {
        run();
    }
}

#[cfg(windows)]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    unsafe {
        platform::exit(0xFFFF);
    }
}