use std::mem;

pub fn transform_memory_content_as_ulong_memory(byte_memory: &[u8]) -> Vec<u64> {
    let size = byte_memory.len() / mem::size_of::<u64>();
    let mut ulong_vec = Vec::with_capacity(size);

    for i in 0..size {
        let mut array = [0u8; 8];
        array.copy_from_slice(&byte_memory[i * 8..(i + 1) * 8]);
        ulong_vec.push(u64::from_ne_bytes(array));
    }

    ulong_vec
}