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

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

extern crate alloc;
use alloc::borrow::Cow;
use cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
use cumulus_primitives_core::AggregateMessageOrigin;
pub use frame_support::{
	construct_runtime,
	dispatch::DispatchClass,
	genesis_builder_helper::{build_state, get_preset},
	pallet_prelude::Get,
	parameter_types,
	traits::{
		EitherOfDiverse, Everything, IsInVec, Nothing, PalletInfoAccess, Randomness,
		WithdrawReasons,
	},
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight},
		IdentityFee, Weight, WeightToFee as _,
	},
	PalletId, StorageValue,
};
use frame_support::{
	derive_impl, ord_parameter_types,
	traits::{
		fungible::{Credit, HoldConsideration, NativeFromLeft, NativeOrWithId, UnionOf},
		tokens::{
			imbalance::{ResolveAssetTo, ResolveTo},
			ConversionFromAssetBalance, PayFromAccount,
		},
		AsEnsureOriginWithArg, ConstBool, EnsureOriginWithArg, EqualPrivilegeOnly, Imbalance,
		InstanceFilter, LinearStoragePrice, OnUnbalanced,
	},
	weights::ConstantMultiplier,
};
use frame_system::{
	limits::{BlockLength, BlockWeights},
	EnsureRoot, EnsureSignedBy, EnsureWithSuccess,
};
pub use integritee_parachains_common::{
	self as integritee_common, AccountId, Address, Balance, BlockNumber, Hash, Header, Nonce,
	Signature, MILLISECS_PER_BLOCK,
};
use integritee_parachains_common::{
	fee::{SlowAdjustingFeeUpdate, WeightToFee},
	AuraId, AVERAGE_ON_INITIALIZE_RATIO, BLOCK_PROCESSING_VELOCITY, DAYS, HOURS,
	MAXIMUM_BLOCK_WEIGHT, MINUTES, MS_PER_DAY, NORMAL_DISPATCH_RATIO,
	RELAY_CHAIN_SLOT_DURATION_MILLIS, SLOT_DURATION, UNINCLUDED_SEGMENT_CAPACITY,
};
use pallet_asset_conversion::{Ascending, Chain, WithFirstAsset};
pub use pallet_balances::Call as BalancesCall;
use pallet_balances::WeightInfo;
pub use pallet_claims;
pub use pallet_collective;
pub use pallet_enclave_bridge;
use pallet_porteer::{ForwardPortedTokens, PortTokens};
pub use pallet_sidechain;
pub use pallet_teeracle;
pub use pallet_teerex::Call as TeerexCall;
pub use pallet_timestamp::Call as TimestampCall;
use pallet_treasury::TreasuryAccountId;
use parachains_common::{message_queue::NarrowOriginToSibling, AssetIdForTrustBackedAssets};
use parity_scale_codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_api::impl_runtime_apis;
use sp_core::{
	crypto::{AccountId32, KeyTypeId},
	ConstU128, ConstU32, OpaqueMetadata,
};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
use sp_runtime::{
	generic, impl_opaque_keys,
	traits::{AccountIdConversion, BlakeTwo256, Block as BlockT, ConvertInto, IdentityLookup},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, DispatchError, RuntimeDebug,
};
pub use sp_runtime::{Perbill, Permill};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use xcm::{
	latest::prelude::AssetId, Version as XcmVersion, VersionedAssetId, VersionedAssets,
	VersionedLocation, VersionedXcm,
};
use xcm_runtime_apis::{
	dry_run::{CallDryRunEffects, Error as XcmDryRunApiError, XcmDryRunEffects},
	fees::Error as XcmPaymentApiError,
};

mod helpers;
mod weights;

mod migrations;
pub mod xcm_config;

#[cfg(test)]
mod tests;

pub type SessionHandlers = ();

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	pub use integritee_parachains_common::opaque::*;
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: Cow::Borrowed("integritee-polkadot"),
	impl_name: Cow::Borrowed("integritee-full"),
	authoring_version: 2,
	spec_version: 560,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 8,
	system_version: 0,
};

pub const TEER: Balance = 1_000_000_000_000;
pub const MILLITEER: Balance = 1_000_000_000;
pub const MICROTEER: Balance = 1_000_000;

// Logic from Polkadot/Kusama.
pub const fn deposit(items: u32, bytes: u32) -> Balance {
	items as Balance * 20 * TEER + (bytes as Balance) * 1_000 * MICROTEER
}

/// A timestamp: milliseconds since the unix epoch.
pub type Moment = u64;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

pub struct DealWithFees<R>(sp_std::marker::PhantomData<R>);

impl<R> OnUnbalanced<Credit<R::AccountId, pallet_balances::Pallet<R>>> for DealWithFees<R>
where
	R: pallet_balances::Config + pallet_authorship::Config + pallet_treasury::Config,
	<R as frame_system::Config>::AccountId: From<AccountId>,
	<R as frame_system::Config>::AccountId: Into<AccountId>,
{
	fn on_unbalanceds(
		mut fees_then_tips: impl Iterator<Item = Credit<R::AccountId, pallet_balances::Pallet<R>>>,
	) {
		if let Some(fees) = fees_then_tips.next() {
			// for fees, 1% to treasury, 99% burned
			// TODO: apply burning function based on cumulative number of extrinsics (#32)
			let mut split = fees.ration(1, 99);

			// tips (voluntary extra fees) go to the treasury entirely. no burning
			if let Some(tips) = fees_then_tips.next() {
				tips.merge_into(&mut split.0);
			}
			ResolveTo::<TreasuryAccountId<R>, pallet_balances::Pallet<R>>::on_unbalanced(split.0);
			// burn remainder by not assigning imbalance to someone
		}
	}
}

parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
	pub const Version: RuntimeVersion = VERSION;
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u8 = 13;
}

/// The default types are being injected by [`derive_impl`](`frame_support::derive_impl`) from
/// [`ParaChainDefaultConfig`](`struct@frame_system::config_preludes::ParaChainDefaultConfig`),
/// but overridden as needed.
#[derive_impl(frame_system::config_preludes::ParaChainDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	type BaseCallFilter = Everything;
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = RuntimeBlockLength;
	type Block = Block;
	type AccountId = AccountId;
	type Nonce = Nonce;
	type Hash = Hash;
	type BlockHashCount = BlockHashCount;
	type DbWeight = RocksDbWeight;
	type Version = Version;
	type AccountData = pallet_balances::AccountData<Balance>;
	type SystemWeightInfo = weights::frame_system::WeightInfo<Runtime>;
	type SS58Prefix = SS58Prefix;
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = Moment;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = weights::pallet_timestamp::WeightInfo<Runtime>;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = MILLITEER;
	pub const TransferFee: u128 = MILLITEER;
	pub const CreationFee: u128 = MILLITEER;
	pub const TransactionByteFee: u128 = MICROTEER;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
	pub const OperationalFeeMultiplier: u8 = 5;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = weights::pallet_balances::WeightInfo<Runtime>;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type DoneSlashHandler = ();
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction =
		pallet_transaction_payment::FungibleAdapter<Balances, DealWithFees<Runtime>>;
	type WeightToFee = WeightToFee;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
	type WeightInfo = ();
}

parameter_types! {
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub const DepositBase: Balance = deposit(1, 88);
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = deposit(0, 32);
	pub const MaxSignatories: u16 = 10; //100
}

impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = weights::pallet_multisig::WeightInfo<Runtime>;
	type BlockNumberProvider = System;
}

parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub const ProxyDepositBase: Balance = deposit(1, 8);
	// Additional storage item size of 33 bytes.
	pub const ProxyDepositFactor: Balance = deposit(0, 33);
	pub const MaxProxies: u16 = 32;
	pub const AnnouncementDepositBase: Balance = deposit(1, 8);
	pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
	pub const MaxPending: u16 = 32;
}

/// The type used to represent the kinds of proxying allowed.
#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	DecodeWithMemTracking,
	RuntimeDebug,
	MaxEncodedLen,
	TypeInfo,
)]
pub enum ProxyType {
	Any,         // Any transactions.
	NonTransfer, // Any type of transaction except balance transfers (including vested transfers)
	Governance,
	// Staking = 3,
	// IdentityJudgement = 4,
	CancelProxy,
	// Auction,
}

impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}

impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => matches!(
				c,
				RuntimeCall::System {..} |
				RuntimeCall::Timestamp {..} |
				// Specifically omitting Indices `transfer`, `force_transfer`
				// Specifically omitting the entire Balances pallet
				RuntimeCall::Treasury {..} |
				RuntimeCall::Bounties(..) |
				RuntimeCall::ChildBounties(..) |
				// EnclaveBridge contains shielding and unshielding, which qualifies as transfer
				RuntimeCall::Teerex(_) |
				RuntimeCall::Teeracle(_) |
				RuntimeCall::Sidechain(_) |
//				RuntimeCall::Vesting(pallet_vesting::Call::vest {..}) |
//				Call::::Vesting(pallet_vesting::Call::vest_other {..}) |
				// Specifically omitting Vesting `vested_transfer`, and `force_vested_transfer`
				RuntimeCall::Proxy {..} |
				RuntimeCall::Multisig {..}
			),
			ProxyType::Governance => {
				matches!(
					c,
					RuntimeCall::Treasury { .. } |
						RuntimeCall::Bounties(..) |
						RuntimeCall::ChildBounties(..)
				)
			},
			ProxyType::CancelProxy => {
				matches!(c, RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. }))
			},
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			(ProxyType::NonTransfer, _) => true,
			_ => false,
		}
	}
}

impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = MaxProxies;
	type WeightInfo = weights::pallet_proxy::WeightInfo<Runtime>;
	type MaxPending = MaxPending;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
	type BlockNumberProvider = System;
}

parameter_types! {
	pub const MinVestedTransfer: Balance = TEER;
		pub UnvestedFundsAllowedWithdrawReasons: WithdrawReasons =
			WithdrawReasons::except(WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE);
}

impl pallet_vesting::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BlockNumberToBalance = ConvertInto;
	type BlockNumberProvider = System;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = weights::pallet_vesting::WeightInfo<Runtime>;
	type UnvestedFundsAllowedWithdrawReasons = UnvestedFundsAllowedWithdrawReasons;
	const MAX_VESTING_SCHEDULES: u32 = 28;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
		RuntimeBlockWeights::get().max_block;
	pub const MaxScheduledPerBlock: u32 = 50;
}

impl pallet_scheduler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	// one schedule is the founder allocation. We only allow RootOrigin here such that it takes a democracy proposal to change this schedule
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = weights::pallet_scheduler::WeightInfo<Runtime>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type Preimages = Preimage;
	type BlockNumberProvider = System;
}

impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = weights::pallet_utility::WeightInfo<Runtime>;
}

parameter_types! {
	pub const PreimageBaseDeposit: Balance = TEER;
	pub const PreimageByteDeposit: Balance = deposit(0, 1);
	pub const PreimageHoldReason: RuntimeHoldReason =
		RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);
}

