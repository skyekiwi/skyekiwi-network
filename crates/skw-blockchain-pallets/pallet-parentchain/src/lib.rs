#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use skw_blockchain_primitives::types::{ShardId, CallIndex};
	use sp_std::vec::Vec;	
	use super::WeightInfo;
	
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_registry::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type WeightInfo: WeightInfo;

		/// Maximum delay between receiving a call and submitting result for it
		#[pallet::constant]
		type DelayThreshold: Get<<Self as frame_system::Config>::BlockNumber>;

		/// Maximum number of outcomes allowed per submission 
		#[pallet::constant]
		type MaxOutcomePerSubmission: Get<u32>;

		/// Maximum length of sizze for each outcome submitted
		#[pallet::constant]
		type MaxSizePerOutcome: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);
	
	/// Threshold of outcome submission from SecretKeepers needed to confirm a block
	#[pallet::storage]
	#[pallet::getter(fn shard_confirmation_threshold)]
	pub(super) type ShardConfirmationThreshold<T: Config> = StorageMap<_, Twox64Concat, 
		ShardId, u64>;

	/// state_root of offchain runtime
	#[pallet::storage]
	#[pallet::getter(fn state_root_at)]
	pub(super) type StateRoot<T: Config> = StorageDoubleMap<_, Twox64Concat, ShardId, 
		Twox64Concat, T::BlockNumber, [u8; 32]>;

	/// confirmations received for offchain runtime for blocks 
	#[pallet::storage]
	#[pallet::getter(fn confirmation_of)]
	pub(super) type Confirmation<T: Config> = StorageDoubleMap<_, Twox64Concat, ShardId,
	Twox64Concat, T::BlockNumber, u64>;

	/// outcome received each call 
	#[pallet::storage]
	#[pallet::getter(fn outcome_of)]
	pub(super) type Outcome<T: Config> = StorageMap<_, Twox64Concat, 
		CallIndex, BoundedVec<u8, T::MaxSizePerOutcome>>;

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

		/// (ROOT ONLY) set the confirmation threshold for a shard
		#[pallet::weight(<T as Config>::WeightInfo::set_shard_confirmation_threshold())]
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

		/// submit a batch of outcomes for a block
		#[pallet::weight(<T as Config>::WeightInfo::submit_outcome(outcome_call_index.len() as u32))]
		pub fn submit_outcome(
			origin: OriginFor<T>,
			block_number: T::BlockNumber, shard_id: ShardId,

			state_root: [u8; 32],

			outcome_call_index: Vec<CallIndex>,
			outcome: Vec<Vec<u8>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
		
			// TODO: validate outcome
			ensure!(pallet_registry::Pallet::<T>::is_valid_shard_id(shard_id), Error::<T>::InvalidShardId);
			ensure!(pallet_registry::Pallet::<T>::is_valid_secret_keeper(&who), Error::<T>::Unauthorized);
			let now = frame_system::Pallet::<T>::block_number();
			ensure!(now <= block_number + T::DelayThreshold::get(), Error::<T>::OutcomeSubmissionTooLate);

			// by default - confirmation at 1
			let threshold = Self::shard_confirmation_threshold(shard_id).unwrap_or(1);

			// TODO: maybe we should allow any submission and let clients handle the rest
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
				let bounded_outcome = BoundedVec::<u8, T::MaxSizePerOutcome>::try_from(outcome[i].clone())
					.map_err(|_| Error::<T>::InvalidOutcome)?;

				<Outcome<T>>::insert(&call_index, &bounded_outcome);
			}

			<StateRoot<T>>::insert(&shard_id, &block_number, &state_root);

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
		pub fn validate_outcome(outcome: &Vec<u8>) -> bool {
			outcome.len() < T::MaxSizePerOutcome::get() as usize
		}
	}
}
