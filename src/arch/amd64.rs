use core::arch::asm;
pub fn halt() -> ! {
    loop {
        unsafe { asm!("hlt", options(att_syntax, nomem, nostack)) };
    }
}