impl pallet_preimage::Config for Runtime {
	type WeightInfo = weights::pallet_preimage::WeightInfo<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type Consideration = HoldConsideration<
		AccountId,
		Balances,
		PreimageHoldReason,
		LinearStoragePrice<PreimageBaseDeposit, PreimageByteDeposit, Balance>,
	>;
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const RelayOrigin: AggregateMessageOrigin = AggregateMessageOrigin::Parent;
}

type ConsensusHook = cumulus_pallet_aura_ext::FixedVelocityConsensusHook<
	Runtime,
	RELAY_CHAIN_SLOT_DURATION_MILLIS,
	BLOCK_PROCESSING_VELOCITY,
	UNINCLUDED_SEGMENT_CAPACITY,
>;

impl cumulus_pallet_parachain_system::Config for Runtime {
	type WeightInfo = weights::cumulus_pallet_parachain_system::WeightInfo<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId = staging_parachain_info::Pallet<Runtime>;
	type DmpQueue = frame_support::traits::EnqueueWithOrigin<MessageQueue, RelayOrigin>;
	type ReservedDmpWeight = ReservedDmpWeight;
	type OutboundXcmpMessageSource = XcmpQueue;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
	type ConsensusHook = ConsensusHook;
	type SelectCore = cumulus_pallet_parachain_system::DefaultCoreSelector<Runtime>;
}

impl staging_parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

parameter_types! {
	pub MessageQueueServiceWeight: Weight = Perbill::from_percent(35) * RuntimeBlockWeights::get().max_block;
}

impl pallet_message_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_message_queue::WeightInfo<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type MessageProcessor = pallet_message_queue::mock_helpers::NoopMessageProcessor<
		cumulus_primitives_core::AggregateMessageOrigin,
	>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type MessageProcessor = xcm_builder::ProcessXcmMessage<
		AggregateMessageOrigin,
		xcm_executor::XcmExecutor<xcm_config::XcmConfig>,
		RuntimeCall,
	>;
	type Size = u32;
	// The XCMP queue pallet is only ever able to handle the `Sibling(ParaId)` origin:
	type QueueChangeHandler = NarrowOriginToSibling<XcmpQueue>;
	type QueuePausedQuery = NarrowOriginToSibling<XcmpQueue>;
	type HeapSize = sp_core::ConstU32<{ 64 * 1024 }>;
	type MaxStale = sp_core::ConstU32<8>;
	type ServiceWeight = MessageQueueServiceWeight;
	type IdleMaxServiceWeight = ();
}

parameter_types! {
	pub const AssetDeposit: Balance = TEER;
	pub const ApprovalDeposit: Balance = 100 * MILLITEER;
	pub const StringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = TEER;
	pub const MetadataDepositPerByte: Balance = 10 * MILLITEER;
	pub const MaxInstructions: u32 = 100;
	pub const MaxAuthorities: u32 = 100_000;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = MaxAuthorities;
	type AllowMultipleBlocksPerSlot = ConstBool<false>;
	type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Self>;
}

// Integritee pallet
parameter_types! {
	pub const MomentsPerDay: Moment = 86_400_000; // [ms/d]
	pub const MaxAttestationRenewalPeriod: Moment =172_800_000; // 48h
}

impl pallet_teerex::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MomentsPerDay = MomentsPerDay;
	type MaxAttestationRenewalPeriod = MaxAttestationRenewalPeriod;
	type WeightInfo = weights::pallet_teerex::WeightInfo<Runtime>;
}

impl pallet_enclave_bridge::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = pallet_balances::Pallet<Runtime>;
	type WeightInfo = weights::pallet_enclave_bridge::WeightInfo<Runtime>;
}

// Integritee pallet
parameter_types! {
	pub const EarlyBlockProposalLenience: u64 = 100;
}

impl pallet_sidechain::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_sidechain::WeightInfo<Runtime>;
}
// added by Integritee
parameter_types! {
	pub const UnlockPeriod: Moment = 7 * MS_PER_DAY;
}
impl pallet_teerdays::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_teerdays::WeightInfo<Runtime>;
	type Currency = Balances;
	type CurrencyBalance = Balance;
	type UnlockPeriod = UnlockPeriod;
}

parameter_types! {
	pub Prefix: &'static [u8] = b"Pay TEERs to the integriTEE account:";
}

// Integritee pallet
impl pallet_claims::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type VestingSchedule = Vesting;
	type Prefix = Prefix;
	type MoveClaimOrigin = EnsureRoot<AccountId>;
	type WeightInfo = weights::pallet_claims::WeightInfo<Runtime>;
}
parameter_types! {
	pub const MaxWhitelistedReleases: u32 = 10;
	pub const MaxOracleBlobLen: u32 = 4096;
}

// Integritee pallet
impl pallet_teeracle::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_teeracle::WeightInfo<Runtime>;
	type MaxWhitelistedReleases = MaxWhitelistedReleases;
	type MaxOracleBlobLen = MaxOracleBlobLen;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 100 * MILLITEER;
	pub const ProposalBondMaximum: Balance = 500 * TEER;
	pub const SpendPeriod: BlockNumber = prod_or_fast!(6 * DAYS, 6 * MINUTES);
	pub const PayoutSpendPeriod: BlockNumber = prod_or_fast!(6 * DAYS, 6 * MINUTES);
	pub const Burn: Permill = Permill::from_percent(1);
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const DataDepositPerByte: Balance = 100 * MILLITEER;
	pub const MaxApprovals: u32 = 100;
	pub const MaxBalance: Balance = Balance::MAX;
	pub TreasuryAccount: AccountId = Treasury::account_id();
}

pub struct NoConversion;

