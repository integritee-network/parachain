// Copyright 2021 Integritee AG and Supercomputing Systems AG
// This file is part of the "Integritee parachain" and is
// based on Cumulus from Parity Technologies (UK) Ltd.

// Integritee parachain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Integritee parachain.  If not, see <http://www.gnu.org/licenses/>.

//! XCM configuration for Integritee Runtime.
//!

use super::{
	AccountId, AssetRegistry, Assets, Balance, Balances, EnsureRootOrMoreThanHalfCouncil,
	MaxInstructions, MessageQueue, ParachainInfo, ParachainSystem, PolkadotXcm, Runtime,
	RuntimeCall, RuntimeEvent, RuntimeOrigin, TreasuryAccount, XcmpQueue, TEER,
};
use crate::weights;
use core::marker::PhantomData;
use cumulus_primitives_core::{AggregateMessageOrigin, GlobalConsensus};
use cumulus_primitives_utility::XcmFeesTo32ByteAccount;
use frame_support::{
	match_types,
	pallet_prelude::{Get, PalletInfoAccess, Weight},
	parameter_types,
	traits::{Contains, ContainsPair, Everything, Nothing, TransformOrigin},
};
use frame_system::EnsureRoot;
use pallet_xcm::XcmPassthrough;
use parachains_common::{message_queue::ParaIdToSibling, AssetIdForTrustBackedAssets};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use polkadot_parachain_primitives::primitives::Sibling;
use polkadot_runtime_common::xcm_sender::NoPriceForMessageDelivery;
use scale_info::TypeInfo;
use sp_core::ConstU32;
use sp_runtime::{traits::Convert, RuntimeDebug};
use sp_std::{
	convert::{From, Into},
	prelude::*,
};
use staging_xcm::latest::prelude::*;
#[allow(deprecated)]
use staging_xcm_builder::CurrencyAdapter;
use staging_xcm_builder::{
	AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
	AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, DenyReserveTransferToRelayChain,
	DenyThenTry, DescribeAllTerminal, DescribeFamily, EnsureXcmOrigin, FixedRateOfFungible,
	FixedWeightBounds, FungiblesAdapter, HashedDescription, NativeAsset, NoChecking,
	ParentAsSuperuser, ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative,
	SiblingParachainConvertsVia, SignedAccountId32AsNative, SignedToAccountId32,
	SovereignSignedViaLocation, TakeWeightCredit, TrailingSetTopicAsId, WithComputedOrigin,
};
use staging_xcm_executor::{traits::JustTry, XcmExecutor};
use xcm_primitives::{AsAssetMultiLocation, ConvertedRegisteredAssetId};
use xcm_transactor_primitives::*;

/// Supported local Currencies. Keep this to TEER,
/// other assets will be handled through AssetRegistry pallet
#[derive(
	Encode,
	Decode,
	Eq,
	PartialEq,
	Copy,
	Clone,
	RuntimeDebug,
	PartialOrd,
	Ord,
	TypeInfo,
	MaxEncodedLen,
)]
pub enum CurrencyId {
	TEER,
}

/// Converts a Mulitloaction into a CurrencyId. Used by XCMP LocalAssetTransactor for asset filtering:
/// we only accept Assets that are convertable to a "CurrencyId".
/// other assets will be handled through AssetRegistry pallet
impl Convert<MultiLocation, Option<CurrencyId>> for CurrencyIdConvert {
	fn convert(location: MultiLocation) -> Option<CurrencyId> {
		let self_para_id: u32 = ParachainInfo::parachain_id().into();

		match location {
			// semi-absolute location spec
			MultiLocation { parents, interior } if parents == 1 => match interior {
				// that's how xTokens with Karura, Bifrost, Moonriver refers to TEER
				X2(Parachain(para_id), junction)
					if junction == TEER_GENERAL_KEY && para_id == self_para_id =>
					Some(CurrencyId::TEER),
				// that's how the Asset Hub refers to TEER
				X1(Parachain(para_id)) if para_id == self_para_id => Some(CurrencyId::TEER),
				_ => None,
			},
			// same for local location spec. we don't care if parents is 0 or 1
			MultiLocation { parents, interior } if parents == 0 => match interior {
				X1(junction) if junction == TEER_GENERAL_KEY => Some(CurrencyId::TEER),
				Here => Some(CurrencyId::TEER),
				_ => None,
			},
			_ => None,
		}
	}
}

