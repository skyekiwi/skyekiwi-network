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
pub type PublicKey = [u8; 32];

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, ensure};
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

	#[pallet::storage]
	#[pallet::getter(fn call_history_of)]
	pub(super) type CallHistory<T: Config> = StorageDoubleMap<_, Twox64Concat,
		ShardId, Twox64Concat, T::BlockNumber, Vec<(EncodedCall, T::AccountId)> >;
	
	#[pallet::storage]
	#[pallet::getter(fn shard_initialization_call)]
	pub(super) type ShardInitializationCall<T: Config> = StorageMap<_, Twox64Concat,
		ShardId, EncodedCall >;
		
	#[pallet::storage]
	#[pallet::getter(fn shard_secret_id)]
	pub(super) type ShardSecretIndex<T: Config> = StorageMap<_, Twox64Concat,
		ShardId, SecretId >;
	
	#[pallet::storage]
	#[pallet::getter(fn shard_public_key)]
	pub(super) type ShardPublicKey<T: Config> = StorageMap<_, Twox64Concat,
		ShardId, PublicKey>;
	
	#[pallet::storage]
	#[pallet::getter(fn shard_high_call_index)]
	pub(super) type ShardHighCallIndex<T: Config> = StorageMap<_, Twox64Concat,
		ShardId, CallIndex>;

	#[pallet::storage]
	#[pallet::getter(fn shard_operator)]
	pub(super) type ShardOperator<T: Config> = StorageDoubleMap<_, Twox64Concat,
		ShardId, Twox64Concat, T::AccountId, bool>;
		
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CallReceived(ShardId, T::BlockNumber),
		ShardInitialized(ShardId),
		ShardRolluped(ShardId, CallIndex),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidEncodedCall,
		InvalidContractIndex,
		InvalidCallOutput,
		InvalidShardIndex,
		ShardNotInitialized,
		ShardHasBeenInitialized,
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
			let now = frame_system::Pallet::<T>::block_number();
			let init = Self::shard_initialization_call(&shard_id);
			ensure!(init.is_some(), Error::<T>::ShardNotInitialized);

			match pallet_secrets::Pallet::<T>::register_secret_contract(
				origin, metadata, wasm_blob_cid, shard_id
			) {
				Ok(()) => {
					// insert the init call
					let mut content = Self::call_history_of(&shard_id, &now).unwrap_or(Vec::new());
					content.push((initialization_call, deployer));
					<CallHistory<T>>::mutate(&shard_id, &now,
						|c| *c = Some(content.clone()));
					Self::deposit_event(Event::<T>::CallReceived(shard_id, now));
					
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
			let home_shard = pallet_secrets::Pallet::<T>::home_shard_of(contract_index);
			ensure!(home_shard.is_some(), Error::<T>::InvalidContractIndex);
			let home_shard = home_shard.unwrap();
			let now = frame_system::Pallet::<T>::block_number();

			let init = Self::shard_initialization_call(&home_shard);

			match init {
				Some(_) => {
					let history = Self::call_history_of(&home_shard, now);
					let mut content = history.unwrap_or(Vec::new());
					content.push((call, who));
					<CallHistory<T>>::mutate(&contract_index, &now,
						|c| *c = Some(content.clone()));
					Self::deposit_event(Event::<T>::CallReceived(contract_index, now));
					Ok(())
				},
				None => Err(Error::<T>::ShardNotInitialized.into())
			}
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 3))]
		pub fn initialize_shard(
			origin: OriginFor<T>, 
			shard_id: ShardId,
			call: EncodedCall,
			initial_metadata: Vec<u8>,
			public_key: PublicKey,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			ensure!(Self::shard_operator(&shard_id, &who).is_some(), Error::<T>::Unauthorized);
			ensure!(Self::validate_call(call.clone()), Error::<T>::InvalidEncodedCall);
			let maybe_init = Self::shard_initialization_call(&shard_id);

			ensure!(maybe_init.is_none(), Error::<T>::ShardHasBeenInitialized); 
			match pallet_secrets::Pallet::<T>::register_secret(origin, initial_metadata) {
				Ok(()) => {
					// TODO: is this always right??
					// get the lastest secretId - 1 -> it belongs to the secret we have just created
					let secret_id = pallet_secrets::Pallet::<T>::current_secret_id().saturating_sub(1);

					<ShardSecretIndex<T>>::insert(&shard_id, secret_id);
					<ShardPublicKey<T>>::insert(&shard_id, public_key);
					<ShardInitializationCall<T>>::insert(&shard_id, call);
					<ShardHighCallIndex<T>>::insert(&shard_id, 0);
					Self::deposit_event(Event::<T>::ShardInitialized(shard_id));
					Ok(())
				},
				Err(err) => Err(err)
			}
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 3))]
		pub fn shard_rollup(
			origin: OriginFor<T>,
			shard_id: ShardId,
			metadata: Vec<u8>,
			high_call_index: CallIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			ensure!(Self::shard_operator(&shard_id, &who).is_some(), Error::<T>::Unauthorized);
			let secret_id = Self::shard_secret_id(shard_id);
			ensure!(secret_id.is_some(), Error::<T>::InvalidShardIndex);

			match pallet_secrets::Pallet::<T>::update_metadata(origin, secret_id.unwrap(), metadata) {
				Ok(()) => {
					<ShardHighCallIndex<T>>::mutate(&shard_id, |hci| {
						* hci = Some(high_call_index);
					});
					Self::deposit_event(Event::<T>::ShardRolluped(shard_id, high_call_index));
					Ok(())
				},
				Err(e) => Err(e)
			}
		}
		
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 3))]
		pub fn add_authorized_shard_operator(
			origin: OriginFor<T>,
			shard_id: ShardId,
			operator: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin.clone())?;

			let secret_id = Self::shard_secret_id(shard_id);
			match secret_id {
				Some(id) => {
					pallet_secrets::Pallet::<T>::force_nominate_member(origin, id, operator.clone())?;
					<ShardOperator<T>>::mutate(&shard_id, &operator, |status| {
						* status = Some(true);
					});

					Ok(())
				},
				None => {
					// the shard has not been initialized
					// do nothing on pallet_secrets

					<ShardOperator<T>>::mutate(&shard_id, &operator, |status| {
						* status = Some(true);
					});
		
					Ok(())
				},
			}
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn validate_call(call: EncodedCall,) -> bool {
			call.len() < T::MaxCallLength::get() as usize
		}

		pub fn get_call(
			contract_index: SecretId,
			block_number: T::BlockNumber,
		) -> Option<Vec<(EncodedCall, T::AccountId)>> {
			let home_shard = pallet_secrets::Pallet::<T>::home_shard_of(contract_index);
			if home_shard.is_none() {
				return None;
			}

			let home_shard = home_shard.unwrap();
			Self::call_history_of(&home_shard, &block_number)
		}
	}
}

