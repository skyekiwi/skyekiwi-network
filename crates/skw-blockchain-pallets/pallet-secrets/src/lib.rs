#![cfg_attr(not(feature = "std"), no_std)]
use sp_std::prelude::*;
pub use pallet::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub type SecretId = u64;
pub type CallIndex = u64;
pub type ShardId = u64;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use super::*;
	
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		
		#[pallet::constant]
		type IPFSCIDLength: Get<u32>;

		#[pallet::constant]
		type MaxActiveShards: Get<u64>;
		// type ForceOrigin: EnsureOrigin<Self::Origin>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Secret Metadata of generic secrets & contracts
	#[pallet::storage]
	#[pallet::getter(fn metadata_of)]
	pub(super) type Metadata<T: Config> = StorageMap<_, Twox64Concat, SecretId, Vec<u8>>;

	/// owner of a generic secret - not useful for contracts
	#[pallet::storage]
	#[pallet::getter(fn owner_of)]
	pub(super) type Owner<T: Config> = StorageMap<_, Twox64Concat, SecretId, T::AccountId>;

	/// operator of a generic secret - not useful for contracts
	#[pallet::storage]
	pub(super) type Operator<T: Config> = StorageDoubleMap<_, Twox64Concat, SecretId, Twox64Concat, T::AccountId, bool>;

	/// the secret ID of the next registered secret
	#[pallet::type_value]
	pub(super) fn DefaultId<T: Config>() -> SecretId { 0u64 }
	#[pallet::storage]
	#[pallet::getter(fn current_secret_id)]
	pub(super) type CurrentSecretId<T: Config> = StorageValue<_, SecretId, ValueQuery, DefaultId<T>>;

	/// ================== Secret Contract Specific Fields ==================
	/// Wasm Blob CID of a secret contract
	#[pallet::storage]
	#[pallet::getter(fn wasm_blob_of)]
	pub(super) type WasmBlob<T: Config> = StorageMap<_, Twox64Concat, SecretId, Vec<u8>>;

	/// shard where the contract is located
	#[pallet::storage]
	#[pallet::getter(fn home_shard_of)]
	pub(super) type HomeShard<T: Config> = StorageMap<_, Twox64Concat, SecretId, ShardId>;
	/// ================== Secret Contract Specific Fields ==================

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SecretRegistered(SecretId),
		SecretContractRegistered(SecretId),
		SecretUpdated(SecretId),
		MembershipGranted(SecretId, T::AccountId),
		MembershipRevoked(SecretId, T::AccountId),
		SecretBurnt(SecretId),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidSecretId,
		AccessDenied,
		MetadataNotValid,
		ContractCallIndexError,
		ContractPublicKeyNotValid,
		SecretNotExecutable,
		NotAllowedForSecretContracts,
		InvalidShardId,
	}

	#[pallet::call]
	impl<T:Config> Pallet<T> {

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 3))]
		pub fn register_secret(
			origin: OriginFor<T>, 
			metadata: Vec<u8>
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(metadata.len() == T::IPFSCIDLength::get() as usize, Error::<T>::MetadataNotValid);
			
			let id = <CurrentSecretId<T>>::get();
			let new_id = id.saturating_add(1);

			<Metadata<T>>::insert(&id, metadata);
			<Owner<T>>::insert(&id, who);
			<CurrentSecretId<T>>::set(new_id);
			Self::deposit_event(Event::<T>::SecretRegistered(id));
			
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 4))]
		pub fn register_secret_contract(
			origin: OriginFor<T>, 
			metadata: Vec<u8>,
			wasm_blob_cid: Vec<u8>,
			shard_id: ShardId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(metadata.len() == T::IPFSCIDLength::get() as usize, Error::<T>::MetadataNotValid);
			ensure!(wasm_blob_cid.len() == T::IPFSCIDLength::get() as usize, Error::<T>::MetadataNotValid);
			ensure!(T::MaxActiveShards::get() >= shard_id, Error::<T>::InvalidShardId);

			let id = <CurrentSecretId<T>>::get();
			let new_id = id.saturating_add(1);

			<Metadata<T>>::insert(&id, metadata);
			<WasmBlob<T>>::insert(&id, wasm_blob_cid);
			<Owner<T>>::insert(&id, who);
			<CurrentSecretId<T>>::set(new_id);
			<HomeShard<T>>::insert(&id, shard_id);
			Self::deposit_event(Event::<T>::SecretContractRegistered(id));
			
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn nominate_member(
			origin: OriginFor<T>,
			vault_id: SecretId,
			member: T::AccountId
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::authorize_owner(who, vault_id) == true, Error::<T>::AccessDenied);

			<Operator<T>>::insert(vault_id, &member, true);
			Self::deposit_event(Event::<T>::MembershipGranted(vault_id, member));
			
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn remove_member(
			origin: OriginFor<T>,
			secret_id: SecretId,
			member: T::AccountId
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::authorize_owner(who, secret_id) == true, Error::<T>::AccessDenied);

			<Operator<T>>::take(&secret_id, &member);
			Self::deposit_event(Event::<T>::MembershipRevoked(secret_id, member));
			
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn update_metadata(
			origin: OriginFor<T>,
			secret_id: SecretId,
			metadata: Vec<u8>
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::is_executable(secret_id) == false, Error::<T>::NotAllowedForSecretContracts);
			ensure!(metadata.len() == T::IPFSCIDLength::get() as usize, Error::<T>::MetadataNotValid);
			ensure!(Self::authorize_access(who, secret_id) == true, Error::<T>::AccessDenied);

			// so far, it is garenteed the secret_id is valid 
			<Metadata<T>>::mutate(&secret_id, |meta| *meta = Some(metadata));
			Self::deposit_event(Event::<T>::SecretUpdated(secret_id));
			
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn burn_secret(
			origin: OriginFor<T>,
			secret_id: SecretId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::is_executable(secret_id) == false, Error::<T>::NotAllowedForSecretContracts);
			ensure!(Self::authorize_owner(who, secret_id) == true, Error::<T>::AccessDenied);

			// so far, it is garenteed the secret_id is valid 
			<Metadata<T>>::take(&secret_id);
			<Owner<T>>::take(&secret_id);
			<Operator<T>>::remove_prefix(&secret_id, None);
			
			Self::deposit_event(Event::<T>::SecretBurnt(secret_id));
			
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn authorize_owner(
			who: T::AccountId,
			secret_id: SecretId
		) -> bool {
			<Owner<T>>::get(&secret_id) == Some(who)
		}

		pub fn authorize_access(
			who: T::AccountId,
			secret_id: SecretId
		) -> bool {
			<Operator<T>>::get(&secret_id, &who) == Some(true) || <Owner<T>>::get(&secret_id) == Some(who)
		}

		pub fn is_executable(
			secret_id: SecretId
		) -> bool {
			<WasmBlob<T>>::contains_key(&secret_id) && <HomeShard<T>>::contains_key(&secret_id)
		}

		pub fn compress_hex_key(s: &Vec<u8>) -> Vec<u8> {
			(0..s.len())
				.step_by(2)
				.map(|i| s[i] * 16 + s[i + 1])
				.collect()
		}
	}
}