/// Converts a MultiAsset into a CurrencyId, using the defined Mulitlocation.
impl Convert<MultiAsset, Option<CurrencyId>> for CurrencyIdConvert {
	fn convert(asset: MultiAsset) -> Option<CurrencyId> {
		if let MultiAsset { id: Concrete(location), .. } = asset {
			Self::convert(location)
		} else {
			None
		}
	}
}

parameter_types! {
	pub const RelayChainLocation: MultiLocation = MultiLocation::parent();
	pub AssetHubLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(1000)));
	pub const RelayNetwork: NetworkId = NetworkId::Kusama;
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	// The universal location within the global consensus system
	pub UniversalLocation: InteriorMultiLocation =
		X2(GlobalConsensus(RelayNetwork::get()), Parachain(ParachainInfo::parachain_id().into()));
	pub AssetsPalletLocation: MultiLocation =
		PalletInstance(<Assets as PalletInfoAccess>::index() as u8).into();
	pub CheckingAccount: AccountId = PolkadotXcm::check_account();
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
	// Foreign locations alias into accounts according to a hash of their standard description.
	HashedDescription<AccountId, DescribeFamily<DescribeAllTerminal>>,
);

/// Means for transacting TEER only.
#[allow(deprecated)]
pub type LocalNativeTransactor = CurrencyAdapter<
	// Use this currency:
	Balances,
	// Matcher: matches concrete fungible assets whose `id` could be converted into `CurrencyId`.
	IsNativeConcrete<CurrencyId, CurrencyIdConvert>,
	// Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports.
	(),
>;

/// `AssetId/Balancer` converter for `TrustBackedAssets`
pub type TrustBackedAssetsConvertedConcreteId =
	assets_common::TrustBackedAssetsConvertedConcreteId<AssetsPalletLocation, Balance>;

/// Means for transacting assets besides the native currency on this chain.
/// Even if we currently don't plan to use this for arbitrary assets on our chain,
/// there is no harm in allowing asset transactions via xcm
pub type LocalFungiblesTransactor = FungiblesAdapter<
	// Use this fungibles implementation:
	Assets,
	// Use this currency when it is a fungible asset matching the given location or name:
	TrustBackedAssetsConvertedConcreteId,
	// Convert an XCM MultiLocation into a local account id:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports of `Assets`.
	NoChecking,
	// We don't track any teleports of `Assets`, but a placeholder account is provided due to trait
	// bounds.
	CheckingAccount,
>;

/// Means for transacting reserved fungible assets.
/// AsAssetMultiLocation uses pallet_asset_registry to convert between AssetId and MultiLocation.
/// This will be used for ROC/KSM/DOT derivatives through pallet AssetRegistry
pub type ReservedFungiblesTransactor = FungiblesAdapter<
	// Use this fungibles implementation:
	Assets,
	// Use this currency when it is a registered fungible asset matching the given location or name
	// Assets not found in AssetRegistry will not be used
	ConvertedRegisteredAssetId<
		AssetIdForTrustBackedAssets,
		Balance,
		AsAssetMultiLocation<AssetIdForTrustBackedAssets, AssetRegistry>,
		JustTry,
	>,
	// Convert an XCM MultiLocation into a local account id:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports of `Assets`.
	NoChecking,
	// We don't track any teleports of `Assets`, but a placeholder account is provided due to trait
	// bounds.
	CheckingAccount,
>;

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognised.
	RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognised.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	// Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
	// transaction from the Root origin.
	ParentAsSuperuser<RuntimeOrigin>,
	// Native signed account converter; this just converts an `AccountId32` origin into a normal
	// `Origin::Signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<RuntimeOrigin>,
);

match_types! {
	pub type ParentOrParentsExecutivePlurality: impl Contains<MultiLocation> = {
		MultiLocation { parents: 1, interior: Here } |
		MultiLocation { parents: 1, interior: X1(Plurality { id: BodyId::Executive, .. }) }
	};
}
match_types! {
	pub type ParentOrSiblings: impl Contains<MultiLocation> = {
		MultiLocation { parents: 1, interior: Here } |
		MultiLocation { parents: 1, interior: X1(_) }
	};
}

