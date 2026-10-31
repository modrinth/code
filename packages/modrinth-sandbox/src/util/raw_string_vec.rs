use std::{
    ffi::{CString, OsString},
    os::unix::ffi::OsStringExt,
};

#[derive(Debug)]
pub struct RawStringVec(Vec<*mut libc::c_char>);

impl RawStringVec {
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity + 1))
    }

    pub fn push_c(&mut self, string: CString) {
        self.0.push(string.into_raw());
    }

    pub fn push_os(&mut self, string: OsString) -> eyre::Result<()> {
        self.push_c(CString::new(string.into_vec())?);
        Ok(())
    }

    pub fn into_null_terminated_ptr(mut self) -> *const *mut libc::c_char {
        assert!(self.0.last().unwrap().is_null());
        std::mem::take(&mut self.0).into_raw_parts().0
    }

    pub fn as_null_terminated_ptr(&self) -> *const *mut libc::c_char {
        assert!(self.0.last().unwrap().is_null());
        self.0.as_ptr()
    }

    pub fn ensure_null_terminated(&mut self) {
        if let Some(last) = self.0.last()
            && last.is_null()
        {
            return;
        }
        self.0.push(std::ptr::null_mut());
    }
}

unsafe impl Send for RawStringVec {}
unsafe impl Sync for RawStringVec {}

impl Drop for RawStringVec {
    fn drop(&mut self) {
        for ptr in self.0.drain(..) {
            if !ptr.is_null() {
                drop(unsafe { CString::from_raw(ptr) });
            }
        }
    }
}
