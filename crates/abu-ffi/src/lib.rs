//! `abu-ffi` (`libabuclock`): Cross-Language FFI & UniFFI SDK Bridge.

pub use abu_clock_core::ABU_CLOCK_VERSION;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffi_version_matches_core() {
        assert!(ABU_CLOCK_VERSION.starts_with("v0.0.0."));
    }
}