// The barrier decides if we spend time executing an incoming XCM message
pub type Barrier = TrailingSetTopicAsId<
	DenyThenTry<
		DenyReserveTransferToRelayChain,
		(
			TakeWeightCredit,
			// Expected responses are OK.
			AllowKnownQueryResponses<PolkadotXcm>,
			// Allow XCMs with some computed origins to pass through.
			WithComputedOrigin<
				(
					// If the message is one that immediately attempts to pay for execution, then
					// allow it.
					AllowTopLevelPaidExecutionFrom<Everything>,
					// Parent, its pluralities (i.e. governance bodies), and the Fellows plurality
					// get free execution.
					AllowUnpaidExecutionFrom<ParentOrParentsExecutivePlurality>,
					// Subscriptions for version tracking are OK.
					AllowSubscriptionsFrom<ParentOrSiblings>,
				),
				UniversalLocation,
				ConstU32<8>,
			>,
		),
	>,
>;

pub struct ReserveAssetsFrom<T>(PhantomData<T>);
impl<T: Get<MultiLocation>> ContainsPair<MultiAsset, MultiLocation> for ReserveAssetsFrom<T> {
	fn contains(asset: &MultiAsset, origin: &MultiLocation) -> bool {
		let prefix = T::get();
		log::trace!(target: "xcm::AssetsFrom", "prefix: {:?}, origin: {:?}, asset: {:?}", prefix, origin, asset);
		&prefix == origin
	}
}
pub struct OnlyTeleportNative;
impl Contains<(MultiLocation, Vec<MultiAsset>)> for OnlyTeleportNative {
	fn contains(t: &(MultiLocation, Vec<MultiAsset>)) -> bool {
		let self_para_id: u32 = ParachainInfo::parachain_id().into();
		t.1.iter().any(|asset| {
			log::trace!(target: "xcm::OnlyTeleportNative", "Asset requested to be teleported: {:?}", asset);

			if let MultiAsset {
				id: staging_xcm::latest::AssetId::Concrete(asset_loc),
				fun: Fungible(_a),
			} = asset
			{
				match asset_loc {
					MultiLocation { parents: 0, interior: Here } => true,
					MultiLocation { parents: 1, interior: X1(Parachain(para_id)) }
						if *para_id == self_para_id =>
						true,
					_ => false,
				}
			} else {
				false
			}
		})
	}
}

pub type Traders = (
	// for TEER
	FixedRateOfFungible<
		NativePerSecond,
		XcmFeesTo32ByteAccount<LocalNativeTransactor, AccountId, TreasuryAccount>,
	>,
	// for TEER for XCM from Karura, Bifrost, Moonriver
	FixedRateOfFungible<
		NativeAliasPerSecond,
		XcmFeesTo32ByteAccount<LocalNativeTransactor, AccountId, TreasuryAccount>,
	>,
	// for KSM aka RelayNative
	FixedRateOfFungible<
		RelayNativePerSecond,
		XcmFeesTo32ByteAccount<LocalFungiblesTransactor, AccountId, TreasuryAccount>,
	>,
);

parameter_types! {
	pub const MaxAssetsIntoHolding: u32 = 64;
	pub NativePerSecond: (staging_xcm::v3::AssetId, u128,u128) = (MultiLocation::new(0,Here).into(), TEER * 70, 0u128);
	pub NativeAliasPerSecond: (staging_xcm::v3::AssetId, u128,u128) = (MultiLocation::new(0,X1(TEER_GENERAL_KEY)).into(), TEER * 70, 0u128);
	pub RelayNativePerSecond: (staging_xcm::v3::AssetId, u128,u128) = (MultiLocation::new(1,Here).into(), TEER * 70, 0u128);
	// Weight for one XCM operation.
	pub UnitWeightCost: Weight = Weight::from_parts(1_000_000u64, DEFAULT_PROOF_SIZE);
	pub const IntegriteeNative: MultiAssetFilter = Wild(AllOf { fun: WildFungible, id: Concrete(MultiLocation::here()) });
	pub AssetHubTrustedTeleporter: (MultiAssetFilter, MultiLocation) = (IntegriteeNative::get(), AssetHubLocation::get());
}

pub type TrustedTeleporters = (staging_xcm_builder::Case<AssetHubTrustedTeleporter>,);

pub type Reserves = (NativeAsset, ReserveAssetsFrom<AssetHubLocation>);

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm, ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

/// Means for transacting assets on this chain.
pub type AssetTransactors =
	(LocalNativeTransactor, ReservedFungiblesTransactor, LocalFungiblesTransactor);

