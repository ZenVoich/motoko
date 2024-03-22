pub const PAGE_SIZE: u64 = 64 * 1024;

extern "C" {
    // physical ic0_stable64 operations re-exported by moc
    pub fn ic0_stable64_write(offset: u64, src: u64, size: u64);
    pub fn ic0_stable64_read(dst: u64, offset: u64, size: u64);
    #[cfg(feature = "ic")]
    pub fn ic0_stable64_size() -> u64; // physical memory size
                                       // (virtual) stable_mem operations implemented by moc
    #[cfg(feature = "ic")]
    pub fn moc_stable_mem_get_version() -> usize;
    #[cfg(feature = "ic")]
    pub fn moc_stable_mem_set_version(version: usize);
    pub fn moc_stable_mem_get_size() -> u64; // virtual memory size
    #[cfg(feature = "ic")]
    pub fn moc_stable_mem_set_size(pages: u64); // virtual memory size
    pub fn moc_stable_mem_grow(additional_pages: u64) -> u64;
}

#[cfg(feature = "ic")]
pub fn get_version() -> usize {
    unsafe { moc_stable_mem_get_version() }
}

#[cfg(feature = "ic")]
pub fn set_version(version: usize) {
    unsafe { moc_stable_mem_set_version(version) }
}

pub fn size() -> u64 {
    // SAFETY: This is safe because of the ic0 api guarantees.
    unsafe { moc_stable_mem_get_size() }
}

pub fn grow(pages: u64) -> u64 {
    // SAFETY: This is safe because of the ic0 api guarantees.
    unsafe { moc_stable_mem_grow(pages) }
}

#[cfg(feature = "ic")]
pub fn read(offset: u64, dst: &mut [u8]) {
    // SAFETY: This is safe because of the ic0 api guarantees.
    unsafe { ic0_stable64_read(dst.as_ptr() as u64, offset, dst.len() as u64) }
}

#[cfg(feature = "ic")]
pub fn write(offset: u64, src: &[u8]) {
    // SAFETY: This is safe because of the ic0 api guarantees.
    unsafe { ic0_stable64_write(offset, src.as_ptr() as u64, src.len() as u64) }
}

// Little endian.
#[cfg(feature = "ic")]
pub fn read_u8(offset: u64) -> u8 {
    let mut res: [u8; 1] = [0; 1];
    read(offset, &mut res);
    core::primitive::u8::from_le_bytes(res)
}

// Little endian.
#[cfg(feature = "ic")]
pub fn write_u8(offset: u64, value: u8) {
    write(offset, &core::primitive::u8::to_le_bytes(value));
}

// Little endian.
#[cfg(feature = "ic")]
pub fn read_u16(offset: u64) -> u16 {
    let mut res: [u8; 2] = [0; 2];
    read(offset, &mut res);
    core::primitive::u16::from_le_bytes(res)
}

// Little endian.
#[cfg(feature = "ic")]
pub fn write_u16(offset: u64, value: u16) {
    write(offset, &core::primitive::u16::to_le_bytes(value));
}

// Little endian.
#[cfg(feature = "ic")]
pub fn read_u32(offset: u64) -> u32 {
    let mut res: [u8; 4] = [0; 4];
    read(offset, &mut res);
    core::primitive::u32::from_le_bytes(res)
}

// Little endian.
#[cfg(feature = "ic")]
pub fn write_u32(offset: u64, value: u32) {
    write(offset, &core::primitive::u32::to_le_bytes(value));
}

// Little endian.
#[cfg(feature = "ic")]
pub fn read_u64(offset: u64) -> u64 {
    let mut res: [u8; 8] = [0; 8];
    read(offset, &mut res);
    core::primitive::u64::from_le_bytes(res)
}

// Little endian.
#[cfg(feature = "ic")]
pub fn write_u64(offset: u64, n: u64) {
    write(offset, &core::primitive::u64::to_le_bytes(n));
}

#[cfg(feature = "ic")]
#[no_mangle]
pub extern "C" fn read_stable_memory_version() -> u32 {
    let physical_pages = unsafe { ic0_stable64_size() };
    if physical_pages == 0 {
        // No stable memory -> Legacy version 0.
        return 0;
    }
    if read_u32(0) != 0 {
        // Old stabilization with no experimental stable memory and no regions.
        // It stores non-zero marker at address 0 -> Legacy version 0.
        return 0;
    }
    // Note: Do not use `types::size_of()` as it rounds to 64-bit words.
    let address = physical_pages * PAGE_SIZE - core::mem::size_of::<u32>() as u64;
    read_u32(address)
}
