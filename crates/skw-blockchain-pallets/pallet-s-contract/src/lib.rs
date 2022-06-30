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
	use frame_support::{
		pallet_prelude::*, ensure, PalletId,
		sp_runtime::traits::AccountIdConversion, StorageHasher
	};
	use frame_system::pallet_prelude::*;
	use super::WeightInfo;
	use skw_blockchain_primitives::types::{CallIndex, EncodedCall, ShardId, PublicKey, SecretId, Bytes};
	use frame_support::sp_runtime::SaturatedConversion;
	use sp_std::vec::Vec;

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

		/// maximum length of encoded calls allowed
		#[pallet::constant]
		type MaxCallPerBlock: Get<u32>;

		/// minimum length of a contract name
		#[pallet::constant]
		type MinContractNameLength: Get<u32>;

		/// maximum length of a contract name
		#[pallet::constant]
		type MaxContractNameLength: Get<u32>;

		#[pallet::constant]
		type SContractRoot: Get<PalletId>;

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// wasm_blob of a deployed contracts
	#[pallet::storage]
	#[pallet::getter(fn wasm_blob_cid_of)]
	pub(super) type WasmBlobCID<T: Config> = StorageDoubleMap<_, Twox64Concat,
		ShardId, Blake2_128Concat, BoundedVec<u8, T::MaxContractNameLength>, BoundedVec<u8, T::IPFSCIDLength> >;

	/// call history of a block (ShardId, BlockNumber) -> Vec<CallIndex>
	#[pallet::storage]
	#[pallet::getter(fn call_history_of)]
	pub(super) type CallHistory<T: Config> = StorageDoubleMap<_, Twox64Concat,
		ShardId, Twox64Concat, T::BlockNumber, BoundedVec<CallIndex, T::MaxCallPerBlock> >;
	
	/// call content of a call (ShardId, CallIndex) -> EncodedCall
	#[pallet::storage]
	#[pallet::getter(fn call_record_of)]
	pub(super) type CallRecord<T: Config> = StorageMap<_, Twox64Concat, CallIndex, (BoundedVec<u8, T::MaxCallLength>, T::AccountId) >;

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
		InvalidCallIndex,
		InvalidCallOutput,
		InvalidShardIndex,
		ShardNotInitialized,
		ShardHasBeenInitialized,
		TooManyCallsInCurrentBlock,
		InvalidWasmBlobCID,	
		Unauthorized, 
		Unexpected,
	}

	#[pallet::call]
	impl<T:Config> Pallet<T> {

		/// register a contract with a deployment encoded call
		#[pallet::weight(<T as Config>::WeightInfo::register_contract())]
		pub fn register_contract(
			origin: OriginFor<T>, 
			contract_name: Vec<u8>,
			wasm_blob_cid: Vec<u8>,
			deployment_call: EncodedCall,
			shard_id: ShardId,
		) -> DispatchResult {
			let deployer = ensure_signed(origin.clone())?;

			// Deployment Call Layout: [("action_deploy"), init_call1, init_call2 ...]
			// ("action_deploy") will be automatically appended by the offchain bridge

			let bounded_wasm_blob_cid = BoundedVec::<u8, T::IPFSCIDLength>::try_from(wasm_blob_cid)
			.map_err(|_| Error::<T>::InvalidWasmBlobCID)?;
	
			let bounded_contract_name = BoundedVec::<u8, T::MaxContractNameLength>::try_from(contract_name.clone())
				.map_err(|_| Error::<T>::InvalidContractName)?;

			ensure!(
				Self::validate_name(shard_id, &bounded_contract_name),
				Error::<T>::InvalidContractName
			);

			let call_index = Self::maybe_push_calls(
				deployer, 
				shard_id, 
				&deployment_call, 
				Some(contract_name.clone()),
				false,
			)?;

			// No error below this line 
			<WasmBlobCID<T>>::insert(&shard_id, &bounded_contract_name, bounded_wasm_blob_cid);
			Self::deposit_event(Event::<T>::SecretContractRegistered(shard_id, contract_name, call_index));
			Ok(())
		}

		/// push a batch of calls for a shard
		#[pallet::weight(<T as Config>::WeightInfo::push_call())]
		pub fn push_call(
			origin: OriginFor<T>, 
			shard_id: ShardId,
			call: EncodedCall,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			if  who.clone() == Self::get_pallet_account_id() {
				let call_index = Self::maybe_push_calls(who, shard_id, &call, None, true)?;
				Self::deposit_event(Event::<T>::CallReceived(shard_id, call_index));
			} else {
				let call_index = Self::maybe_push_calls(who, shard_id, &call, None, false)?;
				Self::deposit_event(Event::<T>::CallReceived(shard_id, call_index));
			}
			Ok(())
		}

		/// (SHARD OPERATOR ONLY) initialize a shard with initial state file and initial calls
		#[pallet::weight(<T as Config>::WeightInfo::initialize_shard())]
		pub fn initialize_shard(
			origin: OriginFor<T>, 
			shard_id: ShardId,
			initial_state_metadata: Vec<u8>,
			public_key: PublicKey,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			ensure!(Self::shard_operator(&shard_id, &who).is_some(), Error::<T>::Unauthorized);

			match pallet_secrets::Pallet::<T>::register_secret(origin, initial_state_metadata) {
				Ok(()) => {
					// get the lastest secretId - 1 -> it belongs to the secret we have just created
					let secret_id = pallet_secrets::Pallet::<T>::current_secret_id().saturating_sub(1);

					<ShardSecretIndex<T>>::insert(&shard_id, secret_id);
					<ShardPublicKey<T>>::insert(&shard_id, public_key);
					<ShardHighCallIndex<T>>::insert(&shard_id, 0);
					Self::deposit_event(Event::<T>::ShardInitialized(shard_id));
					Ok(())
				},
				Err(err) => Err(err)
			}
		}

		/// (SHARD OPERATOR ONLY) rollup a shard for key rotations
		#[pallet::weight(<T as Config>::WeightInfo::shard_rollup())]
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
		#[pallet::weight(<T as Config>::WeightInfo::add_authorized_shard_operator())]
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

		/// (ROOT ONLY/TEST ONLY) WILL BE REMOVED Force Update call_record
		#[pallet::weight(0)]
		pub fn force_update_call_record(
			origin: OriginFor<T>,
			call_index: CallIndex,
			call: EncodedCall,
		) -> DispatchResult {
			ensure_root(origin.clone())?;

			let bounded_encoded_call = 
				BoundedVec::<u8, T::MaxCallLength>::try_from(call)
				.map_err(|_| Error::<T>::InvalidEncodedCall)?;	

			// Note: unsafe unwrap here. Root origin so whatever
			let o = Self::call_record_of(&call_index);
			ensure!(o.is_some(), Error::<T>::InvalidCallIndex);
 
			<CallRecord::<T>>::mutate(&call_index, |r| {
				* r = Some((bounded_encoded_call, o.unwrap().1)); 
			});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn validate_name(shard_id: ShardId, name: &BoundedVec::<u8, T::MaxContractNameLength>) -> bool {
			name.len() >= T::MinContractNameLength::get() as usize
				&& 
			Self::wasm_blob_cid_of(shard_id, name).is_none() // as name is not taken
		}

		pub fn is_shard_running(shard_id: ShardId) -> bool {
			// or we can use any of the shard initialization param
			Self::shard_secret_id(shard_id).is_some()
		}

		pub fn add_contract_creation_call (
			calls: &EncodedCall,
			contract_name: Bytes,
		) -> Result<EncodedCall, Error<T>> {
			
			// ASSUME: the calls has been correctly validated
			let res: Result<skw_blockchain_primitives::types::Calls, _> = 
				skw_blockchain_primitives::BorshDeserialize::try_from_slice(&calls[..]);

 			match res {
				Ok(mut calls) => {
					calls.ops.splice(0..0, [skw_blockchain_primitives::types::Call {
						origin_public_key: T::AccountId::encode(&T::SContractRoot::get().into_account()).try_into().unwrap(),
						receipt_public_key: Blake2_256::hash(&contract_name[..]),
						encrypted_egress: false,

						transaction_action: 4,
						amount: Some(10),
						contract_name: Some(contract_name),
						method: None,
						args: None,
					}]);
					
					Ok(skw_blockchain_primitives::BorshSerialize::try_to_vec(&calls).unwrap())
				},

				Err(_) => Err(Error::<T>::InvalidEncodedCall),	
			}
		}
	
		pub fn validate_and_mark_usual_call(
			origin: &T::AccountId,
			calls: &EncodedCall, 
			block_number: T::BlockNumber, 
			shard_id: ShardId,
			no_origin_check: bool,
		) -> Result<EncodedCall, Error::<T> > {

			let calls: Result<skw_blockchain_primitives::types::Calls, _> = 
			skw_blockchain_primitives::BorshDeserialize::try_from_slice(&calls[..]);

			match calls {

				Ok(mut calls) => {
					let res = calls.ops.iter().all(|op| {
						// transaction_action in valid range
						let valid_action = op.transaction_action <= 4; // (checked with u8) && op.transaction_action >= 0

						// T::AccountId::encode(&T::SContractRoot::get().into_account()) == op.origin_public_key

						// only allow calls and view_calls
						let valid_origin = match op.transaction_action {
							0 | 1 | 4 => false,
							2 | 3 => T::AccountId::encode(&origin) == op.origin_public_key,
							_ => false,
						};

						valid_action && (valid_origin || no_origin_check)
					});

					if res == false {
						return Err(Error::<T>::Unauthorized);
					}

					calls.block_number = Some(block_number.saturated_into::<u32>());
					calls.shard_id = shard_id;

					return Ok(
						skw_blockchain_primitives::BorshSerialize::try_to_vec(&calls).unwrap()
					);
				},
				Err(_) => Err(Error::<T>::InvalidEncodedCall),
			}
		}
		
		pub fn maybe_push_calls(
			who: T::AccountId,
			shard_id: ShardId, 
			call: &EncodedCall,
			contract_name: Option<Bytes>,
			force_push: bool,
		) -> Result<CallIndex, Error::<T> > {
			ensure!(
				force_push || Self::is_shard_running(shard_id), 
				Error::<T>::ShardNotInitialized
			);

			let marked_call = Self::validate_and_mark_usual_call(
				&who,
				&call,
				frame_system::Pallet::<T>::block_number(),
				shard_id,
				force_push,
			);

			if marked_call.is_err() {
				return Err(marked_call.err().unwrap());
			}

			let mut marked_call = marked_call.unwrap();

			if contract_name.is_some() {
				marked_call = Self::add_contract_creation_call(
					&marked_call,
					contract_name.unwrap(),
				)?;
			}

			let call_index = Self::current_call_index_of();
			let bounded_encoded_call = 
				BoundedVec::<u8, T::MaxCallLength>::try_from(marked_call)
				.map_err(|_| Error::<T>::InvalidEncodedCall)?;	

			let now = frame_system::Pallet::<T>::block_number();
			<CallRecord<T>>::insert(&call_index, (bounded_encoded_call, who));
			
			let history = Self::call_history_of(&shard_id, now);
			let mut content = history.unwrap_or_default();
			content.try_push(call_index).map_err(|_| Error::<T>::TooManyCallsInCurrentBlock.into())?;
			<CallHistory<T>>::mutate(&shard_id, &now,
				|c| *c = Some(content.clone()));
			<CurrentCallIndex<T>>::set( call_index.saturating_add(1) );

			Ok(call_index)
		}

		pub fn get_pallet_account_id() -> T::AccountId {
			T::SContractRoot::get().into_account()
		}
	}
}

