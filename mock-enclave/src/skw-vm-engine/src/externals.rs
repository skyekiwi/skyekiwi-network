use wasmi::{
  Externals, RuntimeArgs, Trap, TrapKind, HostError,
  RuntimeValue,
};

use skw_vm_host::{VMLogic};
use skw_vm_primitives::errors::{VMLogicError};

#[derive(PartialEq, Eq)]
pub enum HostFunctions {
  ReadRegister = 0,
  RegisterLen = 1,
  WriteRegister = 2,
  CurrentAccountId = 3,
  SignerAccountId = 4,
  SignerAccountPublicKey = 5,
  PredecessorAccountId = 6,
  Input = 7,
  BlockNumber = 8,
  BlockTimestamp = 9,
  EpochHeight = 10,
  StorageUsage = 11,
  AccountBalance = 12,
  AttachedDeposit = 13,
  PrepaidGas = 14,
  UsedGas = 15,
  RandomSeed = 16,
  Sha256 = 17,
  Keccak256 = 18,
  Keccak512 = 19,
  Ripemd160 = 20,
  Ecrecover = 21,
  ValueReturn = 22,
  Panic = 23,
  PanicUtf8 = 24,
  LogUtf8 = 25,
  LogUtf16 = 26,
  Abort = 27,
  PromiseCreate = 28,
  PromiseThen = 29,
  PromiseAnd = 30,
  PromiseBatchCreate = 31,
  PromiseBatchThen = 32,
  PromiseBatchActionCreateAccount = 33,
  PromiseBatchActionDeployContract = 34,
  PromiseBatchActionFunctionCall = 35,
  PromiseBatchActionTransfer = 36,
  PromiseBatchActionDeleteAccount = 37,
  PromiseResultsCount = 38,
  PromiseResult = 39,
  PromiseReturn = 40,
  StorageWrite = 41,
  StorageRead = 42,
  StorageRemove = 43,
  StorageHasKey = 44,
  Gas = 45,
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
      x if x == HostFunctions::SignerAccountPublicKey as usize => HostFunctions::SignerAccountPublicKey,
      x if x == HostFunctions::PredecessorAccountId as usize => HostFunctions::PredecessorAccountId,
      x if x == HostFunctions::Input as usize => HostFunctions::Input,
      x if x == HostFunctions::BlockNumber as usize => HostFunctions::BlockNumber,
      x if x == HostFunctions::BlockTimestamp as usize => HostFunctions::BlockTimestamp,
      x if x == HostFunctions::EpochHeight as usize => HostFunctions::EpochHeight,
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

pub struct VMHost<'a>(pub VMLogic<'a>);

