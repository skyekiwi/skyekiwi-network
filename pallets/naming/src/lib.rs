#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{
	Blake2_128Concat, dispatch::DispatchResult, pallet_prelude::*,
	traits::{
		Currency, ReservableCurrency, EnsureOrigin
	}
};
use sp_runtime::ArithmeticError;
use sp_runtime::traits::{CheckedAdd, CheckedMul};
use sp_std::prelude::*;
pub use pallet::*;

// #[cfg(test)]
// mod tests;

// #[cfg(test)]
// mod mock;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;


#[frame_support::pallet]
pub mod pallet {
	use frame_system::pallet_prelude::*;
	use super::*;
	
	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;

		#[pallet::constant]
		type ReservationFee: Get<BalanceOf <Self>>;

		#[pallet::constant]
		type BlockPerPeriod: Get<<Self as frame_system::Config>::BlockNumber>;
		
		#[pallet::constant]
		type MaxPeriod: Get<u32>;

		type ForceOrigin: EnsureOrigin<Self::Origin>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// resolve_to, expiration, locked amount, period
	#[pallet::storage]
	#[pallet::getter(fn name_of)]
	pub(super) type Naming<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, 
	(T::AccountId, T::BlockNumber, BalanceOf<T>)>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NameRegistered(T::AccountId, T::BlockNumber),
		NameRenewed(T::AccountId, T::BlockNumber),
		NameCleared(T::AccountId),
		NameForceCleared(T::Hash),
	}

	#[pallet::error]
	pub enum Error<T> {
		Unnamed,
		NameTaken,
		PeriodTooLong
	}

	#[pallet::call]
	impl<T:Config> Pallet<T> {
		#[pallet::weight(20_000 + T::DbWeight::get().reads_writes(1, 1))]
		
		/// send in hash of the name and register it to the origin
		pub fn set_or_renew_name(
			origin: OriginFor<T>, 
			name: T::Hash,
			period: u32 // in days
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(period <= T::MaxPeriod::get(), Error::<T>::PeriodTooLong);

			let now = frame_system::Pallet::<T>::block_number();
			let deposit = <BalanceOf<T>>::from(period)
				.checked_mul(&T::ReservationFee::get())
				.ok_or(ArithmeticError::Overflow)?;

			let duration = T::BlockPerPeriod::get()
				.checked_mul(&T::BlockNumber::from(period))
				.ok_or(ArithmeticError::Overflow)?;

			if let Some((_to, _expiration, _deposit)) = <Naming<T>>::get(&name) {
				// EITHER the previous naming expired *OR* a renewal operation
				ensure!(_expiration < now || _to == who, Error::<T>::NameTaken);
				
				let mut renew: bool = false;

				// now the user is authorized to take or renew the name
				let (expiration, new_deposit) = if _expiration < now {
					// register to new user

					// release the old reserve first
					// it should be "_deposit" from the mapping instead of "deposit"
					T::Currency::unreserve(&_to, _deposit.clone());
					(
						now.checked_add(&duration).ok_or(ArithmeticError::Overflow)?,
						deposit,
					)
				} else {
					// renewal
					renew = true;
					
					(
						_expiration.checked_add(&duration).ok_or(ArithmeticError::Overflow)?,
						_deposit.checked_add(&deposit).ok_or(ArithmeticError::Overflow)?,
					)
				};

				T::Currency::reserve(&who, deposit.clone())?;
				<Naming<T>>::insert(&name, (
					who.clone(), 
					expiration, 
					new_deposit,
				));

				if renew {
					Self::deposit_event(Event::<T>::NameRenewed(who, expiration));
				} else {
					Self::deposit_event(Event::<T>::NameRegistered(who, expiration));
				}

			} else {
				// empty name 
				T::Currency::reserve(&who, deposit.clone())?;
				let expiration = now.checked_add(&duration).ok_or(ArithmeticError::Overflow)?;
				<Naming<T>>::insert(&name, (
					who.clone(), 
					expiration, 
					deposit,
				));
				Self::deposit_event(Event::<T>::NameRegistered(who, expiration));
			}
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn clear_name(
			origin: OriginFor<T>, 
			name: T::Hash
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			if let Some((_to, _expiration, _deposit)) = <Naming<T>>::take(&name) {
				T::Currency::unreserve(&_to, _deposit);
				Self::deposit_event(Event::<T>::NameCleared(who));
			} else {
				Err(Error::<T>::Unnamed)?
			}
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn force_clear_name(
			origin: OriginFor<T>, 
			name: T::Hash
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;

			if let Some((_to, _expiration, _deposit)) = <Naming<T>>::take(&name) {
				T::Currency::unreserve(&_to, _deposit);
				Self::deposit_event(Event::<T>::NameForceCleared(name));
			} else {
				Err(Error::<T>::Unnamed)?
			}
			Ok(())
		}
	}
}