pub struct SafeCallFilter;
impl frame_support::traits::Contains<RuntimeCall> for SafeCallFilter {
	fn contains(_call: &RuntimeCall) -> bool {
		// This is safe, as we prevent arbitrary xcm-transact executions.
		// For rationale, see:https://github.com/paritytech/polkadot/blob/19fdd197aff085f7f66e54942999fd536e7df475/runtime/kusama/src/xcm_config.rs#L171
		true
	}
}
pub struct XcmConfig;
impl staging_xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	// How to withdraw and deposit an asset.
	type AssetTransactor = AssetTransactors;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve = Reserves;
	type IsTeleporter = TrustedTeleporters;
	type UniversalLocation = UniversalLocation;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type Trader = Traders;
	type ResponseHandler = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type AssetTrap = PolkadotXcm;
	type AssetClaims = PolkadotXcm;
	type CallDispatcher = RuntimeCall;
	type PalletInstancesInfo = crate::AllPalletsWithSystem;
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type AssetLocker = ();
	type AssetExchanger = ();
	type FeeManager = ();
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type SafeCallFilter = SafeCallFilter;
	type Aliasers = Nothing;
}

// Converts a Signed Local Origin into a MultiLocation
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, ()>; // Prohibit sending arbitrary XCMs from users of this chain
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>; // Allow any local origin in XCM execution.
	type XcmExecuteFilter = Nothing; // Disable generic XCM execution. This does not affect Teleport or Reserve Transfer.
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = OnlyTeleportNative;
	type XcmReserveTransferFilter = Everything; // Transfer are allowed
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type UniversalLocation = UniversalLocation;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type TrustedLockers = ();
	type SovereignAccountOf = LocationToAccountId;
	type MaxLockers = ConstU32<8>;
	type WeightInfo = weights::pallet_xcm::WeightInfo<Runtime>;
	type AdminOrigin = EnsureRoot<AccountId>;
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
}

parameter_types! {
	pub const ShellRuntimeParaId: u32 = 2267u32;
	pub const IntegriteeKsmParaId: u32 = 2015u32;
}

impl pallet_xcm_transactor::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RelayCallBuilder = RelayCallBuilder<IntegriteeKsmParaId>;
	type XcmSender = XcmRouter;
	type SwapOrigin = EnsureRootOrMoreThanHalfCouncil;
	type ShellRuntimeParaId = ShellRuntimeParaId;
	type IntegriteeKsmParaId = IntegriteeKsmParaId;
	type WeightInfo = ();
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = PolkadotXcm;
	// Enqueue XCMP messages from siblings for later processing.
	type XcmpQueue = TransformOrigin<MessageQueue, AggregateMessageOrigin, ParaId, ParaIdToSibling>;
	type MaxInboundSuspended = sp_core::ConstU32<1_000>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = cumulus_pallet_xcmp_queue::weights::SubstrateWeight<Runtime>;
	type PriceForSiblingDelivery = NoPriceForMessageDelivery<ParaId>;
}

/// Copied from moonbeam: https://github.com/PureStake/moonbeam/blob/095031d171b0c163e5649ee35acbc36eef681a82/primitives/xcm/src/ethereum_xcm.rs#L34
pub const DEFAULT_PROOF_SIZE: u64 = 1024;

parameter_types! {
	pub const BaseXcmWeight: Weight= Weight::from_parts(1_000_000u64, DEFAULT_PROOF_SIZE);
	pub const MaxAssetsForTransfer: usize = 2;
}

// The min fee amount in fee asset is split into two parts:
//
// - fee asset sent to fee reserve chain = fee_amount - min_xcm_fee
// - fee asset sent to dest reserve chain = min_xcm_fee
// Check out for more information:
// https://github.com/open-web3-stack/open-runtime-module-library/tree/master/xtokens#transfer-multiple-currencies

parameter_type_with_key! {
	pub ParachainMinFee: |_location: MultiLocation| -> Option<u128> {
		None
	};
}

// What follows here are specialties only used for xToken reserve-transferring TEER to Karura, Bifrost and Moonriver

const fn teer_general_key() -> Junction {
	const TEER_KEY: [u8; 32] = *b"TEER\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
	GeneralKey { length: 4, data: TEER_KEY }
}
const TEER_GENERAL_KEY: Junction = teer_general_key();

/// Converts a CurrencyId into a Multilocation, used by xtoken for XCMP.
pub struct CurrencyIdConvert;
impl Convert<CurrencyId, Option<MultiLocation>> for CurrencyIdConvert {
	fn convert(id: CurrencyId) -> Option<MultiLocation> {
		match id {
			CurrencyId::TEER => Some(MultiLocation::new(
				1,
				X2(Parachain(ParachainInfo::parachain_id().into()), TEER_GENERAL_KEY),
			)),
		}
	}
}