impl ConversionFromAssetBalance<u128, (), u128> for NoConversion {
	type Error = ();
	fn from_asset_balance(balance: Balance, _asset_id: ()) -> Result<Balance, Self::Error> {
		Ok(balance)
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn ensure_successful(_: ()) {}
}

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryPalletId;
	type Currency = pallet_balances::Pallet<Runtime>;
	type RejectOrigin = EnsureRootOrMoreThanHalfCouncil;
	type RuntimeEvent = RuntimeEvent;
	type SpendPeriod = SpendPeriod; //Cannot be 0: Error: Thread 'tokio-runtime-worker' panicked at 'attempt to calculate the remainder with a divisor of zero
	type Burn = (); //No burn
	type BurnDestination = (); //No burn
	type SpendFunds = Bounties;
	type SpendOrigin = EnsureWithSuccess<EnsureRoot<AccountId>, AccountId, MaxBalance>;
	type MaxApprovals = MaxApprovals; //0:cannot approve any proposal
	type WeightInfo = weights::pallet_treasury::WeightInfo<Runtime>;
	type AssetKind = ();
	type Beneficiary = AccountId;
	type BeneficiaryLookup = IdentityLookup<Self::Beneficiary>;
	type Paymaster = PayFromAccount<Balances, TreasuryAccount>;
	type BalanceConverter = NoConversion;
	type PayoutPeriod = PayoutSpendPeriod;
	type BlockNumberProvider = System;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

parameter_types! {
	pub const BountyDepositBase: Balance = TEER;
	pub const BountyDepositPayoutDelay: BlockNumber = prod_or_fast!(4 * DAYS, 4 * MINUTES);
	pub const BountyUpdatePeriod: BlockNumber = prod_or_fast!(90 * DAYS, 15 * MINUTES);
	pub const MaximumReasonLength: u32 = 16384;
	pub const CuratorDepositMultiplier: Permill = Permill::from_percent(50);
	pub const CuratorDepositMin: Balance = TEER;
	pub const CuratorDepositMax: Balance = 100 * TEER;
	pub const BountyValueMinimum: Balance = 100 * TEER;
}

impl pallet_bounties::Config for Runtime {
	type BountyDepositBase = BountyDepositBase;
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	type BountyUpdatePeriod = BountyUpdatePeriod;
	type CuratorDepositMultiplier = CuratorDepositMultiplier;
	type CuratorDepositMin = CuratorDepositMin;
	type CuratorDepositMax = CuratorDepositMax;
	type BountyValueMinimum = BountyValueMinimum;
	type ChildBountyManager = ChildBounties;
	type DataDepositPerByte = DataDepositPerByte;
	type RuntimeEvent = RuntimeEvent;
	type MaximumReasonLength = MaximumReasonLength;
	type WeightInfo = weights::pallet_bounties::WeightInfo<Runtime>;
	type OnSlash = (); // Don't slash rejected bounties
}

parameter_types! {
	pub const MaxActiveChildBountyCount: u32 = 100;
	pub const ChildBountyValueMinimum: Balance = BountyValueMinimum::get() / 10;
}

impl pallet_child_bounties::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaxActiveChildBountyCount = MaxActiveChildBountyCount;
	type ChildBountyValueMinimum = ChildBountyValueMinimum;
	type WeightInfo = weights::pallet_child_bounties::WeightInfo<Runtime>;
}

/// Council collective instance declaration.
/// The council primarly serves to optimize and balance the inclusive referendum system,
/// by being allowed to porpose external democracy proposals and controls the treasury.
pub type CouncilInstance = pallet_collective::Instance1;

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = prod_or_fast!(3 * DAYS, 2 * MINUTES);
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
	pub MaxProposalWeight: Weight = Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;
}

impl pallet_collective::Config<CouncilInstance> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = weights::pallet_collective::WeightInfo<Runtime>;
	type SetMembersOrigin = EnsureRootOrMoreThanHalfCouncil;
	type MaxProposalWeight = MaxProposalWeight;
	type DisapproveOrigin = EnsureRoot<AccountId>;
	type KillOrigin = EnsureRoot<AccountId>;
	type Consideration = ();
}

pub type EnsureRootOrMoreThanHalfCouncil = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<AccountId, CouncilInstance, 1, 2>,
>;

/// Technical committee collective instance declaration.
/// The technical committee primarly serves to safeguard against malicious referenda, implement bug
/// fixes, reverse faulty runtime updates, or add new but battle-tested features.
pub type TechnicalCommitteeInstance = pallet_collective::Instance2;

parameter_types! {
	pub const TechnicalMotionDuration: BlockNumber = prod_or_fast!(3 * DAYS, 2 * MINUTES);
	pub const TechnicalMaxProposals: u32 = 100;
	pub const TechnicalMaxMembers: u32 = 100;
}

impl pallet_collective::Config<TechnicalCommitteeInstance> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeEvent = RuntimeEvent;
	type Proposal = RuntimeCall;
	/// The maximum amount of time (in blocks) for technical committee members to vote on motions.
	/// Motions may end in fewer blocks if enough votes are cast to determine the result.
	type MotionDuration = TechnicalMotionDuration;
	/// The maximum number of Proposlas that can be open in the technical committee at once.
	type MaxProposals = TechnicalMaxProposals;
	/// The maximum number of technical committee members.
	type MaxMembers = TechnicalMaxMembers;
	type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
	type WeightInfo = weights::pallet_collective::WeightInfo<Runtime>;
	type SetMembersOrigin = EnsureRootOrMoreThanHalfCouncil;
	type MaxProposalWeight = MaxProposalWeight;
	type DisapproveOrigin = EnsureRoot<AccountId>;
	type KillOrigin = EnsureRoot<AccountId>;
	type Consideration = ();
}

pub type EnsureRootOrMoreThanHalfTechnicalCommittee = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCommitteeInstance, 1, 2>,
>;

