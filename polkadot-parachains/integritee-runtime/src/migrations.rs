pub mod scheduler {
	// this is necessary because migrations from v0 to v3 are no longer available in the scheduler
	// pallet code and migrating is only possible from v3. The strategy here is to empty the agenda
	use frame_support::traits::OnRuntimeUpgrade;
	use frame_system::pallet_prelude::BlockNumberFor;
	use pallet_scheduler::*;
	use sp_std::vec::Vec;

	#[cfg(feature = "try-runtime")]
	use sp_runtime::TryRuntimeError;

	/// The log target.
	const TARGET: &'static str = "runtime::fix::scheduler::migration";

	pub mod v1 {
		use super::*;
		use frame_support::{pallet_prelude::*, traits::schedule};

		#[cfg_attr(any(feature = "std", test), derive(PartialEq, Eq))]
		#[derive(Clone, RuntimeDebug, Encode, Decode)]
		pub(crate) struct ScheduledV1<Call, BlockNumber> {
			maybe_id: Option<Vec<u8>>,
			priority: schedule::Priority,
			call: Call,
			maybe_periodic: Option<schedule::Period<BlockNumber>>,
		}

		#[frame_support::storage_alias]
		pub(crate) type Agenda<T: Config> = StorageMap<
			Pallet<T>,
			Twox64Concat,
			BlockNumberFor<T>,
			Vec<Option<ScheduledV1<<T as Config>::RuntimeCall, BlockNumberFor<T>>>>,
			ValueQuery,
		>;

		#[frame_support::storage_alias]
		pub(crate) type Lookup<T: Config> =
			StorageMap<Pallet<T>, Twox64Concat, Vec<u8>, TaskAddress<BlockNumberFor<T>>>;
	}

	pub mod v3 {
		use super::*;
		use frame_support::pallet_prelude::*;

		#[frame_support::storage_alias]
		pub(crate) type Agenda<T: Config> = StorageMap<
			Pallet<T>,
			Twox64Concat,
			BlockNumberFor<T>,
			Vec<Option<ScheduledV3Of<T>>>,
			ValueQuery,
		>;

		#[frame_support::storage_alias]
		pub(crate) type Lookup<T: Config> =
			StorageMap<Pallet<T>, Twox64Concat, Vec<u8>, TaskAddress<BlockNumberFor<T>>>;
	}

	pub mod v4 {
		use super::*;
		use frame_support::pallet_prelude::*;

		#[frame_support::storage_alias]
		pub type Agenda<T: Config> = StorageMap<
			Pallet<T>,
			Twox64Concat,
			BlockNumberFor<T>,
			BoundedVec<
				Option<ScheduledOf<T>>,
				<T as pallet_scheduler::Config>::MaxScheduledPerBlock,
			>,
			ValueQuery,
		>;

		#[cfg(feature = "try-runtime")]
		pub(crate) type TaskName = [u8; 32];

		#[cfg(feature = "try-runtime")]
		#[frame_support::storage_alias]
		pub(crate) type Lookup<T: Config> =
			StorageMap<Pallet<T>, Twox64Concat, TaskName, TaskAddress<BlockNumberFor<T>>>;

		/// brute-force empty the agenda for V4.
		pub struct PurgeV4Agenda<T>(sp_std::marker::PhantomData<T>);

		impl<T: Config> OnRuntimeUpgrade for PurgeV4Agenda<T> {
			#[cfg(feature = "try-runtime")]
			fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
				let agendas = v1::Agenda::<T>::iter_keys().count() as u32;
				let lookups = v1::Lookup::<T>::iter_keys().count() as u32;
				log::info!(target: TARGET, "agendas present which will be dropped: {}/{}...", agendas, lookups);
				Ok((agendas, lookups).encode())
			}

			fn on_runtime_upgrade() -> Weight {
				let onchain_version = Pallet::<T>::on_chain_storage_version();
				if onchain_version != 4 {
					log::warn!(
						target: TARGET,
						"skipping migration: executed on wrong storage version.\
					Expected version == 4, found {:?}",
						onchain_version,
					);
					return T::DbWeight::get().reads(1)
				}
				log::info!(target: TARGET, "migrating from {:?} to 4, purging agenda", onchain_version);
				let purged_agendas = v1::Agenda::<T>::clear(u32::MAX, None).unique as u64;
				let purged_lookups = v1::Lookup::<T>::clear(u32::MAX, None).unique as u64;
				T::DbWeight::get()
					.reads_writes(purged_agendas + purged_lookups, purged_agendas + purged_lookups)
			}

			#[cfg(feature = "try-runtime")]
			fn post_upgrade(_state: Vec<u8>) -> Result<(), TryRuntimeError> {
				ensure!(StorageVersion::get::<Pallet<T>>() == 4, "Must upgrade");

				let agendas = Agenda::<T>::iter_keys().count() as u32;
				ensure!(agendas == 0, "agenda must be empty after now");
				let lookups = Lookup::<T>::iter_keys().count() as u32;
				ensure!(lookups == 0, "agenda must be empty after now");

				Ok(())
			}
		}
	}
}

