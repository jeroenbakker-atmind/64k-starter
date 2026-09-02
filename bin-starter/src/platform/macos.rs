#![allow(non_upper_case_globals)]

use objc::{
    class, msg_send,
    runtime::{Object, NO, YES},
    sel, sel_impl,
};

pub type Device = usize;

const NSScreenSaverWindowLevel: i64 = 1000;

// Cocoa structs; passed to/returned from objc_msgSend by value. On aarch64
// objc_msgSend handles struct returns directly (no stret variant), and the
// argument box just forwards values as `extern "C"` arguments.
#[repr(C)]
#[derive(Clone, Copy)]
struct NSPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NSSize {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NSRect {
    origin: NSPoint,
    size: NSSize,
}

unsafe extern "C" {
    static kCFRunLoopDefaultMode: *const core::ffi::c_void;
    fn CGEventSourceKeyState(state_id: i32, keycode: u16) -> i32;
}

static mut APP: *mut Object = core::ptr::null_mut();
static mut WINDOW: *mut Object = core::ptr::null_mut();
static mut CONTENT_VIEW: *mut Object = core::ptr::null_mut();
static mut CONTEXT: *mut Object = core::ptr::null_mut();

unsafe fn app() -> *mut Object {
    if !APP.is_null() {
        return APP;
    }
    let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
    debug_assert!(!app.is_null(), "NSApplication sharedApplication returned nil");
    APP = app;
    app
}

pub unsafe fn enter_fullscreen() {
    debug_assert!(
        CONTEXT.is_null(),
        "enter_fullscreen must be called before create_device"
    );

    let app = app();

    let _: () = msg_send![app, setActivationPolicy: 0i64]; // NSApplicationActivationPolicyRegular

    let screen: *mut Object = msg_send![class!(NSScreen), mainScreen];
    debug_assert!(!screen.is_null(), "[NSScreen mainScreen] returned nil");
    let frame: NSRect = msg_send![screen, frame];
    debug_assert!(
        frame.size.width > 0.0 && frame.size.height > 0.0,
        "[NSScreen mainScreen] frame is empty ({:?}, {:?})",
        frame.size.width,
        frame.size.height
    );

    let window: *mut Object = msg_send![class!(NSWindow), alloc];
    let window: *mut Object =
        msg_send![window, initWithContentRect: frame styleMask: 0i64 backing: 2i64 defer: NO];
    debug_assert!(!window.is_null(), "NSWindow init failed");
    let _: () = msg_send![window, setFrame: frame display: YES];
    let _: () = msg_send![window, setLevel: NSScreenSaverWindowLevel];
    let _: () = msg_send![window, makeKeyAndOrderFront: core::ptr::null::<Object>()];
    let _: () = msg_send![window, orderFrontRegardless];

    WINDOW = window;

    let content_view: *mut Object = msg_send![window, contentView];
    debug_assert!(!content_view.is_null(), "contentView is nil after init");
    CONTENT_VIEW = content_view;

    let _: () = msg_send![app, activateIgnoringOtherApps: YES];
    let _: () = msg_send![app, finishLaunching];

    let _: () = msg_send![class!(NSCursor), hide];
}

pub unsafe fn create_device() -> Device {
    debug_assert!(!CONTENT_VIEW.is_null(), "enter_fullscreen must run first");

    // NSOpenGLPixelFormat attributes: 4.1 core profile, accelerated, double
    // buffered, 32-bit color, then a 0 terminator.
    let attrs: [u32; 7] = [99, 0x4100, 73, 5, 8, 32, 0];
    let pixel_format: *mut Object = msg_send![class!(NSOpenGLPixelFormat), alloc];
    let pixel_format: *mut Object = msg_send![pixel_format, initWithAttributes: attrs.as_ptr()];
    debug_assert!(
        !pixel_format.is_null(),
        "NSOpenGLPixelFormat init failed (4.1 core profile not supported?)"
    );

    let context: *mut Object = msg_send![class!(NSOpenGLContext), alloc];
    let context: *mut Object =
        msg_send![context, initWithFormat: pixel_format shareContext: core::ptr::null::<Object>()];
    debug_assert!(!context.is_null(), "NSOpenGLContext init failed");

    let _: () = msg_send![context, setView: CONTENT_VIEW];
    let _: () = msg_send![context, makeCurrentContext];

    CONTEXT = context;
    context as usize
}

/// Drains the Cocoa event queue so the window actually gets ordered front,
/// displayed and repainted by AppKit. The demo doesn't run NSApplicationMain,
/// so without this the window never appears even though the app is active.
pub unsafe fn pump_events() {
    let app = app();
    // NSAnyEventMask (NSUIntegerMax); untilDate: nil returns immediately.
    let event: *mut Object = msg_send![
        app,
        nextEventMatchingMask: u64::MAX
        untilDate: core::ptr::null::<Object>()
        inMode: kCFRunLoopDefaultMode
        dequeue: YES
    ];
    if !event.is_null() {
        let _: () = msg_send![app, sendEvent: event];
    }
}

pub unsafe fn should_exit() -> bool {
    // kVK_Escape = 53; read the HID key state directly (no event loop needed).
    CGEventSourceKeyState(0, 53) != 0
}

pub unsafe fn swap_buffers(_: Device) {
    debug_assert!(!CONTEXT.is_null(), "swap_buffers before create_device");
    let _: () = msg_send![CONTEXT, flushBuffer];
}

pub unsafe fn exit(code: i32) -> ! {
    std::process::exit(code);
}