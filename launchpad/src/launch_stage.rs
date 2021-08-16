use elrond_wasm::api::EndpointFinishApi;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TypeAbi, PartialEq)]
pub enum LaunchStage {
    None,
    AddTickets,
    SelectWinners,
    ConfirmTickets,
    SelectNewWinners,
    WaitBeforeClaim,
    Claim,
}

impl LaunchStage {
    fn output_bytes(&self) -> &'static [u8] {
        match self {
            LaunchStage::None => b"None",
            LaunchStage::AddTickets => b"Add Tickets",
            LaunchStage::SelectWinners => b"Select Winners",
            LaunchStage::ConfirmTickets => b"Confirm Tickets",
            LaunchStage::SelectNewWinners => b"Select New Winners",
            LaunchStage::WaitBeforeClaim => b"Wait Period Before Claim",
            LaunchStage::Claim => b"Claim",
        }
    }
}

impl EndpointResult for LaunchStage {
    type DecodeAs = BoxedBytes;

    #[inline]
    fn finish<FA>(&self, api: FA)
    where
        FA: EndpointFinishApi + Clone + 'static,
    {
        self.output_bytes().finish(api);
    }
}
