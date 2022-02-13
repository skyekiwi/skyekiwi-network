#![cfg_attr(not(feature = "std"), no_std)]
use sp_std::prelude::*;
pub use pallet::*;


#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

pub type CallIndex = u64;
pub type ShardId = u64;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use super::*;
	
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_registry::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		type DeplayThreshold: Get<<Self as frame_system::Config>::BlockNumber>;

		#[pallet::constant]
		type MaxOutcomePerSubmission: Get<u64>;

		#[pallet::constant]
		type MaxSizePerOutcome: Get<u64>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);
	
	#[pallet::storage]
	#[pallet::getter(fn shard_confirmation_threshold)]
	pub(super) type ShardConfirmationThreshold<T: Config> = StorageMap<_, Twox64Concat, 
		ShardId, u64>;

	#[pallet::storage]
	#[pallet::getter(fn state_root_at)]
	pub(super) type StateRoot<T: Config> = StorageDoubleMap<_, Twox64Concat, ShardId, 
		Twox64Concat, T::BlockNumber, [u8; 32]>;
	
	#[pallet::storage]
	#[pallet::getter(fn state_file_hash_at)]
	pub(super) type StateFileHash<T: Config> = StorageDoubleMap<_, Twox64Concat, ShardId,
	Twox64Concat, T::BlockNumber, [u8; 32]>;

	#[pallet::storage]
	#[pallet::getter(fn confirmation_of)]
	pub(super) type Confirmation<T: Config> = StorageDoubleMap<_, Twox64Concat, ShardId,
	Twox64Concat, T::BlockNumber, u64>;

	#[pallet::storage]
	#[pallet::getter(fn outcome_of)]
	pub(super) type Outcome<T: Config> = StorageMap<_, Twox64Concat, 
		CallIndex, Vec<u8>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		BlockSynced(T::BlockNumber),
		BlockConfirmed(T::BlockNumber),
	}

	#[pallet::error]
	pub enum Error<T> {
		Unauthorized,
		OutcomeSubmissionTooLate,
		InvalidShardId,
		InvalidOutcome,
		InconsistentState,
		Unexpected,
	}

	#[pallet::call]
	impl<T:Config> Pallet<T> {

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(0, 1))]
		pub fn set_shard_confirmation_threshold(
			origin: OriginFor<T>,
			shard_id: ShardId,
			threshold: u64,
		) -> DispatchResult {
			ensure_root(origin)?;
			<ShardConfirmationThreshold<T>>::mutate(&shard_id, |t| {
				* t = Some(threshold)
			});
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(8, 10))]
		pub fn submit_outcome(
			origin: OriginFor<T>,
			block_number: T::BlockNumber,
			shard_id: ShardId,

			state_root: [u8; 32],
			state_file_hash: [u8; 32],

			outcome_call_index: Vec<CallIndex>,
			outcome: Vec<Vec<u8>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			
			ensure!(pallet_registry::Pallet::<T>::is_valid_shard_id(shard_id), Error::<T>::InvalidShardId);
			ensure!(pallet_registry::Pallet::<T>::is_valid_secret_keeper(&who), Error::<T>::Unauthorized);
			let now = frame_system::Pallet::<T>::block_number();
			ensure!(now <= block_number + T::DeplayThreshold::get(), Error::<T>::OutcomeSubmissionTooLate);

			// by default - confirmation at 1
			let threshold = Self::shard_confirmation_threshold(shard_id).unwrap_or(1);

			ensure!(pallet_registry::Pallet::<T>::is_beacon_turn(block_number, &who, shard_id, threshold), Error::<T>::Unauthorized);
			ensure!(outcome_call_index.len() == outcome.len(), Error::<T>::InvalidOutcome);
			ensure!(outcome_call_index.len() < T::MaxOutcomePerSubmission::get() as usize, Error::<T>::InvalidOutcome);

			for o in outcome.iter() {
				ensure!(Self::validate_outcome(&o), Error::<T>::InvalidOutcome);
			}

			let old_state_root = Self::state_root_at(shard_id, block_number);
			// 1. existing state_root doesnt match the current state_root 
			if old_state_root != Some(state_root) {
				if Self::confirmation_of(shard_id, block_number).is_some() {
					// 2. Someone had written another state_root before!!
					// THIS IS VERY SERIOUS
					return Err(Error::<T>::InconsistentState.into());
				}

				// 3. The record has never been written before
				// We can write it - exit the if statement
			} else {
				// 4. old state root matches the current state_root
				// 5. update the confirmation count
				<Confirmation<T>>::mutate(&shard_id, &block_number, |confirmation| {
					*confirmation = Some(confirmation.unwrap_or(0) + 1);
				});
				let confirms = Self::confirmation_of(shard_id, block_number).unwrap_or(0);
				ensure!(confirms <= threshold, Error::<T>::Unauthorized);

				// 6. threshold has been met. The block is confirmed!
				if confirms == threshold {
					Self::deposit_event(Event::<T>::BlockConfirmed(block_number));
				}

				return Ok(());
			}

			// The record has never been written before!

			<Confirmation<T>>::mutate(&shard_id, &block_number, |confirmation| {
				*confirmation = Some(confirmation.unwrap_or(0) + 1);
			});
			let confirms = Self::confirmation_of(shard_id, block_number).unwrap_or(0);
			
			// this should never happen as we have checked for specific secret keepers!
			ensure!(confirms <= threshold, Error::<T>::Unexpected);

			for (i, call_index) in outcome_call_index.iter().enumerate() {
				<Outcome<T>>::insert(&call_index, &outcome[i]);
			}

			<StateRoot<T>>::insert(&shard_id, &block_number, &state_root);
			<StateFileHash<T>>::insert(&shard_id, &block_number, &state_file_hash);

			// emit BlockSynced only at the first time!
			Self::deposit_event(Event::<T>::BlockSynced(block_number));

			if threshold == 1 {
				// if this is the first time the state is syced & threshold == 1
				// the block is confirmed!
				Self::deposit_event(Event::<T>::BlockConfirmed(block_number));
			}
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn validate_outcome(call: &Vec<u8>) -> bool {
			call.len() < T::MaxSizePerOutcome::get() as usize
		}
	}
}

