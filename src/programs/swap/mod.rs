#[cfg(feature = "perena-swap")]
pub mod perena;

#[cfg(feature = "solfi-swap")]
pub mod solfi;

#[cfg(feature = "solfi_v2-swap")]
pub mod solfi_v2;

#[cfg(feature = "manifest-swap")]
pub mod manifest;

#[cfg(feature = "heaven-swap")]
pub mod heaven;

#[cfg(feature = "aldrin-swap")]
pub mod aldrin;

#[cfg(feature = "aldrin_v2-swap")]
pub mod aldrin_v2;

#[cfg(feature = "futarchy-swap")]
pub mod futarchy;

#[cfg(feature = "gamma-swap")]
pub mod gamma;
