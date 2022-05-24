elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait PermissionsModule {
    #[only_owner]
    #[endpoint(setSupportAddress)]
    fn add_support_address(&self, address: ManagedAddress) {
        self.support_address().set(&address);
    }

    fn require_extended_permissions(&self) {
        let caller = self.blockchain().get_caller();
        let owner = self.blockchain().get_owner_address();
        let support_address = self.support_address().get();

        require!(
            caller == owner || caller == support_address,
            "Permission denied"
        );
    }

    #[view(getSupportAddress)]
    #[storage_mapper("supportAddress")]
    fn support_address(&self) -> SingleValueMapper<ManagedAddress>;
}
