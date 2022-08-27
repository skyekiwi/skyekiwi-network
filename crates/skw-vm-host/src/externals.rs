use wasmi::{
	Externals, RuntimeArgs, Trap, TrapKind,
	RuntimeValue,
  };
  
  use crate::{VMLogic};
  use skw_vm_primitives::errors::{VMLogicError, HostError};
  
  #[derive(PartialEq, Eq)]
  pub enum HostFunctions {
	ReadRegister = 0,
	RegisterLen = 1,
	WriteRegister = 2,
	CurrentAccountId = 3,
	SignerAccountId = 4,
	PredecessorAccountId = 5,
	Input = 6,
	BlockNumber = 7,
	BlockTimestamp = 8,
	StorageUsage = 9,
	AccountBalance = 10,
	AttachedDeposit = 11,
	PrepaidGas = 12,
	UsedGas = 13,
	RandomSeed = 14,
	Sha256 = 15,
	Keccak256 = 16,
	Keccak512 = 17,
	Ripemd160 = 18,
	Ecrecover = 19,
	ValueReturn = 20,
	Panic = 21,
	PanicUtf8 = 22,
	LogUtf8 = 23,
	LogUtf16 = 24,
	Abort = 25,
	PromiseCreate = 26,
	PromiseThen = 27,
	PromiseAnd = 28,
	PromiseBatchCreate = 29,
	PromiseBatchThen = 30,
	PromiseBatchActionCreateAccount = 31,
	PromiseBatchActionDeployContract = 32,
	PromiseBatchActionFunctionCall = 33,
	PromiseBatchActionTransfer = 34,
	PromiseBatchActionDeleteAccount = 35,
	PromiseResultsCount = 36,
	PromiseResult = 37,
	PromiseReturn = 38,
	StorageWrite = 39,
	StorageRead = 40,
	StorageRemove = 41,
	StorageHasKey = 42,
	Gas = 43,
	Unknown,
  }
  
  impl From<usize> for HostFunctions {
	fn from(v: usize) -> Self {
	  match v {
		x if x == HostFunctions::ReadRegister as usize => HostFunctions::ReadRegister,
		x if x == HostFunctions::RegisterLen as usize => HostFunctions::RegisterLen,
		x if x == HostFunctions::WriteRegister as usize => HostFunctions::WriteRegister,
		x if x == HostFunctions::CurrentAccountId as usize => HostFunctions::CurrentAccountId,
		x if x == HostFunctions::SignerAccountId as usize => HostFunctions::SignerAccountId,
		x if x == HostFunctions::PredecessorAccountId as usize => HostFunctions::PredecessorAccountId,
		x if x == HostFunctions::Input as usize => HostFunctions::Input,
		x if x == HostFunctions::BlockNumber as usize => HostFunctions::BlockNumber,
		x if x == HostFunctions::BlockTimestamp as usize => HostFunctions::BlockTimestamp,
		x if x == HostFunctions::StorageUsage as usize => HostFunctions::StorageUsage,
		x if x == HostFunctions::AccountBalance as usize => HostFunctions::AccountBalance,
		x if x == HostFunctions::AttachedDeposit as usize => HostFunctions::AttachedDeposit,
		x if x == HostFunctions::PrepaidGas as usize => HostFunctions::PrepaidGas,
		x if x == HostFunctions::UsedGas as usize => HostFunctions::UsedGas,
		x if x == HostFunctions::RandomSeed as usize => HostFunctions::RandomSeed,
		x if x == HostFunctions::Sha256 as usize => HostFunctions::Sha256,
		x if x == HostFunctions::Keccak256 as usize => HostFunctions::Keccak256,
		x if x == HostFunctions::Keccak512 as usize => HostFunctions::Keccak512,
		x if x == HostFunctions::Ripemd160 as usize => HostFunctions::Ripemd160,
		x if x == HostFunctions::Ecrecover as usize => HostFunctions::Ecrecover,
		x if x == HostFunctions::ValueReturn as usize => HostFunctions::ValueReturn,
		x if x == HostFunctions::Panic as usize => HostFunctions::Panic,
		x if x == HostFunctions::PanicUtf8 as usize => HostFunctions::PanicUtf8,
		x if x == HostFunctions::LogUtf8 as usize => HostFunctions::LogUtf8,
		x if x == HostFunctions::LogUtf16 as usize => HostFunctions::LogUtf16,
		x if x == HostFunctions::Abort as usize => HostFunctions::Abort,
		x if x == HostFunctions::PromiseCreate as usize => HostFunctions::PromiseCreate,
		x if x == HostFunctions::PromiseThen as usize => HostFunctions::PromiseThen,
		x if x == HostFunctions::PromiseAnd as usize => HostFunctions::PromiseAnd,
		x if x == HostFunctions::PromiseBatchCreate as usize => HostFunctions::PromiseBatchCreate,
		x if x == HostFunctions::PromiseBatchThen as usize => HostFunctions::PromiseBatchThen,
		x if x == HostFunctions::PromiseBatchActionCreateAccount as usize => HostFunctions::PromiseBatchActionCreateAccount,
		x if x == HostFunctions::PromiseBatchActionDeployContract as usize => HostFunctions::PromiseBatchActionDeployContract,
		x if x == HostFunctions::PromiseBatchActionFunctionCall as usize => HostFunctions::PromiseBatchActionFunctionCall,
		x if x == HostFunctions::PromiseBatchActionTransfer as usize => HostFunctions::PromiseBatchActionTransfer,
		x if x == HostFunctions::PromiseBatchActionDeleteAccount as usize => HostFunctions::PromiseBatchActionDeleteAccount,
		x if x == HostFunctions::PromiseResultsCount as usize => HostFunctions::PromiseResultsCount,
		x if x == HostFunctions::PromiseResult as usize => HostFunctions::PromiseResult,
		x if x == HostFunctions::PromiseReturn as usize => HostFunctions::PromiseReturn,
		x if x == HostFunctions::StorageWrite as usize => HostFunctions::StorageWrite,
		x if x == HostFunctions::StorageRead as usize => HostFunctions::StorageRead,
		x if x == HostFunctions::StorageRemove as usize => HostFunctions::StorageRemove,
		x if x == HostFunctions::StorageHasKey as usize => HostFunctions::StorageHasKey,
		x if x == HostFunctions::Gas as usize => HostFunctions::Gas,
		  _ => HostFunctions::Unknown,
	  }
	}
  }
  
  impl Into<usize> for HostFunctions {
	fn into(self) -> usize {
	  self as usize
	}
  }
  
  pub fn vmlogicerr_to_trap(err: VMLogicError) -> Trap {
	  match err {
		  VMLogicError::HostError(e) => e.into(),
		  VMLogicError::ExternalError(v) => HostError::ExternalError(v).into(),
		  VMLogicError::InconsistentStateError(e) => HostError::InconsistentStateError(e).into(),
	  }
  }
  
  impl<'a> Externals for VMLogic<'a> {
	fn invoke_index(
	  &mut self,
	  index: usize,
	  args: RuntimeArgs,
	) -> Result<Option<RuntimeValue>, Trap> {
	  match HostFunctions::from(index) {
		  HostFunctions::ReadRegister => {
			  let register_id: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.read_register(register_id, ptr)
				  .map(|_| None)
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::RegisterLen => {
			  let register_id: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.register_len(register_id)
				  .map(|ret| Some(ret.into()) )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::WriteRegister => {
			  let register_id: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let data_len: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let data_ptr: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.write_register(register_id, data_len, data_ptr)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::CurrentAccountId => {
			  let register_id: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.current_account_id(register_id)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::SignerAccountId => {
			  let register_id: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.signer_account_id(register_id)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::PredecessorAccountId => {
			  let register_id: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.predecessor_account_id(register_id)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::Input => {
			  let register_id: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.input(register_id)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::BlockNumber => {
			  self.block_number()
				  .map(|ret| Some(ret.into()) )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::BlockTimestamp => {
			  self.block_timestamp()
				  .map(|ret| Some(ret.into()) )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::StorageUsage => {
			  self.storage_usage()
				  .map(|ret| Some(ret.into()) )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::AccountBalance => {
			  let balance_ptr: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.account_balance(balance_ptr)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::AttachedDeposit => {
			  let balance_ptr: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.attached_deposit(balance_ptr)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::PrepaidGas => {
			  self.prepaid_gas()
				  .map(|ret| Some(ret.into()) )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::UsedGas => {
			  self.used_gas()
				  .map(|ret| Some(ret.into()) )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::RandomSeed => {
			  let register_id: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.random_seed(register_id)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::Sha256 => {
			  let value_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let value_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let register_id: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.sha256(value_len, value_ptr, register_id)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::Keccak256 => {
			  let value_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let value_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let register_id: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.keccak256(value_len, value_ptr, register_id)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::Keccak512 => {
			  let value_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let value_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let register_id: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.keccak512(value_len, value_ptr, register_id)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::Ripemd160 => {
			  let value_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let value_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let register_id: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.ripemd160(value_len, value_ptr, register_id)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::Ecrecover => {
			  let hash_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let hash_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let sign_len: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let sig_ptr: u64 = args.nth_checked(3).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let v: u64 = args.nth_checked(4).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let malleability_flag: u64 = args.nth_checked(5).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let register_id: u64 = args.nth_checked(6).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.ecrecover(hash_len, hash_ptr, sign_len, sig_ptr, v, malleability_flag, register_id)
				  .map(|ret| Some(ret.into()) )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::ValueReturn => {
			  let value_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let value_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.value_return(value_len, value_ptr)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::Panic => {
			  self.panic()
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::PanicUtf8 => {
			  let len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.panic_utf8(len, ptr)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::LogUtf8 => {
			  let len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.log_utf8(len, ptr)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::LogUtf16 => {
			  let len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.log_utf16(len, ptr)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::Abort => {
			  let msg_ptr: u32 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let filename_ptr: u32 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let line: u32 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let col: u32 = args.nth_checked(3).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.abort(msg_ptr, filename_ptr, line, col)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::PromiseCreate => {
			  let account_id_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let account_id_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let method_name_len: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let method_name_ptr: u64 = args.nth_checked(3).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let arguments_len: u64 = args.nth_checked(4).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let arguments_ptr: u64 = args.nth_checked(5).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let amount_ptr: u64 = args.nth_checked(6).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let gas: u64 = args.nth_checked(7).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.promise_create(account_id_len, account_id_ptr, method_name_len, method_name_ptr, arguments_len, arguments_ptr, amount_ptr, gas)
				  .map(|ret| Some(ret.into()) )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::PromiseThen => {
			  let promise_index: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let account_id_len: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let account_id_ptr: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let method_name_len: u64 = args.nth_checked(3).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let method_name_ptr: u64 = args.nth_checked(4).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let arguments_len: u64 = args.nth_checked(5).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let arguments_ptr: u64 = args.nth_checked(6).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let amount_ptr: u64 = args.nth_checked(7).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let gas: u64 = args.nth_checked(8).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.promise_then(promise_index, account_id_len, account_id_ptr, method_name_len, method_name_ptr, arguments_len, arguments_ptr, amount_ptr, gas)
				  .map(|ret| Some(ret.into()) )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::PromiseAnd => {
			  let promise_idx_ptr: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let promise_idx_count: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.promise_and(promise_idx_ptr, promise_idx_count)
				  .map(|ret| Some(ret.into()) )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::PromiseBatchCreate => {
			  let account_id_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let account_id_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.promise_batch_create(account_id_len, account_id_ptr)
				  .map(|ret| Some(ret.into()) )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::PromiseBatchThen => {
			  let promise_index: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let account_id_len: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let account_id_ptr: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.promise_batch_then(promise_index, account_id_len, account_id_ptr)
				  .map(|ret| Some(ret.into()) )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::PromiseBatchActionCreateAccount => {
			  let promise_index: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.promise_batch_action_create_account(promise_index)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::PromiseBatchActionDeployContract => {
			  let promise_index: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let code_len: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let code_ptr: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.promise_batch_action_deploy_contract(promise_index, code_len, code_ptr)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::PromiseBatchActionFunctionCall => {
			  let promise_index: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let method_name_len: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let method_name_ptr: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let arguments_len: u64 = args.nth_checked(3).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let arguments_ptr: u64 = args.nth_checked(4).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let amount_ptr: u64 = args.nth_checked(5).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let gas: u64 = args.nth_checked(6).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.promise_batch_action_function_call(promise_index, method_name_len, method_name_ptr, arguments_len, arguments_ptr, amount_ptr, gas)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::PromiseBatchActionTransfer => {
			  let promise_index: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let amount_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.promise_batch_action_transfer(promise_index, amount_ptr)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::PromiseBatchActionDeleteAccount => {
			  let promise_index: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let beneficiary_id_len: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let beneficiary_id_ptr: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.promise_batch_action_delete_account(promise_index, beneficiary_id_len, beneficiary_id_ptr)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::PromiseResultsCount => {
			  self.promise_results_count()
				  .map(|ret| Some(ret.into()) )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::PromiseResult => {
			  let result_idx: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let register_id: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.promise_result(result_idx, register_id)
				  .map(|ret| Some(ret.into()) )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::PromiseReturn => {
			  let promise_idx: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.promise_return(promise_idx)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::StorageWrite => {
			  let key_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let key_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let value_len: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let value_ptr: u64 = args.nth_checked(3).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let register_id: u64 = args.nth_checked(4).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.storage_write(key_len, key_ptr, value_len, value_ptr, register_id)
				  .map(|ret| Some(ret.into()) )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::StorageRead => {
			  let key_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let key_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let register_id: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.storage_read(key_len, key_ptr, register_id)
				  .map(|ret| Some(ret.into()) )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::StorageRemove => {
			  let key_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let key_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let register_id: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.storage_remove(key_len, key_ptr, register_id)
				  .map(|ret| Some(ret.into()) )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::StorageHasKey => {
			  let key_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  let key_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.storage_has_key(key_len, key_ptr)
				  .map(|ret| Some(ret.into()) )
				  .map_err(vmlogicerr_to_trap)
		  },
		  HostFunctions::Gas => {
			  let gas_amount: u32 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			  self.gas(gas_amount)
				  .map(|_| None )
				  .map_err(vmlogicerr_to_trap)
		  },
  
		  _ => {
			  Err(Trap::new(TrapKind::Unreachable))
		  }
	  }
	}
  }