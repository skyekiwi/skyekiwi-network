use wasmi::{Externals, RuntimeArgs, RuntimeValue, Trap, TrapKind};
use skw_vm_host::{VMLogic};

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
        // Some(self.0.read_register(register_id, ptr).unwrap())
        Ok(None)
      },
      _ => {
        Ok(None)
      }
      // HostFunctions::RegisterLen => {
      //   let register_id: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
      //   self.register_len(register_id)?
      // },
      // HostFunctions::WriteRegister => {
      //   let register_id: u64 = args.nth_checked(0).map_err(|_| TrapKind::UnexpectedSignature)?;
      //   let data_len: u64 = args.nth_checked(1).map_err(|_| TrapKind::UnexpectedSignature)?;
      //   let data_ptr: u64 = args.nth_checked(2).map_err(|_| TrapKind::UnexpectedSignature)?;
      //   self.write_register(register_id, data_len, data_ptr)?
      // },
      // _ => {
      //   println!("BOOOOO");
      // }
    }
  }
}
