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
	use frame_support::{
		dispatch::DispatchResult, pallet_prelude::*,
		traits::{
			Currency, ExistenceRequirement::KeepAlive
		}, Twox64Concat
	};
	use frame_support::sp_runtime::traits::AccountIdConversion;
	use frame_system::pallet_prelude::*;
	use super::*;
	use sp_std::vec::Vec;
	
	use skw_blockchain_primitives::{ShardId};	
	pub type BalanceOf<T> = pallet_treasury::BalanceOf<T>;

	#[pallet::config]
	pub trait Config: frame_system::Config 
			+ pallet_s_contract::Config
			+ pallet_treasury::Config  
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type WeightInfo: WeightInfo;

		/// registration reserving requirement
		type ReservationRequirement: Get<BalanceOf <Self>>;

		/// default offchain gas dispense amount
		type DefaultFaucet: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);
	
	/// reserved amount of each account 
	#[pallet::storage]
	#[pallet::getter(fn reserved_amount_of)]
	pub(super) type ReservedAmount <T: Config> = StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, ShardId, BalanceOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		EnclaveAccountCreated(T::AccountId, ShardId),
	}

	#[pallet::error]
	pub enum Error<T> {
		Unauthorized,
		InsufficientBalance,
		MissingProof,
		Unexpected,
	}

	#[pallet::call]
	impl<T:Config> Pallet<T> {

		/// (ROOT ONLY) force create an account inside the enclave
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_shard_confirmation_threshold())]
		pub fn force_create_enclave_account(
			origin: OriginFor<T>,
			shard_id: ShardId,
			account: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin.clone())?;
			
			let encoded_call = Self::build_account_creation_call(&account, shard_id);
			pallet_s_contract::Pallet::<T>::force_push_call(origin, shard_id, encoded_call)?;

			Self::deposit_event(Event::<T>::EnclaveAccountCreated(account, shard_id));
			Ok(())
		}

		/// reserve some token and create an account in the enclave 
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_shard_confirmation_threshold())]
		pub fn create_account(
			origin: OriginFor<T>,
			shard_id: ShardId,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			let treasury = T::PalletId::get().into_account();
			T::Currency::transfer(&who, &treasury, T::ReservationRequirement::get(), KeepAlive)?;
			let encoded_call = Self::build_account_creation_call(&who, shard_id);
			pallet_s_contract::Pallet::<T>::push_call(origin, shard_id, encoded_call)?;

			Self::deposit_event(Event::<T>::EnclaveAccountCreated(who, shard_id));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn build_account_creation_call(
			account: &T::AccountId,
			shard_id: ShardId,
		) -> Vec<u8>{
			let call = skw_blockchain_primitives::InputParams {
				origin: Some(b"system".to_vec()),
				origin_public_key: None,
				encrypted_egress: false,
				
				transaction_action: b"create_account".to_vec(),
				receiver: account.to_string().as_bytes().to_vec(),
				
				// an arb amount of offchain runtime gas token for transacrtions
				amount: Some(T::DefaultFaucet::get()),

				wasm_blob_path: None,
				method: None,
				args: None, 
				to: None,
			};

			let mut batched = Vec::new();
			batched.push(call);
			let batched_calls = skw_blockchain_primitives::Input {
				ops: batched,
				shard_id: shard_id,
				block_number: None,
			};

			// TODO: this unwrap should be safe?
			skw_blockchain_primitives::BorshSerialize::try_to_vec(&batched_calls).unwrap()
		}
	}
}
