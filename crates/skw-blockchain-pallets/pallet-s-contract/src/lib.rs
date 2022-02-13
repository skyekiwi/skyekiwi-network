#![cfg_attr(not(feature = "std"), no_std)]
use sp_std::prelude::*;
pub use pallet::*;

pub use pallet_secrets;
use pallet_secrets::SecretId;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

pub type CallIndex = u64;
pub type EncodedCall = Vec<u8>;
pub type ShardId = u64;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use super::*;
	
	#[pallet::config]
	pub trait Config: 
		frame_system::Config + 
		pallet_secrets::Config
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		type MaxCallLength: Get<u32>;

		#[pallet::constant]
		type MaxOutputLength: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// index of EncodedCall ALWAYS should equals to its CallIndex
	#[pallet::storage]
	#[pallet::getter(fn call_history_of)]
	pub(super) type CallHistory<T: Config> = StorageMap<_, Twox64Concat,
		ShardId, Vec<(T::BlockNumber, EncodedCall, T::AccountId)> >;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CallReceived(SecretId, CallIndex),
		CallFullfilled(SecretId, CallIndex, Vec<u8>),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidEncodedCall,
		InvalidContractIndex,
		InvalidCallIndex,
		InvalidCallOutput,
		Unauthorized, 
		Unexpected,
	}

	#[pallet::call]
	impl<T:Config> Pallet<T> {

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 3))]
		pub fn register_contract(
			origin: OriginFor<T>, 
			metadata: Vec<u8>,
			wasm_blob_cid: Vec<u8>,
			initialization_call: EncodedCall,
			shard_id: ShardId,
		) -> DispatchResult {
			let deployer = ensure_signed(origin.clone())?;

			ensure!(
				Self::validate_call(initialization_call.clone()),
				Error::<T>::InvalidEncodedCall
			);

			match pallet_secrets::Pallet::<T>::register_secret_contract(
				origin, metadata, wasm_blob_cid, shard_id
			) {
				Ok(()) => {
					// TODO: is this always right??
					// get the lastest secretId - 1 -> it belongs to the secret we have just created
					let contract_index = pallet_secrets::Pallet::<T>::current_secret_id().saturating_sub(1);

					// the init call
					Self::try_insert_call(contract_index, initialization_call, deployer, true);
					Ok(())
				},
				Err(err) => Err(err)
			}
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 3))]
		pub fn push_call(
			origin: OriginFor<T>, 
			contract_index: SecretId,
			call: EncodedCall,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::validate_call(call.clone()), Error::<T>::InvalidEncodedCall);
			
			match Self::try_insert_call(contract_index, call, who, false) {
				Some(call_index) => {
					Self::deposit_event(Event::<T>::CallReceived(contract_index, call_index));
					Ok(())
				},
				None => {
					Err(Error::<T>::InvalidCallIndex.into())
				}
			}
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn validate_call(call: EncodedCall,) -> bool {
			call.len() < T::MaxCallLength::get() as usize
		}

		pub fn validate_output(output: &Vec<u8>,) -> bool {
			output.len() < T::MaxOutputLength::get() as usize
		}

		pub fn try_insert_call(
			contract_index: SecretId,
			call: EncodedCall,
			origin: T::AccountId,
			force_insert: bool
		) -> Option<CallIndex> {
			let mut history = Self::call_history_of(&contract_index);
			if history == None {
				if force_insert {
					history = Some(Vec::new());
				} else {
					return None;
				}
			}

			if let Some(mut content) = history {
				let now = frame_system::Pallet::<T>::block_number();
				content.push((now, call, origin));
				<CallHistory<T>>::insert(&contract_index, content.clone());
				Some(content.len() as u64)
			} else {
				None
			}
		}

		pub fn get_call(
			contract_index: SecretId,
			call_index: CallIndex
		) -> Option<(T::BlockNumber, EncodedCall, T::AccountId)> {
			match Self::call_history_of(&contract_index) {
				Some(history) => {
					match history.get(call_index as usize) {
						Some(res) => { Some( res.clone()) },
						None => None
					}
				},
				None => None
			}
		}
	}
}