pub mod collator_selection_init {
	use frame_support::traits::OnRuntimeUpgrade;
	#[cfg(feature = "try-runtime")]
	use sp_runtime::TryRuntimeError;

	/// The log target.
	const TARGET: &str = "runtime::fix::collator_selection_init";
	pub mod v0 {
		use super::*;
		use crate::SessionKeys;
		use frame_support::{pallet_prelude::*, traits::Currency};
		use hex_literal::hex;
		use log::info;
		use parachains_common::impls::BalanceOf;
		use parity_scale_codec::EncodeLike;
		use sp_core::{crypto::key_types::AURA, sr25519};
		use sp_std::{vec, vec::Vec};

		const INVULNERABLE_AURA_A: [u8; 32] =
			hex!("c6c1370a5b6656f3816b2a6c32444ec18d5ac6d33103c4e5c3f359623a19dc47");
		const INVULNERABLE_AURA_B: [u8; 32] =
			hex!("183490bfadaacbb875537599dfc936cb0159eadf9a4cc8a16a584f170b503509");
		const INVULNERABLE_AURA_C: [u8; 32] =
			hex!("a8302634ae0c688c7c3447e4e683279ec9384c1758ec78a7a2def44064cf046c");
		const INVULNERABLE_AURA_D: [u8; 32] =
			hex!("e2750081920a4a704fae7f04069df3018dd259ca2c6af51a9764df226072f75a");
		const INVULNERABLE_AURA_E: [u8; 32] =
			hex!("ba492297546f3c34602551582069b03c0e13000d5de928e9b5db9d18bbd2e435");

		const INVULNERABLE_ACCOUNT_A: [u8; 32] =
			hex!("8e9f7d54e1d9bdbac609f444c9e920a87af41216ee6c9ba0c62032fb1ade0464");
		const INVULNERABLE_ACCOUNT_B: [u8; 32] =
			hex!("b0a7c84862ec760d8817ab01482e955332c9abf11e4568991def6837c1e1ac7b");
		const INVULNERABLE_ACCOUNT_C: [u8; 32] =
			hex!("1a65141f43b54ed8c0f76201ef374a607bd74484741243212a2accd76e315c09");
		const INVULNERABLE_ACCOUNT_D: [u8; 32] =
			hex!("645553b30ea9a250dd0f38f472fdeb04e418b8204b96a2bd30ae151ef660053e");
		const INVULNERABLE_ACCOUNT_E: [u8; 32] =
			hex!("e089d5404fb036a8ac795377213445f9ccebfc6773a183769503974369db7f46");

