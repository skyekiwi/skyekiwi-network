#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{
	Blake2_128Concat, dispatch::DispatchResult, pallet_prelude::*
};
use sp_runtime::ArithmeticError;
use sp_runtime::traits::{CheckedAdd, CheckedMul};
use sp_std::prelude::*;
pub use pallet::*;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;


#[frame_support::pallet]
pub mod pallet {
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
	pub(super) type Metadata<T: Config> = StorageMap<_, Blake2_128Concat, u64, Vec<u8>>;

	#[pallet::storage]
	#[pallet::getter(fn owner_of)]
	pub(super) type Owner<T: Config> = StorageMap<_, Blake2_128Concat, u64, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn operators_of)]
	pub(super) type Operator<T: Config> = StorageMap<_, Blake2_128Concat, u64, Vec<T::AccountId>>;

	#[pallet::type_value]
	pub(super) fn DefaultId<T: Config>() -> u64 { 0u64 }
	#[pallet::storage]
	pub(super) type CurrentSecertId<T: Config> = StorageValue<_, u64, ValueQuery, DefaultId<T>>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SecretRegistered(u64),
		SecretUpdated(u64),
		MembershipGranted(u64, T::AccountId),
		MembershipRevoked(u64, T::AccountId),
		SecretBurnt(u64),
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
			let newId = id.saturating_add(1);

			<Metadata<T>>::insert(&id, metadata);
			<Owner<T>>::insert(&id, who);
			<CurrentSecertId<T>>::set(id);
			Self::deposit_event(Event::<T>::SecretRegistered(newId));
			
			Ok(())
		}
	}
}

