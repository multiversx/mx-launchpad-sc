elrond_wasm::imports!();

const USIZE_BYTES: usize = 4;
const HASH_LEN: usize = 32;

pub type Hash = [u8; HASH_LEN];

extern "C" {
    fn sha256(dataOffset: *const u8, length: i32, resultOffset: *mut u8) -> i32;
}

pub struct Random<CA: CryptoApi> {
    api: CA,
    pub seed: Hash,
    pub index: usize,
}

impl<CA: CryptoApi> Random<CA> {
    #[allow(clippy::boxed_local)]
    pub fn from_seeds(api: CA) -> Self {
        let summed_seeds = [0u8; 48];
        let mut seed = [0u8; HASH_LEN];
        unsafe {
            sha256(summed_seeds.as_ptr(), 48, seed.as_mut_ptr());
        }

        Self {
            seed,
            index: 0,
            api,
        }
    }

    pub fn from_hash(api: CA, hash: Hash, index: usize) -> Self {
        Self {
            seed: hash,
            index,
            api,
        }
    }

    pub fn next_usize(&mut self) -> usize {
        if self.index + USIZE_BYTES > HASH_LEN {
            unsafe {
                self.hash_seed();
            }
        }

        let bytes = &self.seed[self.index..(self.index + USIZE_BYTES)];
        let rand = usize::top_decode(bytes).unwrap_or_default();

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
}

impl<CA: CryptoApi> Random<CA> {
    unsafe fn hash_seed(&mut self) {
        let mut hashed_result = [0u8; HASH_LEN];
        sha256(
            self.seed.as_ptr(),
            HASH_LEN as i32,
            hashed_result.as_mut_ptr(),
        );

        self.seed = hashed_result;
        self.index = 0;
    }
}
