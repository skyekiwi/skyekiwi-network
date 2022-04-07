#![cfg_attr(not(feature = "std"), no_std)]
use sp_std::prelude::*;
pub use pallet::*;

pub use pallet_secrets;
use pallet_secrets::SecretId;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::WeightInfo;

pub type CallIndex = u64;
pub type EncodedCall = Vec<u8>;
pub type ShardId = u64;
pub type PublicKey = [u8; 32];
pub type ContractName = Vec<u8>;

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

		type WeightInfo: WeightInfo;

		/// maximum length of encoded calls allowed
		#[pallet::constant]
		type MaxCallLength: Get<u32>;

		/// minimum length of a contract name
		#[pallet::constant]
		type MinContractNameLength: Get<u32>;

		/// maximum length of a contract name
		#[pallet::constant]
		type MaxContractNameLength: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// wasm_blob of a deployed contracts
	#[pallet::storage]
	#[pallet::getter(fn wasm_blob_cid_of)]
	pub(super) type WasmBlobCID<T: Config> = StorageDoubleMap<_, Twox64Concat,
		ShardId, Blake2_128Concat, ContractName, Vec<u8> >;

	/// call history of a block (ShardId, BlockNumber) -> Vec<CallIndex>
	#[pallet::storage]
	#[pallet::getter(fn call_history_of)]
	pub(super) type CallHistory<T: Config> = StorageDoubleMap<_, Twox64Concat,
		ShardId, Twox64Concat, T::BlockNumber, Vec<CallIndex> >;
	
	/// call content of a call (ShardId, CallIndex) -> EncodedCall
	#[pallet::storage]
	#[pallet::getter(fn call_record_of)]
	pub(super) type CallRecord<T: Config> = StorageMap<_, Twox64Concat, CallIndex, (EncodedCall, T::AccountId) >;

	/// the callIndex that will be assigned to the next calls
	#[pallet::type_value]
	pub(super) fn DefaultId<T: Config>() -> CallIndex { 0u64 }
	#[pallet::storage]
	#[pallet::getter(fn current_call_index_of)]
	pub(super) type CurrentCallIndex<T: Config> = StorageValue<_, CallIndex, ValueQuery, DefaultId<T>>;

	/// the secret id of the shard state file on pallet-secrets
	#[pallet::storage]
	#[pallet::getter(fn shard_secret_id)]
	pub(super) type ShardSecretIndex<T: Config> = StorageMap<_, Twox64Concat,
		ShardId, SecretId >;
	
	/// the public key of the shard, used for sending encrypted encoded calls
	#[pallet::storage]
	#[pallet::getter(fn shard_public_key)]
	pub(super) type ShardPublicKey<T: Config> = StorageMap<_, Twox64Concat,
		ShardId, PublicKey>;
	
	/// the highest call index of a shard as of the latest state rollup
	#[pallet::storage]
	#[pallet::getter(fn shard_high_call_index)]
	pub(super) type ShardHighCallIndex<T: Config> = StorageMap<_, Twox64Concat,
		ShardId, CallIndex>;

	/// authorized members who can access the shard
	#[pallet::storage]
	#[pallet::getter(fn shard_operator)]
	pub(super) type ShardOperator<T: Config> = StorageDoubleMap<_, Twox64Concat,
		ShardId, Twox64Concat, T::AccountId, bool>;
		
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CallReceived(ShardId, CallIndex),
		ShardInitialized(ShardId),
		ShardRolluped(ShardId, CallIndex),
		SecretContractRegistered(ShardId, Vec<u8>, CallIndex),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidContractName,
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

		/// register a contract with a deployment encoded call
		#[pallet::weight(<T as pallet::Config>::WeightInfo::register_contract())]
		pub fn register_contract(
			origin: OriginFor<T>, 
			contract_name: Vec<u8>,
			wasm_blob_cid: Vec<u8>,
			deployment_call: EncodedCall,
			shard_id: ShardId,
		) -> DispatchResult {
			let deployer = ensure_signed(origin.clone())?;

			// TODO: validate deployment call
			// Deployment Call Layout: [("action_deploy"), init_call1, init_call2 ...]
			// ("action_deploy") will be automatically appended by the offchain bridge

			ensure!(
				Self::validate_name(shard_id, contract_name.clone()),
				Error::<T>::InvalidContractName
			);
			ensure!(
				Self::validate_call(deployment_call.clone()),
				Error::<T>::InvalidEncodedCall
			);

			let now = frame_system::Pallet::<T>::block_number();
			ensure!(Self::is_shard_running(shard_id), Error::<T>::ShardNotInitialized);

			// No error below this line 
			<WasmBlobCID<T>>::insert(&shard_id, &contract_name, wasm_blob_cid);

			let call_index = Self::current_call_index_of();
			<CallRecord<T>>::insert(&call_index, (deployment_call, deployer));

			// insert the init call
			let mut content = Self::call_history_of(&shard_id, &now).unwrap_or(Vec::new());
			content.push(call_index);
			<CallHistory<T>>::mutate(&shard_id, &now,
				|c| *c = Some(content.clone()));
			<CurrentCallIndex<T>>::set( call_index.saturating_add(1) );

			Self::deposit_event(Event::<T>::SecretContractRegistered(shard_id, contract_name, call_index));
			Ok(())
		}

		/// push a batch of calls for a shard
		#[pallet::weight(<T as pallet::Config>::WeightInfo::push_call())]
		pub fn push_call(
			origin: OriginFor<T>, 
			shard_id: ShardId,
			call: EncodedCall,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::validate_call(call.clone()), Error::<T>::InvalidEncodedCall);

			ensure!(Self::is_shard_running(shard_id), Error::<T>::ShardNotInitialized);
			let call_index = Self::current_call_index_of();

			let now = frame_system::Pallet::<T>::block_number();
			<CallRecord<T>>::insert(&call_index, (call, who));
			
			let history = Self::call_history_of(&shard_id, now);
			let mut content = history.unwrap_or(Vec::new());
			content.push(call_index);
			<CallHistory<T>>::mutate(&shard_id, &now,
				|c| *c = Some(content.clone()));
			<CurrentCallIndex<T>>::set( call_index.saturating_add(1) );
			Self::deposit_event(Event::<T>::CallReceived(shard_id, call_index));
			Ok(())
		}

		/// (SHARD OPERATOR ONLY) initialize a shard with initial state file and initial calls
		#[pallet::weight(<T as pallet::Config>::WeightInfo::initialize_shard())]
		pub fn initialize_shard(
			origin: OriginFor<T>, 
			shard_id: ShardId,
			call: EncodedCall,
			initial_state_metadata: Vec<u8>,
			public_key: PublicKey,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			ensure!(Self::shard_operator(&shard_id, &who).is_some(), Error::<T>::Unauthorized);
			ensure!(Self::validate_call(call.clone()), Error::<T>::InvalidEncodedCall);
			ensure!(!Self::is_shard_running(shard_id), Error::<T>::ShardHasBeenInitialized); 

			match pallet_secrets::Pallet::<T>::register_secret(origin, initial_state_metadata) {
				Ok(()) => {
					// get the lastest secretId - 1 -> it belongs to the secret we have just created
					let secret_id = pallet_secrets::Pallet::<T>::current_secret_id().saturating_sub(1);
					let now = frame_system::Pallet::<T>::block_number();

					let call_index = Self::current_call_index_of();
					let mut call_history: Vec<CallIndex> = Vec::new();
					call_history.push(call_index);

					// callIndex 0 is the init call - so current call is 1
					<CurrentCallIndex<T>>::set( call_index.saturating_add(1) );
					<CallRecord<T>>::insert(&call_index, (call, who));
					<CallHistory<T>>::insert(&shard_id, &now, call_history);
					<ShardSecretIndex<T>>::insert(&shard_id, secret_id);
					<ShardPublicKey<T>>::insert(&shard_id, public_key);
					<ShardHighCallIndex<T>>::insert(&shard_id, call_index);
					Self::deposit_event(Event::<T>::ShardInitialized(shard_id));
					Ok(())
				},
				Err(err) => Err(err)
			}
		}

		/// (SHARD OPERATOR ONLY) rollup a shard for key rotations
		#[pallet::weight(<T as pallet::Config>::WeightInfo::shard_rollup())]
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
		
		/// (ROOT ONLY) nominate a shard operator
		#[pallet::weight(<T as pallet::Config>::WeightInfo::add_authorized_shard_operator())]
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

		pub fn validate_call(call: EncodedCall) -> bool {
			call.len() <= T::MaxCallLength::get() as usize 
		}

		pub fn validate_name(shard_id: ShardId, name: Vec<u8>,) -> bool {
			name.len() <= T::MaxContractNameLength::get() as usize 
				&&
			name.len() >= T::MinContractNameLength::get() as usize
				&& 
			Self::wasm_blob_cid_of(shard_id, name).is_none()
		}

		pub fn is_shard_running(shard_id: ShardId) -> bool {
			// or we can use any of the shard initialization param
			Self::shard_secret_id(shard_id).is_some()
		}		
	}
}

