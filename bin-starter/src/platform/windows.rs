use core::mem;

use windows_sys::Win32::{
    Graphics::{
        Gdi::{
            ChangeDisplaySettingsA, GetDC, CDS_FULLSCREEN, DEVMODEA, DM_BITSPERPEL, DM_PELSHEIGHT,
            DM_PELSWIDTH, HDC,
        },
        OpenGL::{
            wglCreateContext, wglMakeCurrent, ChoosePixelFormat, SetPixelFormat, SwapBuffers,
            PFD_DOUBLEBUFFER, PFD_SUPPORT_OPENGL, PIXELFORMATDESCRIPTOR,
        },
    },
    System::Threading::ExitProcess,
    UI::{
        Input::KeyboardAndMouse::{GetAsyncKeyState, VK_ESCAPE},
        WindowsAndMessaging::{CreateWindowExA, ShowCursor, WS_MAXIMIZE, WS_POPUP, WS_VISIBLE},
    },
};

pub unsafe fn enter_fullscreen() {
    let mut mode: DEVMODEA = mem::zeroed();
    mode.dmSize = mem::size_of::<DEVMODEA>() as u16;
    mode.dmPelsWidth = 1920;
    mode.dmPelsHeight = 1080;
    mode.dmBitsPerPel = 32;
    mode.dmFields = DM_PELSWIDTH | DM_PELSHEIGHT | DM_BITSPERPEL;

    ChangeDisplaySettingsA(&mode, CDS_FULLSCREEN);
    ShowCursor(0);
}

pub unsafe fn create_device() -> HDC {
    let handle = CreateWindowExA(
        0,
        "edit\0".as_ptr(),
        core::ptr::null(),
        WS_POPUP | WS_VISIBLE | WS_MAXIMIZE,
        0,
        0,
        0,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null(),
    );

    let device = GetDC(handle);
    let mut pfd: PIXELFORMATDESCRIPTOR = mem::zeroed();
    pfd.dwFlags = PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER;
    let format = ChoosePixelFormat(device, &pfd);
    SetPixelFormat(device, format, &pfd);
    wglMakeCurrent(device, wglCreateContext(device));
    device
}

pub unsafe fn should_exit() -> bool {
    GetAsyncKeyState(VK_ESCAPE as i32) != 0
}

pub unsafe fn pump_events() {}

pub unsafe fn swap_buffers(device: HDC) {
    SwapBuffers(device);
}

pub unsafe fn exit(code: i32) -> ! {
    ExitProcess(code as u32);
}