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
	AccountId, Balance, Balances, MaxInstructions, MessageQueue, ParachainInfo, ParachainSystem,
	PolkadotXcm, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin, XcmpQueue, TEER,
};
use core::marker::PhantomData;
use cumulus_primitives_core::{AggregateMessageOrigin, GlobalConsensus};
use frame_support::{
	pallet_prelude::{Get, Weight},
	parameter_types,
	traits::{ContainsPair, Everything, Nothing, TransformOrigin},
	weights::IdentityFee,
};
use frame_system::EnsureRoot;
use integritee_parachains_common::xcm_config::{
	IsNativeConcrete, MultiNativeAsset, RelativeReserveProvider, Reserve,
};
use pallet_xcm::XcmPassthrough;
use parachains_common::message_queue::ParaIdToSibling;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use polkadot_parachain_primitives::primitives::Sibling;
use polkadot_runtime_common::xcm_sender::NoPriceForMessageDelivery;
use scale_info::TypeInfo;
use sp_core::ConstU32;
use sp_runtime::{
	traits::{CheckedConversion, Convert},
	RuntimeDebug,
};
use sp_std::{
	convert::{From, Into},
	prelude::*,
};
use staging_xcm::latest::prelude::*;
#[allow(deprecated)]
use staging_xcm_builder::CurrencyAdapter;
use staging_xcm_builder::{
	AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
	AllowTopLevelPaidExecutionFrom, DenyReserveTransferToRelayChain, DenyThenTry, EnsureXcmOrigin,
	FixedWeightBounds, FrameTransactionalProcessor, ParentAsSuperuser, ParentIsPreset,
	RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
	SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit,
	UsingComponents,
};
use staging_xcm_executor::{traits::MatchesFungible, XcmExecutor};
use xcm_transactor_primitives::*;

const fn teer_general_key() -> Junction {
	const TEER_KEY: [u8; 32] = *b"TEER0000000000000000000000000000";
	GeneralKey { length: 4, data: TEER_KEY }
}

const TEER_GENERAL_KEY: Junction = teer_general_key();

parameter_types! {
	pub const RelayChainLocation: Location = Location::parent();
	pub const RelayNetwork: NetworkId = NetworkId::Kusama;
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	// The universal location within the global consensus system
	pub UniversalLocation: InteriorLocation =
		[GlobalConsensus(RelayNetwork::get()), Parachain(ParachainInfo::parachain_id().into())].into();

	pub SelfReserve: Location = Location {
		parents:0,
		interior: [TEER_GENERAL_KEY].into()
	};
}

// Supported Currencies.
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

/// Converts a CurrencyId into a Location, used by xtoken for XCMP.
pub struct CurrencyIdConvert;
impl Convert<CurrencyId, Option<Location>> for CurrencyIdConvert {
	fn convert(id: CurrencyId) -> Option<Location> {
		match id {
			CurrencyId::TEER => Some(Location {
				parents: 1,
				interior: [Parachain(ParachainInfo::parachain_id().into()), TEER_GENERAL_KEY]
					.into(),
			}),
		}
	}
}

/// Converts a Mulitloaction into a CurrencyId. Used by XCMP LocalAssetTransactor for asset filtering:
/// we only accept Assets that are convertable to a "CurrencyId".
impl Convert<Location, Option<CurrencyId>> for CurrencyIdConvert {
	fn convert(location: Location) -> Option<CurrencyId> {
		let self_para_id: u32 = ParachainInfo::parachain_id().into();

		match location.unpack() {
			(1, interior) => match interior {
				[Parachain(id), TEER_GENERAL_KEY] if *id == self_para_id => Some(CurrencyId::TEER),
				_ => None,
			},
			(0, interior) => match interior {
				[TEER_GENERAL_KEY] => Some(CurrencyId::TEER),
				_ => None,
			},
			_ => None,
		}
	}
}

/// Converts a Asset into a CurrencyId, using the defined Location.
impl Convert<Asset, Option<CurrencyId>> for CurrencyIdConvert {
	fn convert(asset: Asset) -> Option<CurrencyId> {
		if let Asset { id: AssetId(location), .. } = asset {
			Self::convert(location)
		} else {
			None
		}
	}
}

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm, ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

/// Type for specifying how a `Location` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
);

/// Means for transacting assets on this chain.
#[allow(deprecated)]
pub type LocalAssetTransactor = CurrencyAdapter<
	// Use this currency:
	Balances,
	// Matcher: matches concrete fungible assets whose `id` could be converted into `CurrencyId`.
	IsNativeConcrete<CurrencyId, CurrencyIdConvert>,
	// Do a simple punn to convert an AccountId32 Location into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports.
	(),
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

