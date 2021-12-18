#![cfg_attr(not(feature = "std"), no_std)]
use sp_std::prelude::*;
pub use pallet::*;
use pallet_secrets;
// #[cfg(test)]
// mod tests;

// #[cfg(test)]
// mod mock;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

pub type SecretId = u64;
pub type CallIndex = u64;
pub type EncodedCall = Vec<u8>;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use super::*;
	
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_secrets::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		type CallLength: Get<u32>;
		// type ForceOrigin: EnsureOrigin<Self::Origin>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// index of EncodedCall ALWAYS should equals to its index
	#[pallet::storage]
	#[pallet::getter(fn call_history_of)]
	pub(super) type CallHistory<T: Config> = StorageMap<_, Blake2_128Concat, SecretId, Vec<EncodedCall> >;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ContractRegistered(SecretId),
		CallReceived(SecretId, CallIndex),
		
		// should also include the events bubbled up?
		CallFullfilled(SecretId, CallIndex),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalideEncodedCall,
		ContrtactIndexError,
		CallIndexError,
	}

	#[pallet::call]
	impl<T:Config> Pallet<T> {

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 3))]
		pub fn register_contract(
			origin: OriginFor<T>, 
			metadata: Vec<u8>,
			contract_public_key: Vec<u8>,
			initializatio_call: EncodedCall,
		) -> DispatchResult {
			// let who = ensure_signed(origin)?;

			ensure!(Self::validate_call(initializatio_call.clone()), Error::<T>::InvalideEncodedCall);
			match pallet_secrets::Pallet::<T>::register_secret_contract(
				origin, metadata, contract_public_key
			) {
				Ok(()) => {
					// get the lastest secretId - 1 -> it belongs to the secret we have just created
					let contract_index = pallet_secrets::Pallet::<T>::current_secret_id().saturating_sub(1);

					// the first call
					Self::insert_call(contract_index, 0, initializatio_call);

					Self::deposit_event(Event::<T>::ContractRegistered(contract_index));
					
					Ok(())
				},
				Err(err) => Err(err)
			}
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 3))]
		pub fn call(
			origin: OriginFor<T>, 
			contract_index: SecretId,
			call: EncodedCall,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::validate_call(call.clone()), Error::<T>::InvalideEncodedCall);
			
			// 1. get the next call index
			// 2. insert the call
			
			if Self::insert_call(contract_index, 0, call) {
				Self::deposit_event(Event::<T>::CallReceived(contract_index, 0));
				Ok(())
			} else {
				Err(Error::<T>::CallIndexError.into())
			}
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn validate_call(
			call: EncodedCall,
		) -> bool {
			call.len() < T::CallLength::get() as usize
		}

		pub fn insert_call(
			contract_index: SecretId,
			call_index: CallIndex,
			call: EncodedCall,
		) -> bool {
			
			match <CallHistory<T>>::get(&contract_index) {
				Some(history) => {
					match history.len() as usize {
						call_index => {
							return true;
							// insert the call
						},
						_ => false
					}
				},
				None => false
			}
		}
	}
}

