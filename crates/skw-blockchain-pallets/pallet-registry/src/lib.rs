#![cfg_attr(not(feature = "std"), no_std)]
use sp_std::prelude::*;
pub use pallet::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

type ShardId = u64;
#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use frame_support::sp_runtime::traits::Saturating;
	use frame_support::sp_runtime::SaturatedConversion;
	use super::*;
	
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		type RegistrationDuration: Get<u32>;

		#[pallet::constant]
		type MaxActiveShards: Get<u64>;
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
	
	#[pallet::storage]
	#[pallet::getter(fn shard_members_of)]
	pub(super) type ShardMembers<T: Config> = StorageMap<_, Twox64Concat, ShardId, Vec<T::AccountId>>;
	
	#[pallet::storage]
	#[pallet::getter(fn beacon_index_of)]
	pub(super) type BeaconIndex<T: Config> = StorageDoubleMap<_, Twox64Concat, ShardId,
	Twox64Concat, T::AccountId, u64>;

	#[pallet::storage]
	#[pallet::getter(fn beacon_count_of)]
	pub(super) type BeaconCount<T: Config> = StorageMap<_, Twox64Concat, ShardId, u64>;
	
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SecretKeeperRegistered(T::AccountId),
		SecretKeeperRenewed(T::AccountId),
		SecretKeeperRemoved(T::AccountId),
		NewMemberForShard(ShardId),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidSecretKeeper,
		DuplicateRegistration,
		RegistrationNotFound,
		Unexpected,
		InvalidShardId,
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

			ensure!(Self::is_valid_secret_keeper(&who), Error::<T>::InvalidSecretKeeper);

			match Self::try_remove_registration(who.clone()) {
				true => {
					Self::deposit_event(Event::<T>::SecretKeeperRenewed(who));
					Ok(())
				},
				false => Err(Error::<T>::RegistrationNotFound.into())
			}
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 3))]
		pub fn register_running_shard(
			origin: OriginFor<T>,
			shard: ShardId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			
			ensure!(Self::is_valid_shard_id(shard), Error::<T>::InvalidShardId);
			ensure!(Self::is_valid_secret_keeper(&who), Error::<T>::InvalidSecretKeeper);
			let shard_members = Self::shard_members_of(shard);
			let shard_members = match shard_members {
				None => {
					// first member of the shard
					let mut shard_members = Vec::new();
					shard_members.push(who.clone());
					shard_members
				},
				Some(mut shard_members) => {
					shard_members.push(who.clone());
					shard_members
				}
			};

			<BeaconIndex<T>>::insert(&shard, &who, shard_members.len() as u64);
			<BeaconCount<T>>::mutate(&shard, 
				|count| *count = Some(shard_members.len() as u64));
			<ShardMembers<T>>::mutate(&shard, 
				|members| *members = Some(shard_members));

			Ok(())
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
					<Expiration<T>>::remove(&account_id);
					<PublicKey<T>>::remove(&account_id);
					true
				},
				None => false
			}
		}

		pub fn is_valid_secret_keeper(who: &T::AccountId) -> bool {
			let is_registered: bool = 
				<Expiration<T>>::contains_key(who) && 
				// do we really need to check PUblicKey?
				<PublicKey<T>>::contains_key(who);

			match is_registered {
				true => {
					Self::expiration_of(who).unwrap() >= frame_system::Pallet::<T>::block_number()
				},
				false => false
			}
		}

		pub fn is_valid_shard_id(shard: ShardId) -> bool {
			shard <= T::MaxActiveShards::get()
		}

		pub fn is_beacon_turn(
			block_number: T::BlockNumber, 
			who: &T::AccountId, 
			shard: ShardId,
			threshold: u64,
		) -> bool {
			if !Self::is_valid_shard_id(shard) || !Self::is_valid_secret_keeper(who) {
				return false;
			}

			if threshold < 1 {
				return false;
			}

			let beacon_index = Self::beacon_index_of(shard, who);
			if beacon_index.is_none() {
				// sanity check
				return false;
			}

			let beacon_count = Self::beacon_count_of(shard);
			if beacon_count.is_none() || beacon_count == Some(0) {
				// sanity check
				return false;
			}

			let beacon_index = beacon_index.unwrap();
			let beacon_count = beacon_count.unwrap();
			
			if threshold >= beacon_count {
				// always allow when threshold is greater than or equal to the number of members
				return true;
			}

			// NO PANIC BELOW THIS LINE
			let block_number = block_number.saturated_into::<u64>();

			// 1 2 3 4 5 6 7 8 9 
			// _ X X X _ _ _ _ _
			(
				block_number % beacon_count <= beacon_index &&
				beacon_index <= block_number % beacon_count + threshold - 1
			) 
			||
			// X X _ _ _ _ _ _ X
			(
				block_number % beacon_count + threshold - 1 > beacon_count && 
				(
					beacon_count - (block_number % beacon_count + threshold - 1) % beacon_count <= beacon_index 
						||
					beacon_index <= block_number % beacon_count + threshold - 1 - beacon_count
				)
			)
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

