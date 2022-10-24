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
	use frame_support::{ pallet_prelude::*, traits::{PreimageRecipient, PreimageProvider}};
	use frame_system::pallet_prelude::*;
	use skw_blockchain_primitives::types::{SecretId};
	use super::{WeightInfo};
	
	use sp_std::vec::Vec;
	use sp_runtime::traits::{Hash};
	
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		
		type WeightInfo: WeightInfo;

		type Preimage: PreimageRecipient<Self::Hash> + PreimageProvider<Self::Hash>;
		// type ForceOrigin: EnsureOrigin<Self::Origin>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Secret Metadata of generic secrets & contracts
	#[pallet::storage]
	#[pallet::getter(fn metadata_of)]
	pub(super) type Metadata<T: Config> = StorageMap<_, Twox64Concat, SecretId, T::Hash>;

	/// owner of a generic secret - not useful for contracts
	#[pallet::storage]
	#[pallet::getter(fn owner_of)]
	pub(super) type Owner<T: Config> = StorageMap<_, Twox64Concat, SecretId, T::AccountId>;

	/// operator of a generic secret - not useful for contracts
	#[pallet::storage]
	pub(super) type Operator<T: Config> = StorageDoubleMap<_, Twox64Concat, SecretId, Twox64Concat, T::AccountId, bool>;

	/// the secret ID of the next registered secret
	#[pallet::type_value]
	pub(super) fn DefaultId<T: Config>() -> SecretId { 0u32 }
	#[pallet::storage]
	#[pallet::getter(fn current_secret_id)]
	pub(super) type CurrentSecretId<T: Config> = StorageValue<_, SecretId, ValueQuery, DefaultId<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SecretRegistered(SecretId),
		SecretUpdated(SecretId),
		MembershipGranted(SecretId, T::AccountId),
		MembershipRevoked(SecretId, T::AccountId),
		SecretBurnt(SecretId),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidSecretId,
		AccessDenied,
		MetadataStorageError,
		MetadataNotValid,
		SecretNotExecutable,
		NotAllowedForSecretContracts,
		InvalidShardId,
	}

	#[pallet::call]
	impl<T:Config> Pallet<T> {

		/// write a metadata to the secret registry and assign a secret_id
		#[pallet::weight(<T as Config>::WeightInfo::register_secret(metadata.len() as u32))]
		pub fn register_secret(
			origin: OriginFor<T>, 
			metadata: Vec<u8>
		) -> DispatchResult {
			let who = ensure_signed(origin)?;			
			let id = <CurrentSecretId<T>>::get();
			
			let hash = Self::maybe_note_bytes(metadata)?;
			<Metadata<T>>::insert(&id, &hash);
			<Owner<T>>::insert(&id, who);
			<CurrentSecretId<T>>::set(id.saturating_add(1));
			Self::deposit_event(Event::<T>::SecretRegistered(id));
			
			Ok(())
		}
		
		/// nominate an operator to a secret
		#[pallet::weight(<T as Config>::WeightInfo::nominate_member())]
		pub fn nominate_member(
			origin: OriginFor<T>,
			secret_id: SecretId,
			member: T::AccountId
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::authorize_owner(who, secret_id) == true, Error::<T>::AccessDenied);

			<Operator<T>>::insert(secret_id, &member, true);
			Self::deposit_event(Event::<T>::MembershipGranted(secret_id, member));
			
			Ok(())
		}

		/// remove an operator to a secret
		#[pallet::weight(<T as Config>::WeightInfo::remove_member())]
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

		/// update the metadata of a secret
		#[pallet::weight(<T as Config>::WeightInfo::update_metadata(metadata.len() as u32))]
		pub fn update_metadata(
			origin: OriginFor<T>,
			secret_id: SecretId,
			metadata: Vec<u8>
		) -> DispatchResult {
			let who = ensure_signed(origin)?;	
			ensure!(Self::authorize_access(who, secret_id) == true, Error::<T>::AccessDenied);


			// so far, it is garenteed the secret_id is valid 
			match <Metadata<T>>::take(&secret_id) {
				Some(h) => Self::maybe_remove_bytes(&h),
				None => {}
			};

			let hash = Self::maybe_note_bytes(metadata.clone())?;
			<Metadata<T>>::insert(&secret_id, &hash);

			Self::deposit_event(Event::<T>::SecretUpdated(secret_id));
			
			Ok(())
		}

		/// destroy a secret and all its records
		#[pallet::weight(<T as Config>::WeightInfo::burn_secret())]
		pub fn burn_secret(
			origin: OriginFor<T>,
			secret_id: SecretId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::authorize_owner(who, secret_id) == true, Error::<T>::AccessDenied);

			

			<Owner<T>>::remove(&secret_id);
			<Operator<T>>::remove_prefix(&secret_id, None);
			
			Self::deposit_event(Event::<T>::SecretBurnt(secret_id));
			
			Ok(())
		}

		/// (ROOT ONLY) forcefuly nominate an operator to a secret
		#[pallet::weight(<T as Config>::WeightInfo::force_nominate_member())]
		pub fn force_nominate_member(
			origin: OriginFor<T>,
			secret_id: SecretId,
			member: T::AccountId
		) -> DispatchResult {
			ensure_root(origin)?;

			// no checks here!
			<Operator<T>>::insert(secret_id, &member, true);
			Self::deposit_event(Event::<T>::MembershipGranted(secret_id, member));

			Ok(())
		}

		/// (ROOT ONLY) forcefuly remove an operator to a secret
		#[pallet::weight(<T as Config>::WeightInfo::force_remove_member())]
		pub fn force_remove_member(
			origin: OriginFor<T>,
			secret_id: SecretId,
			member: T::AccountId
		) -> DispatchResult {
			ensure_root(origin)?;

			<Operator<T>>::take(&secret_id, &member);
			Self::deposit_event(Event::<T>::MembershipRevoked(secret_id, member));
			
			Ok(())
		}

		/// (ROOT ONLY) forcefuly change owner of a secret
		#[pallet::weight(<T as Config>::WeightInfo::force_change_owner())]
		pub fn force_change_owner(
			origin: OriginFor<T>,
			secret_id: SecretId,
			member: T::AccountId
		) -> DispatchResult {
			ensure_root(origin)?;

			<Owner<T>>::mutate(&secret_id, |owner| {
				* owner = Some(member);
			});
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

		pub fn compress_hex_key(s: &Vec<u8>) -> Vec<u8> {
			(0..s.len())
				.step_by(2)
				.map(|i| s[i] * 16 + s[i + 1])
				.collect()
		}

		// Preimage func are dumped here ... for now
		pub fn maybe_note_bytes(bytes: Vec<u8>) -> Result<T::Hash, DispatchError> {

			let bounded_bytes= BoundedVec::<u8, <<T as crate::pallet::Config>::Preimage as PreimageRecipient<T::Hash>>::MaxSize>::try_from(bytes.clone())
				.map_err(|_| Error::<T>::MetadataNotValid)?;
			let hash = T::Hashing::hash(&bounded_bytes);

			T::Preimage::note_preimage(bounded_bytes);
			Ok(hash)
		}

		pub fn maybe_remove_bytes(hash: &T::Hash) -> () {
			T::Preimage::unnote_preimage(hash);
		}

		pub fn try_get_bytes(hash: &T::Hash) -> Option<Vec<u8>> {
			T::Preimage::get_preimage(hash)
		}
	}
}

