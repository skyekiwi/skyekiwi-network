#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use super::WeightInfo;
	use skw_blockchain_primitives::{ShardId, compress_hex_key};
	use frame_support::sp_runtime::SaturatedConversion;
	use sp_std::vec::Vec;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type WeightInfo: WeightInfo;
		
		/// duration of the validity of registrations
		#[pallet::constant]
		type RegistrationDuration: Get<Self::BlockNumber>;

		/// maximum number of shards allowed
		#[pallet::constant]
		type MaxActiveShards: Get<u32>;

		/// maximum number of shards allowed
		#[pallet::constant]
		type MaxSecretKeepers: Get<u32>;
		
		// type ForceOrigin: EnsureOrigin<Self::Origin>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// a list of all active secret keepers
	#[pallet::storage]
	#[pallet::getter(fn secret_keepers)]
	pub(super) type SecretKeepers<T: Config> = StorageValue<_, BoundedVec< T::AccountId, T::MaxSecretKeepers >, OptionQuery >;
	
	/// registration expiration block number for each secret keepers
	#[pallet::storage]
	#[pallet::getter(fn expiration_of)]
	pub(super) type Expiration<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, T::BlockNumber>;

	/// identity  key of each secret keepers, used to receive the secret
	#[pallet::storage]
	#[pallet::getter(fn public_key_of)]
	pub(super) type PublicKey<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, [u8; 32]>;

	#[pallet::storage]
	#[pallet::getter(fn user_public_key_of)]
	pub(super) type UserPublicKey<T: Config> = StorageMap<_, Twox64Concat, 
		T::AccountId, [u8; 32]>;

	/// members of each shard
	#[pallet::storage]
	#[pallet::getter(fn shard_members_of)]
	pub(super) type ShardMembers<T: Config> = StorageMap<_, Twox64Concat, ShardId, BoundedVec<T::AccountId, T::MaxSecretKeepers>>;

	// Beacons are identifier of when a secret keeper is supposed to submit a outcome

	/// beacon index of each secret keeper, first come first serve
	#[pallet::storage]
	#[pallet::getter(fn beacon_index_of)]
	pub(super) type BeaconIndex<T: Config> = StorageDoubleMap<_, Twox64Concat, ShardId, Twox64Concat, T::AccountId, u64>;

	/// total number of members in a shard - numbers of nodes playing the beacon game
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
		InvalidPublicKey,
		SecretKeeperAtFullCapacity,
	}

	#[pallet::call]
	impl<T:Config> Pallet<T> {

		/// register a secret keeper
		#[pallet::weight(<T as Config>::WeightInfo::register_secret_keeper())]
		pub fn register_secret_keeper(
			origin: OriginFor<T>,
			public_key: Vec<u8>,
			signature: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!<Expiration<T>>::contains_key(&who), Error::<T>::DuplicateRegistration);
			ensure!(Self::validate_signature(signature), Error::<T>::InvalidSecretKeeper);

			// TODO: check for key validity
			let pk = compress_hex_key(&public_key);
			// TODO: switch to primiive publicKey type
			let bounded_pk: [u8; 32] = pk.try_into().map_err(|_| Error::<T>::InvalidPublicKey)?;

			if !Self::try_insert_secret_keeper(	who.clone() ) {
				return Err(Error::<T>::Unexpected.into());
			}

			let now = frame_system::Pallet::<T>::block_number();
			let expiration = now + T::RegistrationDuration::get();

			<Expiration<T>>::insert(&who, expiration);
			<PublicKey<T>>::insert(&who, bounded_pk);
			
			Self::deposit_event(Event::<T>::SecretKeeperRegistered(who));
			Ok(())
		}

		/// renew registration by submitting a new signature and public key
		#[pallet::weight(<T as Config>::WeightInfo::renew_registration())]
		pub fn renew_registration(
			origin: OriginFor<T>,
			public_key: Vec<u8>,
			signature: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(<Expiration<T>>::contains_key(&who), Error::<T>::RegistrationNotFound);
			ensure!(Self::validate_signature(signature), Error::<T>::InvalidSecretKeeper);

			// TODO: check for key validity
			let pk = compress_hex_key(&public_key);
			let bounded_pk = pk.try_into().map_err(|_| Error::<T>::InvalidPublicKey)?;


			let now = frame_system::Pallet::<T>::block_number();
			let expiration = now + T::RegistrationDuration::get();

			<Expiration<T>>::mutate(&who, |expir| *expir = Some(expiration));
			<PublicKey<T>>::mutate(&who, |p| *p = Some(bounded_pk));
			
			Self::deposit_event(Event::<T>::SecretKeeperRenewed(who));
			Ok(())
		}

		/// remove ones own registration record
		#[pallet::weight(<T as Config>::WeightInfo::remove_registration())]
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

		/// register all active shards one is running
		#[pallet::weight(<T as Config>::WeightInfo::register_running_shard())]
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
					let mut shard_members = BoundedVec::<T::AccountId, T::MaxSecretKeepers>::default();
					shard_members.try_push(who.clone()).map_err(|_| Error::<T>::SecretKeeperAtFullCapacity)?;
					shard_members
				},
				Some(mut shard_members) => {
					shard_members.try_push(who.clone()).map_err(|_| Error::<T>::SecretKeeperAtFullCapacity)?;
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

		/// register a user's public key
		// #[pallet::weight(<T as pallet::Config>::WeightInfo::register_user_public_key())]
		#[pallet::weight(0)]
		pub fn register_user_public_key(
			origin: OriginFor<T>,
			public_key: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let pk = compress_hex_key(&public_key);
			let bounded_pk: [u8; 32] = pk.try_into().map_err(|_| Error::<T>::InvalidPublicKey)?;

			<UserPublicKey<T>>::insert(&who, bounded_pk);

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
				None => BoundedVec::<T::AccountId, T::MaxSecretKeepers>::default()
			};

			let res = secret_keepers.try_push( account_id );
			if res == Ok(()) {
				<SecretKeepers<T>>::put(secret_keepers);
				true
			} else {
				false
			}
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
			shard <= T::MaxActiveShards::get().into()
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
	}
}

