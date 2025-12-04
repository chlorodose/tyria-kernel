//! Architecture-specific low level codes(asm warning!)
#[cfg(target_arch = "x86_64")]
#[doc(hidden)]
pub mod amd64;
#[cfg(target_arch = "x86_64")]
pub use amd64::*;
