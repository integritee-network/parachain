// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Common modules for all parachains.
//!
//! Largely inspired by cumulus/parachains/common/

#![cfg_attr(not(feature = "std"), no_std)]

// the two mods do not exist upstream
pub mod currency;
pub mod fee;

pub mod xcm_config;
pub use constants::*;
pub use types::*;

/// Common types of parachains.
mod types {
    use sp_runtime::{
        generic,
        traits::{BlakeTwo256, IdentifyAccount, Verify},
    };

    /// An index to a block.
    pub type BlockNumber = u32;

    /// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
    pub type Signature = sp_runtime::MultiSignature;

    /// Some way of identifying an account on the chain. We intentionally make it equivalent
    /// to the public key of our transaction signing scheme.
    pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

    /// The type for looking up accounts. We don't expect more than 4 billion of them, but you
    /// never know...
    pub type AccountIndex = u32;

    /// Balance of an account.
    pub type Balance = u128;

    /// Nonce of a transaction in the chain.
    pub type Nonce = u32;

    /// A hash of some data used by the chain.
    pub type Hash = sp_core::H256;

    /// Digest item type.
    pub type DigestItem = sp_runtime::generic::DigestItem;

    // Aura consensus authority.
    pub type AuraId = sp_consensus_aura::sr25519::AuthorityId;

    // Id used for identifying assets.
    pub type AssetIdForTrustBackedAssets = u32;

    /// The address format for describing accounts.
    pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
    /// Block header type as expected by this runtime.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
}

/// Common constants of parachains.
mod constants {
    use super::types::BlockNumber;
    use frame_support::weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight};
    use polkadot_core_primitives::Moment;
    use sp_runtime::Perbill;
    /// This determines the average expected block time that we are targeting. Blocks will be
    /// produced at a minimum duration defined by `SLOT_DURATION`. `SLOT_DURATION` is picked up by
    /// `pallet_timestamp` which is in turn picked up by `pallet_aura` to implement `fn
    /// slot_duration()`.
    ///
    /// Change this to adjust the block time.
    pub const MILLISECS_PER_BLOCK: u64 = 12000;
    pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

    // Time is measured by number of blocks.
    pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;

    // Time helpers in milliseconds.
    pub const MS_PER_MINUTE: Moment = 60_000;
    pub const MS_PER_HOUR: Moment = crate::MS_PER_MINUTE * 60;
    pub const MS_PER_DAY: Moment = crate::MS_PER_HOUR * 24;

    /// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
    /// used to limit the maximal weight of a single extrinsic.
    pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);
    /// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
    /// Operational  extrinsics.
    pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

    /// We allow for 0.5 seconds of compute with a 6 second average block time.
    pub const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
        WEIGHT_REF_TIME_PER_SECOND.saturating_div(2),
        polkadot_primitives::MAX_POV_SIZE as u64,
    );

    /// Maximum number of blocks simultaneously accepted by the Runtime, not yet included
    /// into the relay chain.
    pub const UNINCLUDED_SEGMENT_CAPACITY: u32 = 1;
    /// How many parachain blocks are processed by the relay chain per parent. Limits the
    /// number of blocks authored per slot.
    pub const BLOCK_PROCESSING_VELOCITY: u32 = 1;
    /// Relay chain slot duration, in milliseconds.
    pub const RELAY_CHAIN_SLOT_DURATION_MILLIS: u32 = 6000;
}

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    use super::*;
    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
    use sp_runtime::{
        generic,
        traits::{BlakeTwo256, Hash as HashT},
    };
    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;
    /// Opaque block hash type.
    pub type Hash = <BlakeTwo256 as HashT>::Output;
}
