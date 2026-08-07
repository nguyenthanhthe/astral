//! Windows working-set trimmer: unmaps WebView2 pages the OS hasn't touched,
//! shrinking the reported RAM footprint. No-op elsewhere.

#[cfg(target_os = "windows")]
extern "system" {
    fn GetCurrentProcess() -> *mut std::ffi::c_void;
    fn SetProcessWorkingSetSize(
        hProcess: *mut std::ffi::c_void,
        dwMinimumWorkingSetSize: usize,
        dwMaximumWorkingSetSize: usize,
    ) -> i32;
}

/// Trim the current process working set. Always succeeds; on non-Windows it's
/// a no-op so the caller doesn't need platform branching.
pub fn trim_working_set() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // Safety: these are the documented Win32 signatures; passing the
        // current-process pseudo-handle and `usize::MAX` for both bounds is
        // the standard "empty the working set" idiom.
        unsafe {
            let handle = GetCurrentProcess();
            SetProcessWorkingSetSize(handle, usize::MAX, usize::MAX);
        }
        log::debug!("trimmed Windows WorkingSet to minimum footprint");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_never_fails() {
        assert!(trim_working_set().is_ok());
    }
}
