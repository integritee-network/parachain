use core::marker::PhantomData;
use frame_support::traits::ContainsPair;
use sp_runtime::traits::{CheckedConversion, Convert};
use xcm::{
	latest::{Asset, AssetId, Junction, Location},
	prelude::{Fungible, Parachain},
};
use xcm_executor::traits::MatchesFungible;

/// Type alias to conveniently refer to `frame_system`'s `Config::AccountId`.
pub type AccountIdOf<R> = <R as frame_system::Config>::AccountId;

// ORML remainders
pub trait Reserve {
	/// Returns assets reserve location.
	fn reserve(asset: &Asset) -> Option<Location>;
}

// Provide reserve in relative path view
// Self tokens are represeneted as Here
pub struct RelativeReserveProvider;

impl Reserve for RelativeReserveProvider {
	fn reserve(asset: &Asset) -> Option<Location> {
		let AssetId(location) = &asset.id;
		if location.parents == 0 && !is_chain_junction(location.first_interior()) {
			Some(Location::here())
		} else {
			chain_part(location)
		}
	}
}
/// A `ContainsPair` implementation. Filters multi native assets whose
/// reserve is same with `origin`.
pub struct MultiNativeAsset<ReserveProvider>(PhantomData<ReserveProvider>);
impl<ReserveProvider> ContainsPair<Asset, Location> for MultiNativeAsset<ReserveProvider>
where
	ReserveProvider: Reserve,
{
	fn contains(asset: &Asset, origin: &Location) -> bool {
		if let Some(ref reserve) = ReserveProvider::reserve(asset) {
			if reserve == origin {
				return true;
			}
		}
		false
	}
}

/// A `MatchesFungible` implementation. It matches concrete fungible assets
/// whose `id` could be converted into `CurrencyId`.
pub struct IsNativeConcrete<CurrencyId, CurrencyIdConvert>(
	PhantomData<(CurrencyId, CurrencyIdConvert)>,
);
impl<CurrencyId, CurrencyIdConvert, Amount> MatchesFungible<Amount>
	for IsNativeConcrete<CurrencyId, CurrencyIdConvert>
where
	CurrencyIdConvert: Convert<Location, Option<CurrencyId>>,
	Amount: TryFrom<u128>,
{
	fn matches_fungible(a: &Asset) -> Option<Amount> {
		if let (Fungible(ref amount), AssetId(location)) = (&a.fun, &a.id) {
			if CurrencyIdConvert::convert(location.clone()).is_some() {
				return CheckedConversion::checked_from(*amount);
			}
		}
		None
	}
}

fn is_chain_junction(junction: Option<&Junction>) -> bool {
	matches!(junction, Some(Parachain(_)))
}

fn chain_part(loc: &Location) -> Option<Location> {
	match (loc.parents, loc.first_interior()) {
		// sibling parachain
		(1, Some(Parachain(id))) => Some(Location::new(1, [Parachain(*id)])),
		// parent
		(1, _) => Some(Location::parent()),
		// children parachain
		(0, Some(Parachain(id))) => Some(Location::new(0, [Parachain(*id)])),
		_ => None,
	}
}