pub type EnsureRootOrAllTechnicalCommittee = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCommitteeInstance, 1, 1>,
>;

use integritee_parachains_common::porteer::forward_teer;
use sp_core::hex2array;
use xcm::{
	latest::{Location, NetworkId},
	prelude::{GlobalConsensus, Parachain},
};
use xcm_builder::AliasesIntoAccountId32;

ord_parameter_types! {
	pub const Alice: AccountId = AccountId::new(hex2array!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"));

	pub const IntegriteeKusamaLocation: Location = Location {
			parents: 2,
			interior: (GlobalConsensus(NetworkId::Kusama), Parachain(2015)).into(),
		};
	pub const IntegriteeKusamaSovereignAccount: AccountId = AccountId::new(hex2array!("8e420f365988e859988375baa048faf25a88de7ddd979425ccaad93fea6b244d"));
}

pub struct NeverPortTokens;

impl PortTokens for NeverPortTokens {
	type AccountId = AccountId;
	type Balance = Balance;
	type Location = Location;
	type Error = DispatchError;

	fn port_tokens(
		_who: &Self::AccountId,
		_amount: Self::Balance,
		_location: Option<Self::Location>,
	) -> Result<(), Self::Error> {
		Err(DispatchError::Other("porteer: Porting Tokens disabled"))
	}
}

parameter_types! {
	pub const AnyNetwork: Option<NetworkId> = None;
}

impl ForwardPortedTokens for NeverPortTokens {
	type AccountId = AccountId;
	type Balance = Balance;
	type Location = Location;
	type Error = DispatchError;

	fn forward_ported_tokens(
		who: &Self::AccountId,
		amount: Self::Balance,
		location: Self::Location,
	) -> Result<(), Self::Error> {
		forward_teer::<Runtime, AliasesIntoAccountId32<AnyNetwork, AccountId>>(
			who.clone(),
			location,
			amount,
		)
	}
}

parameter_types! {
	pub const HeartBeatTimeout: BlockNumber = 10;
}

impl pallet_porteer::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type PorteerAdmin =
		EitherOfDiverse<EnsureSignedBy<Alice, AccountId32>, EnsureRoot<AccountId32>>;
	type HeartBeatTimeout = HeartBeatTimeout;
	type TokenSenderLocationOrigin = EitherOfDiverse<
		EnsureSignedBy<IntegriteeKusamaSovereignAccount, AccountId32>,
		EnsureRoot<AccountId32>,
	>;
	type PortTokensToDestination = NeverPortTokens;
	type ForwardPortedTokensToDestinations = NeverPortTokens;
	type Location = Location;
	type Fungible = Balances;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = PorteerBenchmarkHelper;
}

pub struct PorteerBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl pallet_porteer::BenchmarkHelper<Location> for PorteerBenchmarkHelper {
	fn get_whitelisted_location() -> Location {
		use xcm::prelude::Junctions;
		Location::new(1, Junctions::X1([Parachain(1000u32.into())].into()))
	}
}

parameter_types! {
	pub const LaunchPeriod: BlockNumber = prod_or_fast!(5 * DAYS, 5 * MINUTES);
	pub const VotingPeriod: BlockNumber = prod_or_fast!(5 * DAYS, 5 * MINUTES);
	pub FastTrackVotingPeriod: BlockNumber = prod_or_fast!(3 * HOURS, 2 * MINUTES);
	pub const MinimumDeposit: Balance = 100 * TEER;
	pub EnactmentPeriod: BlockNumber = prod_or_fast!(2 * DAYS, 1);
	pub const CooloffPeriod: BlockNumber = prod_or_fast!(7 * DAYS, MINUTES);
	pub const InstantAllowed: bool = true;
	pub const MaxVotes: u32 = 100;
	pub const MaxProposals: u32 = 100;
}

impl pallet_democracy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type VoteLockingPeriod = EnactmentPeriod;
	type MinimumDeposit = MinimumDeposit;
	//// Origin allowed to schedule a SuperMajorityApprove (default decline)
	/// referendum once it is is legal for an externally proposed referendum
	type ExternalOrigin = EnsureRootOrMoreThanHalfCouncil;
	/// Origin allowed to schedule a majority-carries (Simple Majority)
	/// referendum once it is legal for an externally proposed referendum
	type ExternalMajorityOrigin = EnsureRootOrMoreThanHalfCouncil;
	/// Origin allowed to schedule a SuperMajorityAgainst (default accept)
	/// referendum once it is legal for an externally proposed referendum
	type ExternalDefaultOrigin = EnsureRootOrMoreThanHalfCouncil;
	/// Majority of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin = EnsureRootOrMoreThanHalfTechnicalCommittee;
	type InstantOrigin = EnsureRootOrMoreThanHalfTechnicalCommittee;
	type InstantAllowed = InstantAllowed;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	// To cancel a proposal which has been passed.
	type CancellationOrigin = EnsureRoot<AccountId>;
	type BlacklistOrigin = EnsureRootOrMoreThanHalfCouncil;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = EnsureRootOrAllTechnicalCommittee;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cooloff period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCommitteeInstance>;
	type CooloffPeriod = CooloffPeriod;
	type Slash = Treasury;
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = MaxVotes;
	type WeightInfo = weights::pallet_democracy::WeightInfo<Runtime>;
	type MaxProposals = MaxProposals;
	type Preimages = Preimage;
	type MaxDeposits = ConstU32<100>;
	type MaxBlacklisted = ConstU32<100>;
	type SubmitOrigin = frame_system::EnsureSigned<AccountId>;
}

impl orml_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SovereignOrigin = EnsureRoot<AccountId>;
}

