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
			Currency, ExistenceRequirement::KeepAlive,
		}, Twox64Concat
	};
	use frame_support::sp_runtime::traits::AccountIdConversion;
	use frame_system::{pallet_prelude::*, RawOrigin};
	use super::*;
	use sp_std::vec::Vec;
	
	use skw_blockchain_primitives::types::{ShardId, Balance};	
	pub type BalanceOf<T> = pallet_treasury::BalanceOf<T>;

	#[pallet::config]
	pub trait Config: frame_system::Config 
			+ pallet_s_contract::Config
			+ pallet_treasury::Config  
	{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

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
	pub(super) type ReservedAmount <T: Config> = StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, ShardId, Balance>;

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
		AlreadyCreated,
		Unexpected,
	}

	#[pallet::call]
	impl<T:Config> Pallet<T> {

		/// (ROOT ONLY) force create an account inside the enclave
		#[pallet::weight(<T as pallet::Config>::WeightInfo::force_create_enclave_account())]
		pub fn force_create_enclave_account(
			origin: OriginFor<T>,
			shard_id: ShardId,
			account: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin.clone())?;
			
			let encoded_call = Self::build_account_creation_call(&account);

			let system_origin: T::AccountId = T::SContractRoot::get().into_account_truncating();
			pallet_s_contract::Pallet::<T>::push_call(RawOrigin::Signed(system_origin).into(), shard_id, encoded_call)?;

			Self::deposit_event(Event::<T>::EnclaveAccountCreated(account, shard_id));
			Ok(())
		}

		/// reserve some token and create an account in the enclave 
		#[pallet::weight(<T as pallet::Config>::WeightInfo::create_account())]
		pub fn create_account(
			origin: OriginFor<T>,
			shard_id: ShardId,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			ensure!(Self::reserved_amount_of(who.clone(), shard_id).is_none(), Error::<T>::AlreadyCreated);

			let treasury = T::PalletId::get().into_account_truncating();
			T::Currency::transfer(&who, &treasury, T::ReservationRequirement::get(), KeepAlive)?;
			
			// token transfered to treasury is a flat fee paid to the system - so reserved_amount is 0
			<ReservedAmount<T>>::insert(&who, &shard_id, 0);

			let encoded_call = Self::build_account_creation_call(&who);

			let system_origin: T::AccountId = T::SContractRoot::get().into_account_truncating();
			pallet_s_contract::Pallet::<T>::push_call(
				RawOrigin::Signed(system_origin).into(),
				shard_id, 
				encoded_call
			)?;


			Self::deposit_event(Event::<T>::EnclaveAccountCreated(who, shard_id));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn build_account_creation_call(
			account: &T::AccountId,
		) -> Vec<u8> {
			let system_origin = T::SContractRoot::get().into_account_truncating();
			let call = skw_blockchain_primitives::types::Call {
				origin_public_key: system_origin,
				// this should be safe as origin is checked with ensure_signed
				receipt_public_key: T::AccountId::encode(&account).try_into().unwrap(),
				encrypted_egress: false,

				transaction_action: 0,
				
				// an arb amount of offchain runtime gas token for transacrtions
				amount: Some(T::DefaultFaucet::get()),
				contract_name: None,
				method: None,
				args: None,
				wasm_code: None,
			};

			let mut batched = Vec::new();
			batched.push(call);
			let batched_calls = skw_blockchain_primitives::types::Calls {
				ops: batched,
				shard_id: 0, // placeholder - will be updated in sContract
				block_number: None,
			};

			// TODO: this unwrap should be safe?
			skw_blockchain_primitives::BorshSerialize::try_to_vec(&batched_calls).unwrap()
		}
	}
}
