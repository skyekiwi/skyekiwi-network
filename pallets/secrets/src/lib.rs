#![cfg_attr(not(feature = "std"), no_std)]
use sp_std::prelude::*;
pub use pallet::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub type VaultId = u64;

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
	pub(super) type Metadata<T: Config> = StorageMap<_, Blake2_128Concat, VaultId, Vec<u8>>;

	#[pallet::storage]
	#[pallet::getter(fn owner_of)]
	pub(super) type Owner<T: Config> = StorageMap<_, Blake2_128Concat, VaultId, T::AccountId>;

	#[pallet::storage]
	pub(super) type Operator<T: Config> = StorageDoubleMap<_, Blake2_128Concat, VaultId, Twox64Concat, T::AccountId, bool>;

	#[pallet::type_value]
	pub(super) fn DefaultId<T: Config>() -> VaultId { 0u64 }
	#[pallet::storage]
	pub(super) type CurrentSecertId<T: Config> = StorageValue<_, VaultId, ValueQuery, DefaultId<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SecretRegistered(VaultId),
		SecretUpdated(VaultId),
		MembershipGranted(VaultId, T::AccountId),
		MembershipRevoked(VaultId, T::AccountId),
		SecretBurnt(VaultId),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidVaultId,
		AccessDenied,
		MetadataNotValid,
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

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn nominate_member(
			origin: OriginFor<T>,
			vault_id: VaultId,
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
			vault_id: VaultId,
			member: T::AccountId
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::authorize_owner(who, vault_id) == true, Error::<T>::AccessDenied);

			<Operator<T>>::take(&vault_id, &member);
			Self::deposit_event(Event::<T>::MembershipRevoked(vault_id, member));
			
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn update_metadata(
			origin: OriginFor<T>,
			vault_id: VaultId,
			metadata: Vec<u8>
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(metadata.len() == T::IPFSCIDLength::get() as usize, Error::<T>::MetadataNotValid);
			ensure!(Self::authorize_access(who, vault_id) == true, Error::<T>::AccessDenied);

			// so far, it is garenteed the vault_id is valid 
			<Metadata<T>>::mutate(&vault_id, |meta| *meta = Some(metadata));
			Self::deposit_event(Event::<T>::SecretUpdated(vault_id));
			
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn burn_secret(
			origin: OriginFor<T>,
			vault_id: VaultId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::authorize_owner(who, vault_id) == true, Error::<T>::AccessDenied);

			// so far, it is garenteed the vault_id is valid 
			<Metadata<T>>::take(&vault_id);
			<Owner<T>>::take(&vault_id);
			<Operator<T>>::remove_prefix(&vault_id, None);
			
			Self::deposit_event(Event::<T>::SecretBurnt(vault_id));
			
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn authorize_owner(
			who: T::AccountId,
			vault_id: VaultId
		) -> bool {
			<Owner<T>>::get(&vault_id) == Some(who)
		}

		pub fn authorize_access(
			who: T::AccountId,
			vault_id: VaultId
		) -> bool {
			<Operator<T>>::get(&vault_id, &who) == Some(true) || <Owner<T>>::get(&vault_id) == Some(who)
		}
	}
}

