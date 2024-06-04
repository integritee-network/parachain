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
	const TARGET: &str = "runtime::fix::scheduler::migration";

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
