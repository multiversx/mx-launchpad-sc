elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait NftWinnersSelectionModule {
    #[endpoint(selectNftWinners)]
    fn select_nft_winners(&self) -> OperationCompletionStatus {
        OperationCompletionStatus::Completed
    }
}
