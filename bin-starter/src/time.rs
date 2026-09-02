use core::time::Duration;

#[cfg(windows)]
pub unsafe fn elapsed() -> Duration {
    use core::ptr;

    use windows_sys::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};

    unsafe fn ticks() -> i64 {
        let mut count = 0i64;
        QueryPerformanceCounter(ptr::addr_of_mut!(count));
        count
    }

    static mut FREQ: i64 = 0;
    if FREQ == 0 {
        QueryPerformanceFrequency(ptr::addr_of_mut!(FREQ));
    }

    static mut START: i64 = 0;
    if START == 0 {
        START = ticks();
    }

    Duration::from_secs_f64((ticks() - START) as f64 / FREQ as f64)
}

#[cfg(target_os = "macos")]
pub unsafe fn elapsed() -> Duration {
    #[repr(C)]
    struct MachTimebaseInfoData {
        numer: u32,
        denom: u32,
    }

    unsafe extern "C" {
        fn mach_absolute_time() -> u64;
        fn mach_timebase_info(info: *mut MachTimebaseInfoData) -> i32;
    }

    static mut TB: MachTimebaseInfoData = MachTimebaseInfoData { numer: 1, denom: 1 };
    static mut INIT: bool = false;
    if !INIT {
        mach_timebase_info(core::ptr::addr_of_mut!(TB));
        INIT = true;
    }

    static mut START: u64 = 0;
    if START == 0 {
        START = mach_absolute_time();
    }

    let now = mach_absolute_time();
    let ticks = now.wrapping_sub(START);
    let nanos = ticks * TB.numer as u64 / TB.denom as u64;
    Duration::from_nanos(nanos)
}