pub type AssetBalance = Balance;

/// always denies creation of assets
pub struct NoAssetCreators;

impl EnsureOriginWithArg<RuntimeOrigin, AssetIdForTrustBackedAssets> for NoAssetCreators {
	type Success = AccountId;

	fn try_origin(
		o: RuntimeOrigin,
		_a: &AssetIdForTrustBackedAssets,
	) -> sp_std::result::Result<Self::Success, RuntimeOrigin> {
		Err(o)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin(_a: &AssetIdForTrustBackedAssets) -> Result<RuntimeOrigin, ()> {
		Err(())
	}
}

pub type MainAssetsInstance = pallet_assets::Instance1;

impl pallet_assets::Config<MainAssetsInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = AssetBalance;
	type RemoveItemsLimit = frame_support::traits::ConstU32<1000>;
	type AssetId = AssetIdForTrustBackedAssets;
	type AssetIdParameter = parity_scale_codec::Compact<AssetIdForTrustBackedAssets>;
	type Currency = Balances;
	type CreateOrigin = NoAssetCreators; //assets can only be created by root
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = ConstU128<{ TEER }>;
	type AssetAccountDeposit = ConstU128<{ TEER }>;
	type MetadataDepositBase = ConstU128<{ TEER }>;
	type MetadataDepositPerByte = ConstU128<{ 10 * MILLITEER }>;
	type ApprovalDeposit = ConstU128<{ 10 * MILLITEER }>;
	type StringLimit = ConstU32<50>;
	type Freezer = ();
	type Holder = ();
	type Extra = ();
	type CallbackHandle = ();
	type WeightInfo = weights::pallet_assets::WeightInfo<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

#[cfg(feature = "runtime-benchmarks")]
pub struct AssetRegistryBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl pallet_asset_registry::BenchmarkHelper<AssetIdForTrustBackedAssets>
	for AssetRegistryBenchmarkHelper
{
	fn get_registered_asset() -> AssetIdForTrustBackedAssets {
		use sp_runtime::traits::StaticLookup;

		let root = frame_system::RawOrigin::Root.into();
		let asset_id = 1;
		let caller = frame_benchmarking::whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller);
		Assets::force_create(root, asset_id.into(), caller_lookup, true, 1)
			.expect("Should have been able to force create asset");
		asset_id
	}
}

impl pallet_asset_registry::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ReserveAssetModifierOrigin = EnsureRoot<Self::AccountId>;
	type Assets = Assets;
	type WeightInfo = weights::pallet_asset_registry::WeightInfo<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = AssetRegistryBenchmarkHelper;
}

pub type NativeAndAssets = UnionOf<
	Balances,
	Assets,
	NativeFromLeft,
	NativeOrWithId<AssetIdForTrustBackedAssets>,
	AccountId,
>;

pub type PoolIdToAccountId = pallet_asset_conversion::AccountIdConverter<
	AssetConversionPalletId,
	(NativeOrWithId<AssetIdForTrustBackedAssets>, NativeOrWithId<AssetIdForTrustBackedAssets>),
>;

pub type AscendingLocator =
	Ascending<AccountId, NativeOrWithId<AssetIdForTrustBackedAssets>, PoolIdToAccountId>;

pub type WithFirstAssetLocator = WithFirstAsset<
	Native,
	AccountId,
	NativeOrWithId<AssetIdForTrustBackedAssets>,
	PoolIdToAccountId,
>;

impl pallet_asset_conversion::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type HigherPrecisionBalance = sp_core::U256;
	type AssetKind = NativeOrWithId<AssetIdForTrustBackedAssets>;
	type Assets = NativeAndAssets;
	type PoolId = (Self::AssetKind, Self::AssetKind);
	type PoolLocator = Chain<WithFirstAssetLocator, AscendingLocator>;
	type PoolAssetId = u32;
	type PoolAssets = PoolAssets;
	type PoolSetupFee = ConstU128<0>; // Asset class deposit fees are sufficient to prevent spam
	type PoolSetupFeeAsset = Native;
	type PoolSetupFeeTarget = ResolveAssetTo<AssetConversionOrigin, Self::Assets>;
	type LiquidityWithdrawalFee = LiquidityWithdrawalFee;
	type LPFee = ConstU32<3>; // 0.3% swap fee
	type PalletId = AssetConversionPalletId;
	type MaxSwapPathLength = ConstU32<3>;
	type MintMinLiquidity = ConstU128<100>;
	type WeightInfo = weights::pallet_asset_conversion::WeightInfo<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

parameter_types! {
	pub AssetsPalletIndex: u32 = <Assets as PalletInfoAccess>::index() as u32;
	pub const AssetConversionPalletId: PalletId = PalletId(*b"py/ascon");
	pub const Native: NativeOrWithId<u32> = NativeOrWithId::Native;
	// we charge no fee for liquidity withdrawal
	pub const LiquidityWithdrawalFee: Permill = Permill::from_perthousand(0);
}

ord_parameter_types! {
	pub const AssetConversionOrigin: sp_runtime::AccountId32 =
		AccountIdConversion::<sp_runtime::AccountId32>::into_account_truncating(&AssetConversionPalletId::get());
}
pub type PoolAssetsInstance = pallet_assets::Instance2;

