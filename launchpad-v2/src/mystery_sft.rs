elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use elrond_wasm::elrond_codec::Empty;

const SFT_INITIAL_AMOUNT: u32 = 1;
static SFT_NAMES: &[&[u8]] = &[b"Confirmed Won", b"Confirmed Lost", b"Not Confirmed"];

pub enum MysterySftTypes {
    ConfirmedWon,
    ConfirmedLost,
    NotConfirmed,
}

impl MysterySftTypes {
    pub fn as_nonce(&self) -> u64 {
        match *self {
            MysterySftTypes::ConfirmedWon => 1,
            MysterySftTypes::ConfirmedLost => 2,
            MysterySftTypes::NotConfirmed => 3,
        }
    }

    pub const fn total_instances() -> usize {
        3
    }
}

#[elrond_wasm::module]
pub trait MysterySftModule:
    elrond_wasm_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + crate::permissions::PermissionsModule
{
    #[payable("*")]
    #[endpoint(issueMysterySft)]
    fn issue_mystery_sft(&self, token_display_name: ManagedBuffer, token_ticker: ManagedBuffer) {
        self.require_extended_permissions();

        let issue_cost = self.call_value().egld_value();
        self.mystery_sft().issue_and_set_all_roles(
            EsdtTokenType::SemiFungible,
            issue_cost,
            token_display_name,
            token_ticker,
            0,
            None,
        );
    }

    #[endpoint(createInitialSfts)]
    fn create_initial_sfts(&self) {
        self.require_extended_permissions();

        let token_id = self.mystery_sft().get_token_id();
        let current_balance = self.blockchain().get_sc_balance(&token_id, 1);
        require!(current_balance == 0, "Initial SFTs already created");

        let initial_amount = BigUint::from(SFT_INITIAL_AMOUNT);
        let api = self.send();
        for i in 0..MysterySftTypes::total_instances() {
            api.esdt_nft_create_compact_named(
                &token_id,
                &initial_amount,
                &ManagedBuffer::new_from_bytes(SFT_NAMES[i]),
                &Empty,
            );
        }
    }

    #[endpoint]
    fn set_transfer_role(&self, opt_addr_to_set: OptionalValue<ManagedAddress>) {
        self.require_extended_permissions();

        let addr = match opt_addr_to_set {
            OptionalValue::Some(addr) => addr,
            OptionalValue::None => self.blockchain().get_sc_address(),
        };
        self.mystery_sft()
            .set_local_roles_for_address(&addr, &[EsdtLocalRole::Transfer], None);
    }

    #[storage_mapper("mysterySftTokenId")]
    fn mystery_sft(&self) -> NonFungibleTokenMapper<Self::Api>;
}
