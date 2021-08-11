elrond_wasm::imports!();

const BLOCK_RAND_SEED_LEN: usize = 48;
pub type BlockRandomSeed = Box<[u8; BLOCK_RAND_SEED_LEN]>;

const U_32_BYTES: usize = 4;

struct Random<CA: CryptoApi> {
    api: CA,
    seed: H256,
    index: usize,
}

impl<CA: CryptoApi> Random<CA> {
    pub fn new(
        api: CA,
        prev_block_seed: BlockRandomSeed,
        current_block_seed: BlockRandomSeed,
    ) -> Self {
        let mut summed_seeds = BlockRandomSeed::new([0u8; BLOCK_RAND_SEED_LEN]);
        for i in 0..BLOCK_RAND_SEED_LEN {
            summed_seeds[i] = prev_block_seed[i].wrapping_add(current_block_seed[i]);
        }

        Self {
            seed: api.sha256(&summed_seeds[..]),
            index: 0,
            api,
        }
    }

    pub fn next_u32(&mut self) -> u32 {
        if self.index + U_32_BYTES > H256::len_bytes() {
            self.hash_seed();
        }

        let bytes = &self.seed.as_bytes()[self.index..(self.index + U_32_BYTES)];
        let result = u32::top_decode(bytes).unwrap_or_default();

        self.index += U_32_BYTES;

        result
    }
}

impl<CA: CryptoApi> Random<CA> {
    fn hash_seed(&mut self) {
        self.seed = self.api.sha256(&self.seed.as_bytes()[..]);
        self.index = 0;
    }
}
