use crate::{Alice, RuntimeCall};
use integritee_parachains_common::porteer::integritee_runtime_porteer_mint;
use parity_scale_codec::{Decode, Encode};

#[test]
fn integritee_kusama_porteer_mint_is_correct() {
	let call = integritee_runtime_porteer_mint(Alice::get(), 10, None);

	let decoded = RuntimeCall::decode(&mut call.encode().as_slice()).unwrap();

	assert_eq!(
		decoded,
		RuntimeCall::Porteer(pallet_porteer::Call::mint_ported_tokens {
			beneficiary: Alice::get(),
			amount: 10,
			forward_tokens_to_location: None,
		})
	)
}
