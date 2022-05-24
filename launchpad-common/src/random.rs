elrond_wasm::imports!();

const USIZE_BYTES: usize = 4;
pub const HASH_LEN: usize = 32;
static FAILED_COPY_ERR_MSG: &[u8] = b"Failed copy to/from managed buffer";

pub type Hash<M> = ManagedByteArray<M, HASH_LEN>;

pub struct Random<M: ManagedTypeApi + CryptoApi> {
    pub seed: ManagedBuffer<M>,
    pub index: usize,
}

impl<M: ManagedTypeApi + CryptoApi> Random<M> {
    pub fn new() -> Self {
        Self {
            seed: ManagedBuffer::new_random(HASH_LEN),
            index: 0,
        }
    }

    pub fn from_hash(hash: Hash<M>, index: usize) -> Self {
        Self {
            seed: ManagedBuffer::from_raw_handle(hash.get_raw_handle()),
            index,
        }
    }

    pub fn next_usize(&mut self) -> usize {
        if self.index + USIZE_BYTES > HASH_LEN {
            self.hash_seed();
        }

        let raw_buffer = match self.seed.copy_slice(self.index, USIZE_BYTES) {
            Some(buffer) => buffer,
            None => M::error_api_impl().signal_error(FAILED_COPY_ERR_MSG),
        };
        let rand = usize::top_decode(raw_buffer).unwrap_or_default();

        self.index += USIZE_BYTES;

        rand
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

    fn hash_seed(&mut self) {
        let mut prev_seed_bytes = [0u8; HASH_LEN];
        let prev_seed_slice = self.seed.load_to_byte_array(&mut prev_seed_bytes);
        let new_seed_bytes = M::crypto_api_impl().sha256_legacy(&prev_seed_slice);

        self.seed.overwrite(&new_seed_bytes[..]);
        self.index = 0;
    }
}
