#![no_std]
#![no_main]
use tyria_kernel as _;

#[unsafe(export_name = "_start")]
pub extern "C" fn start_kernel() -> ! {
    panic!("");
}
