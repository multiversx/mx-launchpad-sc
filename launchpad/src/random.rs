elrond_wasm::imports!();

const USIZE_BYTES: usize = 4;
pub const HASH_LEN: usize = 32;

pub type Hash = [u8; HASH_LEN];
type Handle = i32;

extern "C" {
    fn sha256(dataOffset: *const u8, length: i32, resultOffset: *mut u8) -> i32;

    fn mBufferNew() -> i32;
    fn mBufferNewFromBytes(byte_ptr: *const u8, byte_len: i32) -> i32;

    fn mBufferGetBytes(mBufferHandle: i32, resultOffset: *mut u8) -> i32;
    fn mBufferGetByteSlice(
        sourceHandle: i32,
        startingPosition: i32,
        sliceLength: i32,
        resultOffset: *mut u8,
    ) -> i32;

    fn mBufferSetBytes(mBufferHandle: i32, byte_ptr: *const u8, byte_len: i32) -> i32;
    fn mBufferSetRandom(destinationHandle: i32, length: i32) -> i32;
}

pub struct Random {
    pub seed_handle: Handle,
    pub index: usize,
}

impl Random {
    pub fn new() -> Self {
        unsafe {
            let seed_handle = mBufferNew();
            mBufferSetRandom(seed_handle, HASH_LEN as i32);

            Self {
                seed_handle,
                index: 0,
            }
        }
    }

    pub fn from_hash(hash: Hash, index: usize) -> Self {
        unsafe {
            let seed_handle = mBufferNewFromBytes(hash.as_ptr(), HASH_LEN as i32);

            Self { seed_handle, index }
        }
    }

    pub fn next_usize(&mut self) -> usize {
        unsafe {
            if self.index + USIZE_BYTES > HASH_LEN {
                self.hash_seed();
            }

            let mut raw_bytes = [0u8; USIZE_BYTES];
            mBufferGetByteSlice(
                self.seed_handle,
                self.index as i32,
                USIZE_BYTES as i32,
                raw_bytes.as_mut_ptr(),
            );

            let rand = usize::top_decode(&raw_bytes[..]).unwrap_or_default();

            self.index += USIZE_BYTES;

            rand
        }
    }

    /// Range is [min, max)
    pub fn next_usize_in_range(&mut self, min: usize, max: usize) -> usize {
        let rand = self.next_usize();

        if min >= max {
            min
        } else {
            min + rand % (max - min)
        }
    }
}

impl Random {
    fn hash_seed(&mut self) {
        unsafe {
            let mut seed_bytes = [0u8; HASH_LEN];
            let mut hashed_result = [0u8; HASH_LEN];

            mBufferGetBytes(self.seed_handle, seed_bytes.as_mut_ptr());
            sha256(
                seed_bytes.as_ptr(),
                HASH_LEN as i32,
                hashed_result.as_mut_ptr(),
            );

            mBufferSetBytes(self.seed_handle, hashed_result.as_ptr(), HASH_LEN as i32);
            self.index = 0;
        }
    }
}
