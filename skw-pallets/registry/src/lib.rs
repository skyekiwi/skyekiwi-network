#![cfg_attr(not(feature = "std"), no_std)]
use sp_std::prelude::*;
pub use pallet::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use frame_support::sp_runtime::traits::Saturating;
	use super::*;
	
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		type RegistrationDuration: Get<u32>;
		// type ForceOrigin: EnsureOrigin<Self::Origin>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn secret_keepers)]
	pub(super) type SecretKeepers<T: Config> = StorageValue<_, 
		Vec< T::AccountId >, OptionQuery >;
	
	#[pallet::storage]
	#[pallet::getter(fn expiration_of)]
	pub(super) type Expiration<T: Config> = StorageMap<_, Twox64Concat, 
		T::AccountId, T::BlockNumber>;

	#[pallet::storage]
	#[pallet::getter(fn public_key_of)]
	pub(super) type PublicKey<T: Config> = StorageMap<_, Twox64Concat, 
		T::AccountId, Vec<u8>>;


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SecretKeeperRegistered(T::AccountId),
		SecretKeeperRenewed(T::AccountId),
		SecretKeeperRemoved(T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidSecretKeeper,
		DuplicateRegistration,
		RegistrationNotFound,
		Unexpected,
	}

	#[pallet::call]
	impl<T:Config> Pallet<T> {

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 3))]
		pub fn register_secret_keeper(
			origin: OriginFor<T>,
			public_key: Vec<u8>,
			signature: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!<Expiration<T>>::contains_key(&who), Error::<T>::DuplicateRegistration);
			// TODO: do we really need this?
			ensure!(!<PublicKey<T>>::contains_key(&who), Error::<T>::DuplicateRegistration);
			ensure!(Self::validate_signature(signature), Error::<T>::InvalidSecretKeeper);

			// TODO: check for key validity
			let pk = Self::compress_hex_key(&public_key);

			if !Self::try_insert_secret_keeper(	who.clone() ) {
				return Err(Error::<T>::Unexpected.into());
			}

			let now = frame_system::Pallet::<T>::block_number();
			let expiration = now.saturating_add( T::RegistrationDuration::get().into() );

			<Expiration<T>>::insert(&who, expiration);
			<PublicKey<T>>::insert(&who, pk);
			
			Self::deposit_event(Event::<T>::SecretKeeperRegistered(who));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 3))]
		pub fn renew_registration(
			origin: OriginFor<T>,
			public_key: Vec<u8>,
			signature: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(<Expiration<T>>::contains_key(&who), Error::<T>::RegistrationNotFound);
			// TODO: do we really need this?
			ensure!(<PublicKey<T>>::contains_key(&who), Error::<T>::RegistrationNotFound);
			ensure!(Self::validate_signature(signature), Error::<T>::InvalidSecretKeeper);

			// TODO: check for key validity
			let pk = Self::compress_hex_key(&public_key);

			let now = frame_system::Pallet::<T>::block_number();
			let expiration = now.saturating_add( T::RegistrationDuration::get().into() );

			<Expiration<T>>::mutate(&who, |expir| *expir = Some(expiration));
			<PublicKey<T>>::mutate(&who, |p| *p = Some(pk));
			
			Self::deposit_event(Event::<T>::SecretKeeperRenewed(who));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 3))]
		pub fn remove_registration(
			origin: OriginFor<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(<Expiration<T>>::contains_key(&who), Error::<T>::RegistrationNotFound);
			// TODO: do we really need this?
			ensure!(<PublicKey<T>>::contains_key(&who), Error::<T>::RegistrationNotFound);

			match Self::try_remove_registration(who.clone()) {
				true => {
					<Expiration<T>>::remove(&who);
					<PublicKey<T>>::remove(&who);

					Self::deposit_event(Event::<T>::SecretKeeperRenewed(who));
					Ok(())
				},
				false => Err(Error::<T>::RegistrationNotFound.into())
			}
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn validate_signature(_call: Vec<u8>) -> bool {
			// TODO: validate da sig
			true
		}

		pub fn try_insert_secret_keeper(
			account_id: T::AccountId,
		) -> bool {

			let mut secret_keepers = match Self::secret_keepers() {
				Some(sk) => sk,
				None => Vec::new()
			};

			secret_keepers.push( account_id );
			<SecretKeepers<T>>::set( Some(secret_keepers) );

			true
		}

		pub fn try_remove_registration(
			account_id: T::AccountId,
		) -> bool {
			let mut secret_keepers = Self::secret_keepers().unwrap();
			match secret_keepers.iter().position(|id| *id == account_id) {
				Some(index) => {
					secret_keepers.swap_remove(index);
					<SecretKeepers<T>>::set( Some(secret_keepers) );
					true
				},
				None => false
			}
		}

		// TODO: move this to a shared crate
		pub fn compress_hex_key(s: &Vec<u8>) -> Vec<u8> {
			(0..s.len())
				.step_by(2)
				.map(|i| s[i] * 16 + s[i + 1])
				.collect()
		}
	}
}