impl pallet_assets::Config<PoolAssetsInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type RemoveItemsLimit = ConstU32<1000>;
	type AssetId = u32;
	type AssetIdParameter = u32;
	type Currency = Balances;
	type CreateOrigin =
		AsEnsureOriginWithArg<EnsureSignedBy<AssetConversionOrigin, sp_runtime::AccountId32>>;
	type ForceOrigin = EnsureRoot<AccountId>;
	// Deposits are zero because creation/admin is limited to Asset Conversion pallet.
	type AssetDeposit = ConstU128<0>;
	type AssetAccountDeposit = ConstU128<0>;
	type MetadataDepositBase = ConstU128<0>;
	type MetadataDepositPerByte = ConstU128<0>;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = ConstU32<50>;
	type Holder = ();
	type Freezer = ();
	type Extra = ();
	type WeightInfo = weights::pallet_assets::WeightInfo<Runtime>;
	type CallbackHandle = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type EventHandler = (CollatorSelection,);
}

parameter_types! {
	pub const Period: u32 = 6 * HOURS;
	pub const Offset: u32 = 0;
}
impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type SessionManager = CollatorSelection;
	// Essentially just Aura, but let's be pedantic.
	type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type DisablingStrategy = ();
	type WeightInfo = weights::pallet_session::WeightInfo<Runtime>;
}

parameter_types! {
	pub const PotId: PalletId = PalletId(*b"PotStake");
	pub const SessionLength: BlockNumber = 6 * HOURS;
}

impl pallet_collator_selection::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type UpdateOrigin = EnsureRootOrMoreThanHalfCouncil;
	type PotId = PotId;
	type MaxCandidates = ConstU32<100>;
	type MinEligibleCollators = ConstU32<4>;
	type MaxInvulnerables = ConstU32<20>;
	// should be a multiple of session or things will get inconsistent
	type KickThreshold = Period;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = weights::pallet_collator_selection::WeightInfo<Runtime>;
}
impl pallet_sudo::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

construct_runtime!(
	pub enum Runtime
	{
		// Basic.
		System: frame_system = 0,
		ParachainSystem: cumulus_pallet_parachain_system = 1,
		// RandomnessCollectiveFlip: pallet_randomness_collective_flip = 2,
		Timestamp: pallet_timestamp = 3,
		ParachainInfo: staging_parachain_info = 4,
		Preimage: pallet_preimage = 5,
		Multisig: pallet_multisig = 6,
		Proxy: pallet_proxy = 7,
		Scheduler: pallet_scheduler = 8,
		Utility: pallet_utility = 9,

		// Funds and fees.
		Balances: pallet_balances = 10,
		TransactionPayment: pallet_transaction_payment = 11,
		Vesting: pallet_vesting = 12,

		Sudo: pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>} = 17,

		// Governance.
		Treasury: pallet_treasury = 13,
		Democracy: pallet_democracy = 14,
		Council: pallet_collective::<Instance1> = 15,
		TechnicalCommittee:
			pallet_collective::<Instance2> = 16,
		Bounties: pallet_bounties = 18,
		ChildBounties: pallet_child_bounties = 19,

		// Consensus.
		Authorship: pallet_authorship = 20,
		CollatorSelection: pallet_collator_selection = 21,
		Session: pallet_session = 22,
		Aura: pallet_aura = 23,
		AuraExt: cumulus_pallet_aura_ext = 24,

		// XCM helpers.
		XcmpQueue: cumulus_pallet_xcmp_queue = 30,
		PolkadotXcm: pallet_xcm = 31,
		CumulusXcm: cumulus_pallet_xcm = 32,
		MessageQueue: pallet_message_queue = 33,
		XTokens: orml_xtokens = 34,
		OrmlXcm: orml_xcm = 35,
		//XcmTransactor: pallet_xcm_transactor = 36,

		// fungibles
		Assets: pallet_assets::<Instance1> = 41,
		AssetRegistry: pallet_asset_registry = 42,
		AssetConversion: pallet_asset_conversion = 43,
		PoolAssets: pallet_assets::<Instance2> = 44,

		// Integritee pallets.
		Teerex: pallet_teerex = 50,
		Claims: pallet_claims = 51,
		Teeracle: pallet_teeracle = 52,
		Sidechain: pallet_sidechain= 53,
		EnclaveBridge: pallet_enclave_bridge = 54,
		TeerDays: pallet_teerdays = 55,
		Porteer: pallet_porteer = 56,
	}
);

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
	frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

parameter_types! {
	/// Weight for balance unreservations
	pub BalanceTransferAllowDeath: Weight = weights::pallet_balances::WeightInfo::<Runtime>::transfer_allow_death();
}

/// Migrations to apply on runtime upgrade.
pub type Migrations = (
	pallet_session::migrations::v1::MigrateV0ToV1<
		Runtime,
		pallet_session::migrations::v1::InitOffenceSeverity<Runtime>,
	>,
	cumulus_pallet_aura_ext::migration::MigrateV0ToV1<Runtime>,
	pallet_child_bounties::migration::MigrateV0ToV1<Runtime, BalanceTransferAllowDeath>,
	// permanent
	pallet_xcm::migration::MigrateToLatestXcmVersion<Runtime>,
);

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem, // Solochain: AllPalletsReversedWithSystemFirst, Statemint: AllPallets. Which one to take?
	Migrations,
>;

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	define_benchmarks!(
		[frame_system, SystemBench::<Runtime>]
		[pallet_asset_conversion, AssetConversion]
		[pallet_asset_registry, AssetRegistry]
		[pallet_assets, Assets]
		[pallet_balances, Balances]
		[pallet_bounties, Bounties]
		[pallet_child_bounties, ChildBounties]
		[pallet_claims, Claims]
		[pallet_collator_selection, CollatorSelection]
		[pallet_collective, Council]
		[pallet_democracy, Democracy]
		[pallet_message_queue, MessageQueue]
		[pallet_multisig, Multisig]
		[pallet_preimage, Preimage]
		[pallet_proxy, Proxy]
		[pallet_scheduler, Scheduler]
		[pallet_session, SessionBench::<Runtime>]
		[pallet_sidechain, Sidechain]
		[pallet_teeracle, Teeracle]
		[pallet_teerdays, TeerDays]
		[pallet_teerex, Teerex]
		[pallet_enclave_bridge, EnclaveBridge]
		[pallet_timestamp, Timestamp]
		[pallet_treasury, Treasury]
		[pallet_vesting, Vesting]
		[pallet_utility, Utility]
		[cumulus_pallet_xcmp_queue, XcmpQueue]
		[cumulus_pallet_parachain_system, ParachainSystem]
	);
}

