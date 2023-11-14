#![feature(proc_macro_hygiene)]

mod bigint;
mod bitrel;
mod continuation_table;
mod crc32;
mod gc;
mod leb128;
mod memory;
mod principal_id;
mod stabilization;
mod stream;
mod text;
mod utf8;

use motoko_rts::types::{read64, write64, Bytes};

fn main() {
    if std::mem::size_of::<usize>() != 4 {
        println!("Motoko RTS only works on 32-bit architectures");
        std::process::exit(1);
    }

    unsafe {
        test_read_write_64_bit();
        bigint::test();
        bitrel::test();
        continuation_table::test();
        crc32::test();
        gc::test();
        leb128::test();
        principal_id::test();
        stabilization::test();
        stream::test();
        text::test();
        utf8::test();
    }
}

fn test_read_write_64_bit() {
    println!("Testing 64-bit read-write");
    const TEST_VALUE: u64 = 0x1234_5678_9abc_def0;
    let mut lower = 0u32;
    let mut upper = 0u32;
    write64(&mut lower, &mut upper, TEST_VALUE);
    assert_eq!(lower, 0x9abc_def0);
    assert_eq!(upper, 0x1234_5678);
    assert_eq!(read64(lower, upper), TEST_VALUE);
}

// Called by the RTS to panic
#[no_mangle]
extern "C" fn rts_trap(ptr: *const u8, len: Bytes<u32>) -> ! {
    let msg = unsafe { std::slice::from_raw_parts(ptr, len.as_usize()) };
    match core::str::from_utf8(msg) {
        Err(err) => panic!(
            "rts_trap_with called with non-UTF8 string (error={:?}, string={:?})",
            err, msg
        ),
        Ok(str) => panic!("rts_trap_with: {:?}", str),
    }
}

// Called by RTS BigInt functions to panic. Normally generated by the compiler
#[no_mangle]
extern "C" fn bigint_trap() -> ! {
    panic!("bigint_trap called");
}

// Called by the RTS for debug prints
#[no_mangle]
unsafe extern "C" fn print_ptr(ptr: usize, len: u32) {
    let str: &[u8] = core::slice::from_raw_parts(ptr as *const u8, len as usize);
    println!("[RTS] {}", String::from_utf8_lossy(str));
}
