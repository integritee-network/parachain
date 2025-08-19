use crate::{IntegriteePolkadotLocation, IntegriteePolkadotSovereignAccount, RuntimeCall};
use integritee_parachains_common::porteer::integritee_runtime_porteer_mint;
use parity_scale_codec::{Decode, Encode};
use xcm_executor::traits::ConvertLocation;

#[test]
fn ik_porteer_sovereign_account_matches() {
	sp_io::TestExternalities::default().execute_with(|| {
		let account = crate::xcm_config::LocationToAccountId::convert_location(
			&IntegriteePolkadotLocation::get(),
		)
		.unwrap();

		assert_eq!(account, IntegriteePolkadotSovereignAccount::get());
	});
}

#[test]
fn integritee_kusama_porteer_mint_is_correct() {
	let beneficiary = IntegriteePolkadotSovereignAccount::get();
	let amount = 10;

	let call = integritee_runtime_porteer_mint(beneficiary.clone(), amount, None);

	let decoded = RuntimeCall::decode(&mut call.encode().as_slice()).unwrap();

	assert_eq!(
		decoded,
		RuntimeCall::Porteer(pallet_porteer::Call::mint_ported_tokens {
			beneficiary,
			amount,
			forward_tokens_to_location: None,
		})
	)
}
