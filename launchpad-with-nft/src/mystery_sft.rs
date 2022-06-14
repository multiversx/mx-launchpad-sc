elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use elrond_wasm::{elrond_codec::Empty, hex_literal};

pub const NFT_AMOUNT: u32 = 1;
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
}

#[derive(TopEncode, TopDecode)]
pub struct SftSetupSteps {
    pub issued_token: bool,
    pub created_initial_tokens: bool,
    pub set_transfer_role: bool,
}

// temporary until role name is fixed
static ESDT_TRANSFER_ROLE_NAME: &[u8] = b"ESDTTransferRole";
static ESDT_SYSTEM_SC_ADDRESS_ARRAY: [u8; 32] =
    hex_literal::hex!("000000000000000000010000000000000000000000000000000000000002ffff");
mod esdt_system_sc_proxy {
    elrond_wasm::imports!();

    #[elrond_wasm::proxy]
    pub trait EsdtSystemScProxy {
        #[endpoint(setSpecialRole)]
        fn set_special_roles(
            &self,
            token_id: TokenIdentifier,
            address: ManagedAddress,
            role_name: ManagedBuffer,
        );
    }
}

#[elrond_wasm::module]
pub trait MysterySftModule:
    elrond_wasm_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + launchpad_common::permissions::PermissionsModule
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

        let steps_mapper = self.sft_setup_steps();
        let mut steps = steps_mapper.get();
        require!(
            !steps.created_initial_tokens,
            "Initial SFTs already created"
        );

        let token_id = self.mystery_sft().get_token_id();
        let initial_amount = BigUint::from(NFT_AMOUNT);
        let api = self.send();
        for sft_name in SFT_NAMES {
            api.esdt_nft_create_compact_named(
                &token_id,
                &initial_amount,
                &ManagedBuffer::new_from_bytes(sft_name),
                &Empty,
            );
        }

        steps.issued_token = true;
        steps.created_initial_tokens = true;
        steps_mapper.set(&steps);
    }

    #[endpoint(setTransferRole)]
    fn set_transfer_role(&self, opt_addr_to_set: OptionalValue<ManagedAddress>) {
        self.require_extended_permissions();

        let addr = match opt_addr_to_set {
            OptionalValue::Some(addr) => addr,
            OptionalValue::None => {
                self.sft_setup_steps()
                    .update(|steps| steps.set_transfer_role = true);

                self.blockchain().get_sc_address()
            }
        };

        let token_id = self.mystery_sft().get_token_id();
        self.system_sc_proxy(ESDT_SYSTEM_SC_ADDRESS_ARRAY.into())
            .set_special_roles(
                token_id,
                addr,
                ManagedBuffer::new_from_bytes(ESDT_TRANSFER_ROLE_NAME),
            )
            .async_call()
            .call_and_exit();
    }

    fn require_all_sft_setup_steps_complete(&self) {
        let steps = self.sft_setup_steps().get();
        require!(
            steps.issued_token && steps.created_initial_tokens && steps.set_transfer_role,
            "SFT setup not complete"
        );
    }

    #[proxy]
    fn system_sc_proxy(&self, sc_address: ManagedAddress)
        -> esdt_system_sc_proxy::Proxy<Self::Api>;

    #[storage_mapper("mysterySftTokenId")]
    fn mystery_sft(&self) -> NonFungibleTokenMapper<Self::Api>;

    #[storage_mapper("sftSetupSteps")]
    fn sft_setup_steps(&self) -> SingleValueMapper<SftSetupSteps>;
}
