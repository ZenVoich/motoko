use std::array::from_fn;

use crate::memory::{initialize_test_memory, reset_test_memory};

use motoko_rts::continuation_table::{
    continuation_count, recall_continuation, remember_continuation,
};
use motoko_rts::memory::alloc_blob;
use motoko_rts::types::{Bytes, Value};

pub unsafe fn test() {
    println!("Testing continuation table ...");

    assert_eq!(continuation_count(), 0);

    const N: usize = 2000; // >256, to exercise `double_continuation_table`

    let mut heap = initialize_test_memory();

    let pointers: [Value; N] = from_fn(|_| alloc_blob(&mut heap, Bytes(0)));

    let mut references: [u32; N] = [0; N];
    for i in 0..N {
        references[i] = remember_continuation(&mut heap, pointers[i]);
        assert_eq!(continuation_count(), (i + 1) as u32);
    }

    for i in 0..N / 2 {
        let c = recall_continuation(references[i]);
        assert_eq!(c.get_ptr(), pointers[i].get_ptr());
        assert_eq!(continuation_count(), (N - i - 1) as u32);
    }

    for i in 0..N / 2 {
        references[i] = remember_continuation(&mut heap, pointers[i]);
        assert_eq!(continuation_count(), (N / 2 + i + 1) as u32);
    }

    for i in (0..N).rev() {
        assert_eq!(
            recall_continuation(references[i]).get_ptr(),
            pointers[i].get_ptr(),
        );
        assert_eq!(continuation_count(), i as u32);
    }

    reset_test_memory();
}