impl<'a> Externals for VMHost<'a> {
  fn invoke_index(
    &mut self,
    index: usize,
    args: RuntimeArgs,
  ) -> Result<Option<RuntimeValue>, Trap> {
    match HostFunctions::from(index) {
		HostFunctions::ReadRegister => {
			let register_id: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.read_register(register_id, ptr)
				.map(|_| None)
				.map_err(|e| e.into())
		},
		HostFunctions::RegisterLen => {
			let register_id: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.register_len(register_id)
				.map(|ret| Some(ret.into()) )
				.map_err(|e| e.into())
		},
		HostFunctions::WriteRegister => {
			let register_id: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let data_len: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			let data_ptr: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.write_register(register_id, data_len, data_ptr)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::CurrentAccountId => {
			let register_id: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.current_account_id(register_id)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::SignerAccountId => {
			let register_id: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.signer_account_id(register_id)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::SignerAccountPublicKey => {
			let register_id: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.signer_account_pk(register_id)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::PredecessorAccountId => {
			let register_id: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.predecessor_account_id(register_id)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::Input => {
			let register_id: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.input(register_id)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::BlockNumber => {
			self.0.block_number()
				.map(|ret| Some(ret.into()) )
				.map_err(|e| e.into())
		},
		HostFunctions::BlockTimestamp => {
			self.0.block_timestamp()
				.map(|ret| Some(ret.into()) )
				.map_err(|e| e.into())
		},
		HostFunctions::EpochHeight => {
			self.0.epoch_height()
				.map(|ret| Some(ret.into()) )
				.map_err(|e| e.into())
		},
		HostFunctions::StorageUsage => {
			self.0.storage_usage()
				.map(|ret| Some(ret.into()) )
				.map_err(|e| e.into())
		},
		HostFunctions::AccountBalance => {
			let balance_ptr: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.account_balance(balance_ptr)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::AttachedDeposit => {
			let balance_ptr: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.attached_deposit(balance_ptr)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::PrepaidGas => {
			self.0.prepaid_gas()
				.map(|ret| Some(ret.into()) )
				.map_err(|e| e.into())
		},
		HostFunctions::UsedGas => {
			self.0.used_gas()
				.map(|ret| Some(ret.into()) )
				.map_err(|e| e.into())
		},
		HostFunctions::RandomSeed => {
			let register_id: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.random_seed(register_id)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::Sha256 => {
			let value_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let value_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			let register_id: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.sha256(value_len, value_ptr, register_id)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::Keccak256 => {
			let value_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let value_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			let register_id: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.keccak256(value_len, value_ptr, register_id)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::Keccak512 => {
			let value_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let value_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			let register_id: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.keccak512(value_len, value_ptr, register_id)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::Ripemd160 => {
			let value_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let value_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			let register_id: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.ripemd160(value_len, value_ptr, register_id)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::Ecrecover => {
			let hash_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let hash_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			let sign_len: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			let sig_ptr: u64 = args.nth_checked(3).map_err(|_| TrapKind::UnexpectedSignature)?;
			let v: u64 = args.nth_checked(4).map_err(|_| TrapKind::UnexpectedSignature)?;
			let malleability_flag: u64 = args.nth_checked(5).map_err(|_| TrapKind::UnexpectedSignature)?;
			let register_id: u64 = args.nth_checked(6).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.ecrecover(hash_len, hash_ptr, sign_len, sig_ptr, v, malleability_flag, register_id)
				.map(|ret| Some(ret.into()) )
				.map_err(|e| e.into())
		},
		HostFunctions::ValueReturn => {
			let value_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let value_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.value_return(value_len, value_ptr)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::Panic => {
			self.0.panic()
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::PanicUtf8 => {
			let len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.panic_utf8(len, ptr)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::LogUtf8 => {
			let len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.log_utf8(len, ptr)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::LogUtf16 => {
			let len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.log_utf16(len, ptr)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::Abort => {
			let msg_ptr: u32 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let filename_ptr: u32 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			let line: u32 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			let col: u32 = args.nth_checked(3).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.abort(msg_ptr, filename_ptr, line, col)
				.map(|_| None )
				.map_err(|e| e.into())
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
			self.0.promise_create(account_id_len, account_id_ptr, method_name_len, method_name_ptr, arguments_len, arguments_ptr, amount_ptr, gas)
				.map(|ret| Some(ret.into()) )
				.map_err(|e| e.into())
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
			self.0.promise_then(promise_index, account_id_len, account_id_ptr, method_name_len, method_name_ptr, arguments_len, arguments_ptr, amount_ptr, gas)
				.map(|ret| Some(ret.into()) )
				.map_err(|e| e.into())
		},
		HostFunctions::PromiseAnd => {
			let promise_idx_ptr: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let promise_idx_count: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.promise_and(promise_idx_ptr, promise_idx_count)
				.map(|ret| Some(ret.into()) )
				.map_err(|e| e.into())
		},
		HostFunctions::PromiseBatchCreate => {
			let account_id_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let account_id_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.promise_batch_create(account_id_len, account_id_ptr)
				.map(|ret| Some(ret.into()) )
				.map_err(|e| e.into())
		},
		HostFunctions::PromiseBatchThen => {
			let promise_index: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let account_id_len: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			let account_id_ptr: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.promise_batch_then(promise_index, account_id_len, account_id_ptr)
				.map(|ret| Some(ret.into()) )
				.map_err(|e| e.into())
		},
		HostFunctions::PromiseBatchActionCreateAccount => {
			let promise_index: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.promise_batch_action_create_account(promise_index)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::PromiseBatchActionDeployContract => {
			let promise_index: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let code_len: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			let code_ptr: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.promise_batch_action_deploy_contract(promise_index, code_len, code_ptr)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::PromiseBatchActionFunctionCall => {
			let promise_index: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let method_name_len: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			let method_name_ptr: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			let arguments_len: u64 = args.nth_checked(3).map_err(|_| TrapKind::UnexpectedSignature)?;
			let arguments_ptr: u64 = args.nth_checked(4).map_err(|_| TrapKind::UnexpectedSignature)?;
			let amount_ptr: u64 = args.nth_checked(5).map_err(|_| TrapKind::UnexpectedSignature)?;
			let gas: u64 = args.nth_checked(6).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.promise_batch_action_function_call(promise_index, method_name_len, method_name_ptr, arguments_len, arguments_ptr, amount_ptr, gas)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::PromiseBatchActionTransfer => {
			let promise_index: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let amount_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.promise_batch_action_transfer(promise_index, amount_ptr)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::PromiseBatchActionDeleteAccount => {
			let promise_index: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let beneficiary_id_len: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			let beneficiary_id_ptr: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.promise_batch_action_delete_account(promise_index, beneficiary_id_len, beneficiary_id_ptr)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::PromiseResultsCount => {
			self.0.promise_results_count()
				.map(|ret| Some(ret.into()) )
				.map_err(|e| e.into())
		},
		HostFunctions::PromiseResult => {
			let result_idx: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let register_id: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.promise_result(result_idx, register_id)
				.map(|ret| Some(ret.into()) )
				.map_err(|e| e.into())
		},
		HostFunctions::PromiseReturn => {
			let promise_idx: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.promise_return(promise_idx)
				.map(|_| None )
				.map_err(|e| e.into())
		},
		HostFunctions::StorageWrite => {
			let key_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let key_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			let value_len: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			let value_ptr: u64 = args.nth_checked(3).map_err(|_| TrapKind::UnexpectedSignature)?;
			let register_id: u64 = args.nth_checked(4).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.storage_write(key_len, key_ptr, value_len, value_ptr, register_id)
				.map(|ret| Some(ret.into()) )
				.map_err(|e| e.into())
		},
		HostFunctions::StorageRead => {
			let key_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let key_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			let register_id: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.storage_read(key_len, key_ptr, register_id)
				.map(|ret| Some(ret.into()) )
				.map_err(|e| e.into())
		},
		HostFunctions::StorageRemove => {
			let key_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let key_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			let register_id: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.storage_remove(key_len, key_ptr, register_id)
				.map(|ret| Some(ret.into()) )
				.map_err(|e| e.into())
		},
		HostFunctions::StorageHasKey => {
			let key_len: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			let key_ptr: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.storage_has_key(key_len, key_ptr)
				.map(|ret| Some(ret.into()) )
				.map_err(|e| e.into())
		},
		HostFunctions::Gas => {
			let gas_amount: u32 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
			self.0.gas(gas_amount)
				.map(|_| None )
				.map_err(|e| e.into())
		},

		_ => {
			Err(Trap::new(TrapKind::Unreachable))
		}
    }
  }
}
