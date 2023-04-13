#![no_std]
#![no_main]

#[allow(non_upper_case_globals)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
mod vmlinux;

mod network_monitor;
mod process_monitor;

pub use network_monitor::network_monitor;
pub use process_monitor::process_monitor;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
