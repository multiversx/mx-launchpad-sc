multiversx_sc::derive_imports!();

use multiversx_sc::{
    api::ManagedTypeApi,
    codec::{TopDecode, TopEncode},
    contract_base::{CallableContract, ContractBase},
    types::{EgldOrEsdtTokenIdentifier, EsdtTokenPayment, ManagedAddress},
};
use multiversx_sc_scenario::{managed_token_id, tx_mock::TxContextStack, DebugApi};

use super::{LOCKED_TOKEN_ID, LOCK_FN_NAME};

#[derive(Clone)]
pub struct SimpleLockMock {}

impl ContractBase for SimpleLockMock {
    type Api = DebugApi;
}

impl CallableContract for SimpleLockMock {
    fn call(&self, fn_name: &[u8]) -> bool {
        if fn_name != LOCK_FN_NAME {
            return false;
        }

        self.call_lock_tokens();

        true
    }

    fn clone_obj(&self) -> Box<dyn CallableContract> {
        Box::new(self.clone())
    }
}

impl SimpleLockMock {
    pub fn new() -> Self {
        SimpleLockMock {}
    }

    fn call_lock_tokens(&self) {
        let api = TxContextStack::static_peek();
        let args = api.input_ref().args.clone();
        if args.len() != 2 {
            panic!("Invalid args");
        }

        let unlock_epoch = u64::top_decode(args[0].clone()).unwrap();
        let dest_addr = ManagedAddress::<DebugApi>::top_decode(args[1].clone()).unwrap();

        let payment = self.call_value().egld_or_single_esdt();
        let current_epoch = self.blockchain().get_block_epoch();
        if current_epoch >= unlock_epoch {
            self.send().direct(
                &dest_addr,
                &payment.token_identifier,
                payment.token_nonce,
                &payment.amount,
            );

            let mut result = Vec::new();
            payment.top_encode(&mut result).unwrap();
            api.tx_result_cell.borrow_mut().result_values.push(result);

            return;
        }

        let attributes = LockedTokenAttributes {
            original_token_id: payment.token_identifier.clone(),
            original_token_nonce: payment.token_nonce,
            unlock_epoch,
        };
        let locked_token_nonce = self.send().esdt_nft_create_compact_named(
            &managed_token_id!(LOCKED_TOKEN_ID),
            &payment.amount,
            &payment.token_identifier.clone().into_name(),
            &attributes,
        );
        self.send().direct_esdt(
            &dest_addr,
            &managed_token_id!(LOCKED_TOKEN_ID),
            locked_token_nonce,
            &payment.amount,
        );

        let output_payment = EsdtTokenPayment::new(
            managed_token_id!(LOCKED_TOKEN_ID),
            locked_token_nonce,
            payment.amount,
        );
        let mut result = Vec::new();
        output_payment.top_encode(&mut result).unwrap();
        api.tx_result_cell.borrow_mut().result_values.push(result);
    }
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedDecode, NestedEncode, PartialEq, Debug)]
pub struct LockedTokenAttributes<M: ManagedTypeApi> {
    pub original_token_id: EgldOrEsdtTokenIdentifier<M>,
    pub original_token_nonce: u64,
    pub unlock_epoch: u64,
}
