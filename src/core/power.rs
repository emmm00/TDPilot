//! Live CPU clock via `CallNtPowerInformation(ProcessorInformation)`.
//!
//! ryzenadj's PM table has a `CCLK BUSY VALUE` row, but on Hawk Point it
//! reports `nan`, so we fall back to the kernel's own per-core counters and
//! report the busiest core (which is what a user watching a clock expects).

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct ProcessorPowerInformation {
    number: u32,
    max_mhz: u32,
    current_mhz: u32,
    mhz_limit: u32,
    max_idle_state: u32,
    current_idle_state: u32,
}

#[link(name = "powrprof")]
unsafe extern "system" {
    fn CallNtPowerInformation(
        information_level: i32,
        input_buffer: *mut core::ffi::c_void,
        input_buffer_size: u32,
        output_buffer: *mut core::ffi::c_void,
        output_buffer_size: u32,
    ) -> i32;
}

const PROCESSOR_INFORMATION: i32 = 11;
const MAX_CORES: usize = 64;

/// Highest current clock across all logical processors, in MHz.
/// Returns 0.0 if the call fails.
pub fn cpu_mhz() -> f32 {
    let mut buf = [ProcessorPowerInformation::default(); MAX_CORES];
    let bytes = (MAX_CORES * std::mem::size_of::<ProcessorPowerInformation>()) as u32;

    let status = unsafe {
        CallNtPowerInformation(
            PROCESSOR_INFORMATION,
            std::ptr::null_mut(),
            0,
            buf.as_mut_ptr() as *mut core::ffi::c_void,
            bytes,
        )
    };
    if status < 0 {
        return 0.0;
    }

    buf.iter()
        .map(|p| p.current_mhz as f32)
        .fold(0.0f32, f32::max)
}
