//! This module implements a simple buffer to be used by the compiler (in generated code)

use crate::idl_trap_with;

#[repr(packed)]
pub struct GenBuf<A> {
    /// Pointer into the buffer
    pub ptr: A,
    /// Pointer to the end of the buffer
    pub end: A,
}

pub type Buf = GenBuf<*mut u8>;

impl Buf {
    #[cfg(feature = "ic")]
    pub(crate) unsafe fn advance(self: *mut Self, n: u32) {
        advance(self, n)
    }
}

/// Read a single byte
pub(crate) unsafe fn read_byte(buf: *mut Buf) -> u8 {
    if (*buf).ptr >= (*buf).end {
        idl_trap_with("byte read out of buffer");
    }

    let byte = *(*buf).ptr;
    (*buf).ptr = (*buf).ptr.add(1);

    byte
}

#[cfg(feature = "ic")]
/// Read a little-endian word
pub(crate) unsafe fn read_word(buf: *mut Buf) -> u32 {
    if (*buf).ptr.add(3) >= (*buf).end {
        idl_trap_with("word read out of buffer");
    }

    let p = (*buf).ptr;
    let word = u32::from_le_bytes([*p, *p.add(1), *p.add(2), *p.add(3)]);

    (*buf).ptr = (*buf).ptr.add(4);

    word
}

#[cfg(feature = "ic")]
unsafe fn advance(buf: *mut Buf, n: u32) {
    if (*buf).ptr.add(n as usize) > (*buf).end {
        idl_trap_with("advance out of buffer");
    }

    (*buf).ptr = (*buf).ptr.add(n as usize);
}

/// Can also be used for sleb
#[cfg(feature = "ic")]
#[no_mangle]
pub(crate) unsafe extern "C" fn skip_leb128(buf: *mut Buf) {
    loop {
        let byte = read_byte(buf);
        if byte & 0b1000_0000 == 0 {
            break;
        }
    }
}

/// Check if the potentially incomplete buffer holds the requested number of bytes at its prefix.
#[cfg(feature = "ic")]
#[no_mangle]
pub unsafe extern "C" fn check_prefix(buf: *mut Buf, required: usize) -> bool {
    (*buf).end.sub_ptr((*buf).ptr) >= required
}

/// Check if the potentially incomplete buffer holds a valid (s)leb128 at its prefix.
/// Note: This is a byte-wise loop, doing unaligned 64-bit chunks (where possible) could
///       speed up things.
#[cfg(feature = "ic")]
#[no_mangle]
pub unsafe extern "C" fn check_leb128_prefix(buf: *mut Buf) -> bool {
    let (mut ptr, end) = ((*buf).ptr, (*buf).end);
    while ptr != end {
        let byte = *ptr;
        if byte & 0b1000_0000 == 0 {
            return true;
        }
        ptr = ptr.add(1);
    }
    false
}

/// Move remaining buffer contents to `base` and use `fill` to load more content
/// up to `(*buf).end`.
#[cfg(feature = "ic")]
pub unsafe fn refill<F: FnOnce(*mut u8, u64) -> ()>(buf: *mut Buf, base: *mut u8, fill: F) {
    let len = (*buf).end.sub_ptr((*buf).ptr);
    libc::memcpy(base as *mut _, (*buf).ptr as *const _, len);
    let bytes = (*buf).end.sub_ptr(base) - len;
    fill(base.add(len), bytes as u64);
    (*buf).ptr = base;
}

/// Set up a descriptor for an area of stable memory to slurp data from.
type StableBuf = GenBuf<u64>;

impl StableBuf {
    #[cfg(feature = "ic")]
    pub(crate) unsafe fn advance(self: *mut Self, n: u32) {
        if (*self).ptr + n as u64 > (*self).end {
            idl_trap_with("advance out of stable buffer");
        }

        (*self).ptr += n as u64;
    }
}
