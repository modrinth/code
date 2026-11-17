use std::{ffi::c_char, ptr::NonNull, slice, str};

use eyre::{Context, ContextCompat, Result, ensure};
use libc::size_t;

/// A borrowed UTF-8 string. `len` counts bytes; no trailing NUL is required.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct ModrinthSandboxString {
    pub ptr: *const c_char,
    pub len: size_t,
}

/// An optional borrowed UTF-8 string. `len` counts bytes; no trailing NUL is
/// required. A null pointer with zero length represents an absent string.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct ModrinthSandboxStringOption {
    pub ptr: *const c_char,
    pub len: size_t,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ModrinthSandboxStringSlice {
    pub ptr: *const ModrinthSandboxString,
    pub len: size_t,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ModrinthSandboxStringPairsSlice {
    pub key_ptr: *const ModrinthSandboxString,
    pub key_len: size_t,
    pub value_ptr: *const ModrinthSandboxString,
    pub value_len: size_t,
}

impl ModrinthSandboxString {
    /// Copies this borrowed UTF-8 string into Rust-owned memory.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null. If `len` is nonzero, `ptr` must be valid for
    /// reading `len` bytes for the duration of this call.
    pub unsafe fn try_to_owned(self) -> Result<Box<str>> {
        let ptr = NonNull::new(self.ptr.cast_mut())
            .wrap_err("expected a non-null string")?;

        // SAFETY: The caller guarantees that `ptr` is valid for reading
        // `len` bytes. The pointer was checked for null above, and `u8` has
        // alignment 1, including for an empty slice.
        let bytes = unsafe {
            slice::from_raw_parts(ptr.as_ptr().cast::<u8>(), self.len)
        };

        let value = str::from_utf8(bytes).wrap_err("string is not UTF-8")?;
        Ok(value.into())
    }
}

impl ModrinthSandboxStringOption {
    /// Copies this optional borrowed UTF-8 string into Rust-owned memory.
    ///
    /// A null pointer with a zero length represents [`None`]. A non-null
    /// pointer represents [`Some`], including when the length is zero.
    ///
    /// # Safety
    ///
    /// If `ptr` is non-null and `len` is nonzero, `ptr` must be valid for
    /// reading `len` bytes for the duration of this call.
    pub unsafe fn try_to_owned(self) -> Result<Option<Box<str>>> {
        if self.ptr.is_null() {
            ensure!(
                self.len == 0,
                "optional string has a null pointer with a nonzero length"
            );
            return Ok(None);
        }

        // SAFETY: The caller guarantees that a non-null pointer is valid for
        // reading `len` bytes. `ModrinthSandboxString` has the same contract.
        let value = unsafe {
            ModrinthSandboxString {
                ptr: self.ptr,
                len: self.len,
            }
            .try_to_owned()
        }?;
        Ok(Some(value))
    }
}

impl ModrinthSandboxStringSlice {
    /// Copies this borrowed slice and all of its UTF-8 strings into
    /// Rust-owned memory.
    ///
    /// A null pointer with a zero length represents an empty slice.
    ///
    /// # Safety
    ///
    /// If `len` is nonzero, `ptr` must be properly aligned and valid for
    /// reading `len` [`ModrinthSandboxString`] values. Every string in the
    /// slice must uphold [`ModrinthSandboxString::try_to_owned`]'s safety
    /// contract.
    pub unsafe fn try_to_owned(self) -> Result<Vec<Box<str>>> {
        if self.len == 0 {
            return Ok(Vec::new());
        }

        let ptr = NonNull::new(self.ptr.cast_mut())
            .wrap_err("expected a non-null string slice")?;

        // SAFETY: The caller guarantees that `ptr` is properly aligned and
        // valid for reading `len` string descriptors. The zero-length case was
        // handled above and the pointer was checked for null.
        let items = unsafe { slice::from_raw_parts(ptr.as_ptr(), self.len) };
        let mut strings = Vec::with_capacity(items.len());
        for (index, item) in items.iter().copied().enumerate() {
            // SAFETY: The caller guarantees that every string descriptor in
            // the slice upholds `ModrinthSandboxString`'s safety contract.
            let string = unsafe { item.try_to_owned() }
                .wrap_err_with(|| format!("reading string at index {index}"))?;
            strings.push(string);
        }

        Ok(strings)
    }
}

impl ModrinthSandboxStringPairsSlice {
    /// Copies these parallel key and value slices into Rust-owned pairs.
    ///
    /// # Safety
    ///
    /// If the corresponding length is nonzero, `key_ptr` and `value_ptr` must
    /// each be properly aligned and valid for reading that many
    /// [`ModrinthSandboxString`] values. Every string in both slices must
    /// uphold [`ModrinthSandboxString::try_to_owned`]'s safety contract.
    pub unsafe fn try_to_owned(self) -> Result<Vec<(String, String)>> {
        ensure!(
            self.key_len == self.value_len,
            "string pair slices have different lengths: {} keys and {} values",
            self.key_len,
            self.value_len
        );

        // SAFETY: The caller guarantees that the key slice and each of its
        // string descriptors uphold their respective safety contracts.
        let keys = unsafe {
            ModrinthSandboxStringSlice {
                ptr: self.key_ptr,
                len: self.key_len,
            }
            .try_to_owned()
        }
        .wrap_err("reading string pair keys")?;

        // SAFETY: The caller guarantees that the value slice and each of its
        // string descriptors uphold their respective safety contracts.
        let values = unsafe {
            ModrinthSandboxStringSlice {
                ptr: self.value_ptr,
                len: self.value_len,
            }
            .try_to_owned()
        }
        .wrap_err("reading string pair values")?;

        Ok(keys
            .into_iter()
            .zip(values)
            .map(|(key, value)| (String::from(key), String::from(value)))
            .collect())
    }
}