		pub struct InitInvulnerables<T>(sp_std::marker::PhantomData<T>);
		impl<T> OnRuntimeUpgrade for InitInvulnerables<T>
		where
			T: frame_system::Config
				+ pallet_collator_selection::Config
				+ pallet_session::Config
				+ pallet_balances::Config,
			<T as frame_system::Config>::AccountId: From<[u8; 32]>,
			<T as pallet_session::Config>::ValidatorId: From<[u8; 32]>,
			<T as pallet_session::Config>::Keys: From<SessionKeys>,
			<T as pallet_balances::Config>::Balance: From<u128>,
			<T as pallet_balances::Config>::Balance: EncodeLike<
				<<T as pallet_collator_selection::Config>::Currency as Currency<
					<T as frame_system::Config>::AccountId,
				>>::Balance,
			>,
		{
			#[cfg(feature = "try-runtime")]
			fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, TryRuntimeError> {
				let invulnerables_len = pallet_collator_selection::Invulnerables::<T>::get().len();
				Ok((invulnerables_len as u32).encode())
			}

			fn on_runtime_upgrade() -> Weight {
				let invulnerables_len = pallet_collator_selection::Invulnerables::<T>::get().len();
				if invulnerables_len > 0 {
					info!(target: TARGET, "no need to initialize invulnerables");
					return T::DbWeight::get().reads_writes(1, 0)
				}
				info!(target: TARGET, "initializing the set of invulnerables");

				let raw_aura_keys: Vec<[u8; 32]> = vec![
					INVULNERABLE_AURA_A,
					INVULNERABLE_AURA_B,
					INVULNERABLE_AURA_C,
					INVULNERABLE_AURA_D,
					INVULNERABLE_AURA_E,
				];
				let raw_account_keys: Vec<[u8; 32]> = vec![
					INVULNERABLE_ACCOUNT_A,
					INVULNERABLE_ACCOUNT_B,
					INVULNERABLE_ACCOUNT_C,
					INVULNERABLE_ACCOUNT_D,
					INVULNERABLE_ACCOUNT_E,
				];

				let validatorids: Vec<<T as pallet_session::Config>::ValidatorId> =
					raw_account_keys.iter().map(|&pk| pk.into()).collect();

				pallet_session::Validators::<T>::put(validatorids);

				let queued_keys: Vec<(
					<T as pallet_session::Config>::ValidatorId,
					<T as pallet_session::Config>::Keys,
				)> = raw_account_keys
					.iter()
					.zip(raw_aura_keys.iter())
					.map(|(&account, &aura)| {
						(
							account.into(),
							SessionKeys { aura: sr25519::Public::from_raw(aura).into() }.into(),
						)
					})
					.collect();

				pallet_session::QueuedKeys::<T>::put(queued_keys);

				for (&account, &aura) in raw_account_keys.iter().zip(raw_aura_keys.iter()) {
					pallet_session::NextKeys::<T>::insert::<
						<T as pallet_session::Config>::ValidatorId,
						<T as pallet_session::Config>::Keys,
					>(
						account.into(),
						SessionKeys { aura: sr25519::Public::from_raw(aura).into() }.into(),
					);
					pallet_session::KeyOwner::<T>::insert::<
						_,
						<T as pallet_session::Config>::ValidatorId,
					>((AURA, aura.encode()), account.into());
				}

				let mut invulnerables: Vec<<T as frame_system::Config>::AccountId> =
					raw_account_keys.iter().map(|&pk| pk.into()).collect();
				invulnerables.sort();
				let invulnerables: BoundedVec<_, T::MaxInvulnerables> =
					invulnerables.try_into().unwrap();
				pallet_collator_selection::Invulnerables::<T>::put(invulnerables);

				pallet_collator_selection::CandidacyBond::<T>::put::<BalanceOf<T>>(
					5_000_000_000_000u128.into(),
				);

				T::DbWeight::get().reads_writes(0, 4 + 5 * 2)
			}

			#[cfg(feature = "try-runtime")]
			fn post_upgrade(state: sp_std::vec::Vec<u8>) -> Result<(), TryRuntimeError> {
				let invulnerables_len =
					pallet_collator_selection::Invulnerables::<T>::get().len() as u32;
				let apriori_invulnerables_len: u32 = Decode::decode(&mut state.as_slice()).expect(
					"the state parameter should be something that was generated by pre_upgrade",
				);
				ensure!(
					invulnerables_len > 0,
					"invulnerables are empty after initialization. that should not happen"
				);
				info!(target: TARGET, "apriori invulnerables: {}, aposteriori: {}", apriori_invulnerables_len, invulnerables_len);
				Ok(())
			}
		}
	}
}
