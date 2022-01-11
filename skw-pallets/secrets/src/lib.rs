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
		// type ForceOrigin: EnsureOrigin<Self::Origin>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn metadata_of)]
	pub(super) type Metadata<T: Config> = StorageMap<_, Blake2_128Concat, SecretId, Vec<u8>>;

	// we gotta do more checks with this public key
	// 	-> can they duplicate? should not trust clients for all this info
	#[pallet::storage]
	#[pallet::getter(fn public_key_of_contract)]
	pub(super) type ContractPublicKey<T: Config> = StorageMap<_, Blake2_128Concat, SecretId, Vec<u8>>;

	#[pallet::storage]
	#[pallet::getter(fn high_remote_call_index_of)]
	pub(super) type HighRemoteCallIndex<T: Config> = StorageMap<_, Twox64Concat, SecretId, CallIndex>;

	#[pallet::storage]
	#[pallet::getter(fn owner_of)]
	pub(super) type Owner<T: Config> = StorageMap<_, Blake2_128Concat, SecretId, T::AccountId>;

	#[pallet::storage]
	pub(super) type Operator<T: Config> = StorageDoubleMap<_, Blake2_128Concat, SecretId, Twox64Concat, T::AccountId, bool>;

	#[pallet::type_value]
	pub(super) fn DefaultId<T: Config>() -> SecretId { 0u64 }
	#[pallet::storage]
	#[pallet::getter(fn current_secret_id)]
	pub(super) type CurrentSecertId<T: Config> = StorageValue<_, SecretId, ValueQuery, DefaultId<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SecretRegistered(SecretId),
		SecretContractRegistered(SecretId, Vec<u8>),
		SecretUpdated(SecretId),
		SecretContractRolluped(SecretId, CallIndex),
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
			
			let id = <CurrentSecertId<T>>::get();
			let new_id = id.saturating_add(1);

			<Metadata<T>>::insert(&id, metadata);
			<Owner<T>>::insert(&id, who);
			<CurrentSecertId<T>>::set(new_id);
			Self::deposit_event(Event::<T>::SecretRegistered(id));
			
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 5))]
		pub fn register_secret_contract(
			origin: OriginFor<T>, 
			metadata: Vec<u8>,
			contract_public_key: Vec<u8>
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(metadata.len() == T::IPFSCIDLength::get() as usize, Error::<T>::MetadataNotValid);
			
			// compress the pk: [0x3, 0x2, 0x1, 0xf] => [0x32, 0x1f]
			// TODO: when the public key is invalid on compression - exception needs to be handled
			let pk: Vec<u8> = Self::compress_hex_key(&contract_public_key);
			ensure!(pk.len() == 32 as usize, Error::<T>::ContractPublicKeyNotValid);
			
			let id = <CurrentSecertId<T>>::get();
			let new_id = id.saturating_add(1);

			<Metadata<T>>::insert(&id, metadata);
			<ContractPublicKey<T>>::insert(&id, pk.clone());
			<HighRemoteCallIndex<T>>::insert(&id, 0u64);
			<Owner<T>>::insert(&id, who);
			<CurrentSecertId<T>>::set(new_id);
			Self::deposit_event(Event::<T>::SecretContractRegistered(id, pk.clone()));
			
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
			ensure!(metadata.len() == T::IPFSCIDLength::get() as usize, Error::<T>::MetadataNotValid);
			ensure!(Self::authorize_access(who, secret_id) == true, Error::<T>::AccessDenied);

			// so far, it is garenteed the secret_id is valid 
			<Metadata<T>>::mutate(&secret_id, |meta| *meta = Some(metadata));
			Self::deposit_event(Event::<T>::SecretUpdated(secret_id));
			
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3, 3))]
		pub fn contract_rollup(
			origin: OriginFor<T>,
			secret_id: SecretId,
			metadata: Vec<u8>,
			new_high_remote_call_index: CallIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(metadata.len() == T::IPFSCIDLength::get() as usize, Error::<T>::MetadataNotValid);
			ensure!(Self::authorize_access(who, secret_id) == true, Error::<T>::AccessDenied);
			ensure!(Self::is_executable(secret_id) == true, Error::<T>::SecretNotExecutable);
			ensure!(new_high_remote_call_index > <HighRemoteCallIndex<T>>::get(&secret_id).unwrap(), Error::<T>::ContractCallIndexError);

			// so far, it is garenteed the secret_id is valid 
			<Metadata<T>>::mutate(&secret_id, |meta| *meta = Some(metadata));
			<HighRemoteCallIndex<T>>::mutate(&secret_id, |index| *index = Some(new_high_remote_call_index));

			Self::deposit_event(Event::<T>::SecretContractRolluped(secret_id, new_high_remote_call_index));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn burn_secret(
			origin: OriginFor<T>,
			secret_id: SecretId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::authorize_owner(who, secret_id) == true, Error::<T>::AccessDenied);

			// so far, it is garenteed the secret_id is valid 
			<Metadata<T>>::take(&secret_id);
			<Owner<T>>::take(&secret_id);
			<Operator<T>>::remove_prefix(&secret_id, None);

			if Self::is_executable(secret_id) {
				<ContractPublicKey<T>>::take(&secret_id);
				<HighRemoteCallIndex<T>>::take(&secret_id);
			}
			
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
			<ContractPublicKey<T>>::contains_key(&secret_id) && <HighRemoteCallIndex<T>>::contains_key(&secret_id)
		}

		pub fn compress_hex_key(s: &Vec<u8>) -> Vec<u8> {
			(0..s.len())
				.step_by(2)
				.map(|i| s[i] * 16 + s[i + 1])
				.collect()
		}
	}
}

