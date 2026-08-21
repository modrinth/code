use std::{env, path::Path};

const NVIDIA_EXPLICIT_SYNC_ENV: &str = "__NV_DISABLE_EXPLICIT_SYNC";

pub fn configure_webkit() {
    if should_disable_nvidia_explicit_sync(
        env::var_os("WAYLAND_DISPLAY").is_some(),
        Path::new("/sys/module/nvidia").exists(),
        env::var_os(NVIDIA_EXPLICIT_SYNC_ENV).is_some(),
    ) {
        unsafe {
            env::set_var(NVIDIA_EXPLICIT_SYNC_ENV, "1");
        }
    }
}

fn should_disable_nvidia_explicit_sync(
    wayland: bool,
    nvidia: bool,
    explicitly_configured: bool,
) -> bool {
    wayland && nvidia && !explicitly_configured
}

#[cfg(test)]
mod tests {
    use super::should_disable_nvidia_explicit_sync;

    #[test]
    fn disables_explicit_sync_for_nvidia_wayland() {
        assert!(should_disable_nvidia_explicit_sync(true, true, false));
    }

    #[test]
    fn leaves_other_graphics_stacks_unchanged() {
        assert!(!should_disable_nvidia_explicit_sync(false, true, false));
        assert!(!should_disable_nvidia_explicit_sync(true, false, false));
    }

    #[test]
    fn preserves_explicit_configuration() {
        assert!(!should_disable_nvidia_explicit_sync(true, true, true));
    }
}
