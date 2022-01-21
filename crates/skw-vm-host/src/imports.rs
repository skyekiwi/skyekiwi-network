use wasmi::{
  Error as InterpeterError, FuncInstance, FuncRef, ImportsBuilder, ModuleImportResolver,
  Signature, ValueType, MemoryRef, MemoryDescriptor,
};

use skw_vm_primitives::errors::HostError;
use crate::externals::HostFunctions;

pub fn create_builder(resolver: &dyn ModuleImportResolver) -> ImportsBuilder {
  ImportsBuilder::new().with_resolver("env", resolver)
}

#[derive(Debug, Clone)]
pub struct WasmiImportResolver(Option<MemoryRef>);

impl WasmiImportResolver {
	pub fn new(memory: MemoryRef) -> Self {
		WasmiImportResolver(Some(memory))
	}
}

impl ModuleImportResolver for WasmiImportResolver {
	fn resolve_func(
		&self, 
		func_name: &str,
		_signature: &Signature,
	) -> Result<FuncRef, InterpeterError> {
		match func_name {
			"read_register" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64, ValueType::I64][..], None),
				HostFunctions::ReadRegister.into(),
			)),
			"register_len" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64][..], Some(ValueType::I64)),
				HostFunctions::RegisterLen.into(),
			)),
			"write_register" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64, ValueType::I64, ValueType::I64][..] , None),
				HostFunctions::WriteRegister.into(),
			)),
			"current_account_id" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64][..], None),
				HostFunctions::CurrentAccountId.into(),
			)),
			"signer_account_id" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64][..], None),
				HostFunctions::SignerAccountId.into(),
			)),
			"signer_account_pk" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64][..] , None),
				HostFunctions::SignerAccountPublicKey.into(),
			)),
			"predecessor_account_id" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64][..], None),
				HostFunctions::PredecessorAccountId.into(),
			)),
			"input" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64][..], None),
				HostFunctions::Input.into(),
			)),
			"block_number" => Ok(FuncInstance::alloc_host(
				Signature::new(&[][..], Some(ValueType::I64)),
				HostFunctions::BlockNumber.into(),
			)),
			"block_timestamp" => Ok(FuncInstance::alloc_host(
				Signature::new(&[][..], Some(ValueType::I64)),
				HostFunctions::BlockTimestamp.into(),
			)),
			"storage_usage" => Ok(FuncInstance::alloc_host(
				Signature::new(&[][..], Some(ValueType::I64)),
				HostFunctions::StorageUsage.into(),
			)),
			"account_balance" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64][..], None),
				HostFunctions::AccountBalance.into(),
			)),
			"attached_deposit" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64][..], None),
				HostFunctions::AttachedDeposit.into(),
			)),
			"prepaid_gas" => Ok(FuncInstance::alloc_host(
				Signature::new(&[][..], Some(ValueType::I64)),
				HostFunctions::PrepaidGas.into(),
			)),
			"used_gas" => Ok(FuncInstance::alloc_host(
				Signature::new(&[][..], Some(ValueType::I64)),
				HostFunctions::UsedGas.into(),
			)),
			"random_seed" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64][..], None),
				HostFunctions::RandomSeed.into(),
			)),
			"sha256" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64, ValueType::I64, ValueType::I64][..], None),
				HostFunctions::Sha256.into(),
			)),
			"keccak256" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64, ValueType::I64, ValueType::I64][..], None),
				HostFunctions::Keccak256.into(),
			)),
			"keccak512" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64, ValueType::I64, ValueType::I64][..], None),
				HostFunctions::Keccak512.into(),
			)),
			"ripemd160" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64, ValueType::I64, ValueType::I64][..], None),
				HostFunctions::Ripemd160.into(),
			)),
			"ecrecover" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType:: I64, ValueType::I64, ValueType::I64, ValueType::I64, ValueType::I64, ValueType::I64, ValueType::I64][..], Some(ValueType::I64)),
				HostFunctions::Ecrecover.into(),
			)),
			"value_return" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64, ValueType::I64][..], None),
				HostFunctions::ValueReturn.into(),
			)),
			"panic" => Ok(FuncInstance::alloc_host(
				Signature::new(&[][..], None),
				HostFunctions::Panic.into(),
			)),
			"panic_utf8" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64, ValueType::I64][..], None),
				HostFunctions::PanicUtf8.into(),
			)),
			"log_utf8" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64, ValueType::I64][..], None),
				HostFunctions::LogUtf8.into(),
			)),
			"log_utf16" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64, ValueType::I64][..], None),
				HostFunctions::LogUtf16.into(),
			)),
			"abort" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I32, ValueType::I32, ValueType::I32, ValueType::I32][..], None),
				HostFunctions::Abort.into(),
			)),
			"promise_create" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64, ValueType::I64, ValueType::I64, ValueType::I64, ValueType::I64, ValueType::I64, ValueType::I64, ValueType::I64][..], Some(ValueType::I64)),
				HostFunctions::PromiseCreate.into(),
			)),
			"promise_then" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64, ValueType::I64, ValueType::I64, ValueType::I64, ValueType::I64, ValueType::I64, ValueType::I64, ValueType::I64, ValueType::I64][..], Some(ValueType::I64)),
				HostFunctions::PromiseThen.into(),
			)),
			"promise_and" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64, ValueType::I64][..], Some(ValueType::I64)),
				HostFunctions::PromiseAnd.into(),
			)),
			"promise_batch_create" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64, ValueType::I64][..], Some(ValueType::I64)),
				HostFunctions::PromiseBatchCreate.into(),
			)),
			"promise_batch_then" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64, ValueType::I64, ValueType::I64][..], Some(ValueType::I64)),
				HostFunctions::PromiseBatchThen.into(),
			)),
			"promise_batch_action_deploy_contract" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64, ValueType::I64, ValueType::I64][..], None),
				HostFunctions::PromiseBatchActionDeployContract.into(),
			)),
			"promise_batch_action_function_call" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64, ValueType::I64, ValueType::I64, ValueType::I64, ValueType::I64, ValueType::I64, ValueType::I64][..], None),
				HostFunctions::PromiseBatchActionFunctionCall.into(),
			)),
			"promise_results_count" => Ok(FuncInstance::alloc_host(
				Signature::new(&[][..], Some(ValueType::I64)),
				HostFunctions::PromiseResultsCount.into(),
			)),
			"promise_result" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64, ValueType::I64][..], Some(ValueType::I64)),
				HostFunctions::PromiseResult.into(),
			)),
			"promise_return" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64][..], None),
				HostFunctions::PromiseReturn.into(),
			)),
			"storage_write" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64, ValueType::I64, ValueType::I64, ValueType::I64, ValueType::I64][..], Some(ValueType::I64)),
				HostFunctions::StorageWrite.into(),
			)),
			"storage_read" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64, ValueType::I64, ValueType::I64][..], Some(ValueType::I64)),
				HostFunctions::StorageRead.into(),
			)),
			"storage_remove" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64, ValueType::I64, ValueType::I64][..], Some(ValueType::I64)),
				HostFunctions::StorageRemove.into(),
			)),
			"storage_has_key" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I64, ValueType::I64][..], Some(ValueType::I64)),
				HostFunctions::StorageHasKey.into(),
			)),
			"gas" => Ok(FuncInstance::alloc_host(
				Signature::new(&[ValueType::I32][..], None),
				HostFunctions::Gas.into(),
			)),
			_ => Err(InterpeterError::Trap(HostError::InvalidMethodName.into()))
		}
	}

	fn resolve_memory(
		&self,
		_field_name: &str,
		_memory_type: &MemoryDescriptor,
	) -> Result<MemoryRef, InterpeterError> {
		match self.0.clone() {
			None => Err(InterpeterError::Instantiation("memory missing".to_string())),
			Some(m) => Ok(m),
		}
	}
}