/// This struct offers uses RelativeReserveProvider to output relative views of Locations
/// However, additionally accepts a Location that aims at representing the chain part
/// (parent: 1, Parachain(paraId)) of the absolute representation of our chain.
/// If a token reserve matches against this absolute view, we return  Some(Location::here())
/// This helps users by preventing errors when they try to transfer a token through xtokens
/// to our chain (either inserting the relative or the absolute value).
pub struct AbsoluteAndRelativeReserve<AbsoluteLocation>(PhantomData<AbsoluteLocation>);
impl<AbsoluteLocation> Reserve for AbsoluteAndRelativeReserve<AbsoluteLocation>
where
	AbsoluteLocation: Get<Location>,
{
	fn reserve(asset: &Asset) -> Option<Location> {
		RelativeReserveProvider::reserve(asset).map(|relative_reserve| {
			if relative_reserve == AbsoluteLocation::get() {
				Location::here()
			} else {
				relative_reserve
			}
		})
	}
}

parameter_types! {
	// Weight for one XCM operation. Copied from moonbeam.
	pub UnitWeightCost: Weight = Weight::from_parts(200_000_000u64, DEFAULT_PROOF_SIZE);
	// One TEER buys 1 second of weight.
	pub const WeightPrice: (Location, u128) = (Location::parent(), TEER);
}

pub type Barrier = DenyThenTry<
	DenyReserveTransferToRelayChain,
	(
		TakeWeightCredit,
		AllowTopLevelPaidExecutionFrom<Everything>,
		// Expected responses are OK.
		AllowKnownQueryResponses<PolkadotXcm>,
		// Subscriptions for version tracking are OK.
		AllowSubscriptionsFrom<Everything>,
	),
>;

pub struct SafeCallFilter;
impl frame_support::traits::Contains<RuntimeCall> for SafeCallFilter {
	fn contains(_call: &RuntimeCall) -> bool {
		// This is safe, as we prevent arbitrary xcm-transact executions.
		// For rationale, see:https://github.com/paritytech/polkadot/blob/19fdd197aff085f7f66e54942999fd536e7df475/runtime/kusama/src/xcm_config.rs#L171
		true
	}
}

parameter_types! {
	pub const MaxAssetsIntoHolding: u32 = 64;
}

pub struct XcmConfig;
impl staging_xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	// How to withdraw and deposit an asset.
	type AssetTransactor = LocalAssetTransactor;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve = MultiNativeAsset<AbsoluteAndRelativeReserve<SelfLocationAbsolute>>;
	type IsTeleporter = (); // No teleport for now. Better be safe than sorry.
	type UniversalLocation = UniversalLocation;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type Trader = UsingComponents<IdentityFee<Balance>, SelfReserve, AccountId, Balances, ()>;
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
	type TransactionalProcessor = FrameTransactionalProcessor;
}

// Converts a Signed Local Origin into a Location
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

// FIXME: We should probably update the configuration here.
// See acala and moonbeam example : https://github.com/integritee-network/parachain/issues/103
impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, ()>; // Prohibit sending arbitrary XCMs from users of this chain
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>; // Allow any local origin in XCM execution.
	type XcmExecuteFilter = Nothing; // Disable generic XCM execution. This does not affect Teleport or Reserve Transfer.
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Nothing; // Do not allow teleports
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
	// TODO pallet-xcm weights
	type WeightInfo = pallet_xcm::TestWeightInfo;
	type AdminOrigin = EnsureRoot<AccountId>;
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
}

parameter_types! {
	pub const IntegriteeKsmParaId: u32 = 2015;
	pub const ShellRuntimeParaId: u32 = 2267;
}

impl pallet_xcm_transactor::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RelayCallBuilder = RelayCallBuilder<ShellRuntimeParaId>;
	type XcmSender = XcmRouter;
	type SwapOrigin = EnsureRoot<AccountId>;
	type ShellRuntimeParaId = ShellRuntimeParaId;
	type IntegriteeKsmParaId = IntegriteeKsmParaId;
	type WeightInfo = ();
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

// FIXME: Update to PolkadotXcm.
impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = ();
	// Enqueue XCMP messages from siblings for later processing.
	type XcmpQueue = TransformOrigin<MessageQueue, AggregateMessageOrigin, ParaId, ParaIdToSibling>;
	type MaxInboundSuspended = sp_core::ConstU32<1_000>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = cumulus_pallet_xcmp_queue::weights::SubstrateWeight<Runtime>;
	type PriceForSiblingDelivery = NoPriceForMessageDelivery<ParaId>;
}

parameter_types! {
	// This is how we are going to detect whether the asset is a Reserve asset
	pub SelfLocation: Location = Location::here();
	// We need this to be able to catch when someone is trying to execute a non-
	// cross-chain transfer in xtokens through the absolute path way
	pub SelfLocationAbsolute: Location = Location {
		parents:1,
		interior: [Parachain(ParachainInfo::parachain_id().into())].into()
	};
}

/// Copied from moonbeam: https://github.com/PureStake/moonbeam/blob/095031d171b0c163e5649ee35acbc36eef681a82/primitives/xcm/src/ethereum_xcm.rs#L34
pub const DEFAULT_PROOF_SIZE: u64 = 128 * 1024;