impl_runtime_apis! {
	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			pallet_aura::Authorities::<Runtime>::get().into_inner()
		}
	}

	impl cumulus_primitives_aura::AuraUnincludedSegmentApi<Block> for Runtime {
		fn can_build_upon(
			included_hash: <Block as BlockT>::Hash,
			slot: cumulus_primitives_aura::Slot,
		) -> bool {
			ConsensusHook::can_build_upon(included_hash, slot)
		}
	}

	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> sp_std::vec::Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
		for Runtime
	{
		fn query_call_info(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}

	impl xcm_runtime_apis::fees::XcmPaymentApi<Block> for Runtime {
		fn query_acceptable_payment_assets(xcm_version: xcm::Version) -> Result<Vec<VersionedAssetId>, XcmPaymentApiError> {
			let acceptable_assets = vec![AssetId(xcm_config::SelfLocation::get())];
			PolkadotXcm::query_acceptable_payment_assets(xcm_version, acceptable_assets)
		}

		fn query_weight_to_asset_fee(weight: Weight, asset: VersionedAssetId) -> Result<u128, XcmPaymentApiError> {
			let latest_asset_id: Result<AssetId, ()> = asset.clone().try_into();
			match latest_asset_id {
				Ok(asset_id) if asset_id.0 == xcm_config::SelfLocation::get() => {
					Ok(WeightToFee::weight_to_fee(&weight))
				},
				Ok(asset_id) if asset_id.0 == xcm_config::RelayChainLocation::get() => {
					Ok(WeightToFee::weight_to_fee(&weight))
				},
				_ => todo!("Asset fee payment for {:?} not implemented", asset),
			}
		}

		fn query_xcm_weight(message: VersionedXcm<()>) -> Result<Weight, XcmPaymentApiError> {
			PolkadotXcm::query_xcm_weight(message)
		}

		fn query_delivery_fees(destination: VersionedLocation, message: VersionedXcm<()>) -> Result<VersionedAssets, XcmPaymentApiError> {
			PolkadotXcm::query_delivery_fees(destination, message)
		}
	}

	impl xcm_runtime_apis::dry_run::DryRunApi<Block, RuntimeCall, RuntimeEvent, OriginCaller> for Runtime {
		fn dry_run_call(origin: OriginCaller, call: RuntimeCall, result_xcms_version: XcmVersion) -> Result<CallDryRunEffects<RuntimeEvent>, XcmDryRunApiError> {
			PolkadotXcm::dry_run_call::<Runtime, xcm_config::XcmRouter, OriginCaller, RuntimeCall>(origin, call, result_xcms_version)
		}

		fn dry_run_xcm(origin_location: VersionedLocation, xcm: VersionedXcm<RuntimeCall>) -> Result<XcmDryRunEffects<RuntimeEvent>, XcmDryRunApiError> {
			PolkadotXcm::dry_run_xcm::<Runtime, xcm_config::XcmRouter, RuntimeCall, xcm_config::XcmConfig>(origin_location, xcm)
		}
	}

	impl xcm_runtime_apis::conversions::LocationToAccountApi<Block, AccountId> for Runtime {
		fn convert_location(location: VersionedLocation) -> Result<
			AccountId,
			xcm_runtime_apis::conversions::Error
		> {
			xcm_runtime_apis::conversions::LocationToAccountHelper::<
				AccountId,
				xcm_config::LocationToAccountId,
			>::convert_location(location)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, RuntimeBlockWeights::get().max_block)
		}

		fn execute_block(
			block: Block,
			state_root_check: bool,
			signature_check: bool,
			select: frame_try_runtime::TryStateSelect,
		) -> Weight {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here.
			Executive::try_execute_block(block, state_root_check, signature_check, select).unwrap()
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();
			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, BenchmarkError};
			use sp_storage::TrackedStorageKey;
			use frame_system_benchmarking::Pallet as SystemBench;
			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			impl cumulus_pallet_session_benchmarking::Config for Runtime {}

			impl frame_system_benchmarking::Config for Runtime {
				fn setup_set_code_requirements(code: &sp_std::vec::Vec<u8>) -> Result<(), BenchmarkError> {
					ParachainSystem::initialize_for_set_code_benchmark(code.len() as u32);
					Ok(())
				}

				fn verify_set_code() {
					System::assert_last_event(cumulus_pallet_parachain_system::Event::<Runtime>::ValidationFunctionStored.into());
				}
			}
			// Whitelisted keys to be ignored in benchmarking DB-access tracking.
			//
			// Reasoning:
			// 		Previously accessed storage keys are stored in the `StorageOverlay`, i.e. the runtime cache.
			// 		A cache with the life-time of one block.
			// 		Accessing these keys afterwards in the same block is considered as negligible overhead.
			// 		Hence, we whitelist storage keys that are accessed every block anyhow because accessing them
			// 		in our pallet can be considered free.
			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_state::<RuntimeGenesisConfig>(config)
		}

		fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
			get_preset::<RuntimeGenesisConfig>(id, |_| None)
		}

		fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
			Default::default()
		}
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
}
