use staging_xcm::{
	latest::{Location, NetworkId},
	prelude::{GlobalConsensus, PalletInstance, Parachain},
};
use staging_xcm_executor::traits::ConvertLocation;

#[test]
fn ik_porteer_sovereign_account_matches() {
	sp_io::TestExternalities::default().execute_with(|| {
		let location = Location {
			parents: 2,
			// Todo: maybe we can use the sovereign account of the parachain, without the pallet ?
			interior: (GlobalConsensus(NetworkId::Kusama), Parachain(2015), PalletInstance(56)).into(),
		};

		let account = crate::xcm_config::LocationToAccountId::convert_location(&location).unwrap();

		assert_eq!(
			account,
			crate::PorteerOnIntegriteeKusama::get()
		);
	});
}
