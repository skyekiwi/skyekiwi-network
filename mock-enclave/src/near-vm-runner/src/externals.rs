use wasmi::{Externals, RuntimeArgs, RuntimeValue, Trap};
use crate::logic::VMLogic;
// import some errors

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
      x if x == HostFunction::ReadRegister as usize => HostFunction::ReadRegister,
      x if x == HostFunction::RegisterLen as usize => HostFunction::RegisterLen,
      x if x == HostFunction::WriteRegister as usize => HostFunction::WriteRegister,
      x if x == HostFunction::CurrentAccountId as usize => HostFunction::CurrentAccountId,
      x if x == HostFunction::SignerAccountId as usize => HostFunction::SignerAccountId,
      x if x == HostFunction::SignerAccountPublicKey as usize => HostFunction::SignerAccountPublicKey,
      x if x == HostFunction::PredecessorAccountId as usize => HostFunction::PredecessorAccountId,
      x if x == HostFunction::Input as usize => HostFunction::Input,
      x if x == HostFunction::BlockNumber as usize => HostFunction::BlockNumber,
      x if x == HostFunction::BlockTimestamp as usize => HostFunction::BlockTimestamp,
      x if x == HostFunction::EpochHeight as usize => HostFunction::EpochHeight,
      x if x == HostFunction::StorageUsage as usize => HostFunction::StorageUsage,
      x if x == HostFunction::AccountBalance as usize => HostFunction::AccountBalance,
      x if x == HostFunction::AttachedDeposit as usize => HostFunction::AttachedDeposit,
      x if x == HostFunction::PrepaidGas as usize => HostFunction::PrepaidGas,
      x if x == HostFunction::UsedGas as usize => HostFunction::UsedGas,
      x if x == HostFunction::RandomSeed as usize => HostFunction::RandomSeed,
      x if x == HostFunction::Sha256 as usize => HostFunction::Sha256,
      x if x == HostFunction::Keccak256 as usize => HostFunction::Keccak256,
      x if x == HostFunction::Keccak512 as usize => HostFunction::Keccak512,
      x if x == HostFunction::Ripemd160 as usize => HostFunction::Ripemd160,
      x if x == HostFunction::Ecrecover as usize => HostFunction::Ecrecover,
      x if x == HostFunction::ValueReturn as usize => HostFunction::ValueReturn,
      x if x == HostFunction::Panic as usize => HostFunction::Panic,
      x if x == HostFunction::PanicUtf8 as usize => HostFunction::PanicUtf8,
      x if x == HostFunction::LogUtf8 as usize => HostFunction::LogUtf8,
      x if x == HostFunction::LogUtf16 as usize => HostFunction::LogUtf16,
      x if x == HostFunction::Abort as usize => HostFunction::Abort,
      x if x == HostFunction::PromiseCreate as usize => HostFunction::PromiseCreate,
      x if x == HostFunction::PromiseThen as usize => HostFunction::PromiseThen,
      x if x == HostFunction::PromiseAnd as usize => HostFunction::PromiseAnd,
      x if x == HostFunction::PromiseBatchCreate as usize => HostFunction::PromiseBatchCreate,
      x if x == HostFunction::PromiseBatchThen as usize => HostFunction::PromiseBatchThen,
      x if x == HostFunction::PromiseBatchActionCreateAccount as usize => HostFunction::PromiseBatchActionCreateAccount,
      x if x == HostFunction::PromiseBatchActionDeployContract as usize => HostFunction::PromiseBatchActionDeployContract,
      x if x == HostFunction::PromiseBatchActionFunctionCall as usize => HostFunction::PromiseBatchActionFunctionCall,
      x if x == HostFunction::PromiseBatchActionTransfer as usize => HostFunction::PromiseBatchActionTransfer,
      x if x == HostFunction::PromiseBatchActionDeleteAccount as usize => HostFunction::PromiseBatchActionDeleteAccount,
      x if x == HostFunction::PromiseResultsCount as usize => HostFunction::PromiseResultsCount,
      x if x == HostFunction::PromiseResult as usize => HostFunction::PromiseResult,
      x if x == HostFunction::PromiseReturn as usize => HostFunction::PromiseReturn,
      x if x == HostFunction::StorageWrite as usize => HostFunction::StorageWrite,
      x if x == HostFunction::StorageRead as usize => HostFunction::StorageRead,
      x if x == HostFunction::StorageRemove as usize => HostFunction::StorageRemove,
      x if x == HostFunction::StorageHasKey as usize => HostFunction::StorageHasKey,
      x if x == HostFunction::Gas as usize => HostFunction::Gas,
        _ => HostFunction::Unknown,
    }
  }
}

impl Into<usize> for HostFunctions {
  fn into(self) -> usize {
    self as usize
  }
}

impl Externals for VMLogic {
  fn invoke_index(
    &mut self,
    index: usize,
    args: RuntimeArgs,
  ) -> Result<Option<RuntimeValue>, Trap> {
    match HostFunctions::from(index) {
      HostFunction::ReadRegister => {
        &&&
      },
      HostFunction::RegisterLen => {
        &&&
      },
      HostFunction::WriteRegister => {
        &&&
      },
      HostFunction::CurrentAccountId => {
        &&&
      },
      HostFunction::SignerAccountId => {
        &&&
      },
      HostFunction::SignerAccountPublicKey => {
        &&&
      },
      HostFunction::PredecessorAccountId => {
        &&&
      },
      HostFunction::Input => {
        &&&
      },
      HostFunction::BlockNumber => {
        &&&
      },
      HostFunction::BlockTimestamp => {
        &&&
      },
      HostFunction::EpochHeight => {
        &&&
      },
      HostFunction::StorageUsage => {
        &&&
      },
      HostFunction::AccountBalance => {
        &&&
      },
      HostFunction::AttachedDeposit => {
        &&&
      },
      HostFunction::PrepaidGas => {
        &&&
      },
      HostFunction::UsedGas => {
        &&&
      },
      HostFunction::RandomSeed => {
        &&&
      },
      HostFunction::Sha256 => {
        &&&
      },
      HostFunction::Keccak256 => {
        &&&
      },
      HostFunction::Keccak512 => {
        &&&
      },
      HostFunction::Ripemd160 => {
        &&&
      },
      HostFunction::Ecrecover => {
        &&&
      },
      HostFunction::ValueReturn => {
        &&&
      },
      HostFunction::Panic => {
        &&&
      },
      HostFunction::PanicUtf8 => {
        &&&
      },
      HostFunction::LogUtf8 => {
        &&&
      },
      HostFunction::LogUtf16 => {
        &&&
      },
      HostFunction::Abort => {
        &&&
      },
      HostFunction::PromiseCreate => {
        &&&
      },
      HostFunction::PromiseThen => {
        &&&
      },
      HostFunction::PromiseAnd => {
        &&&
      },
      HostFunction::PromiseBatchCreate => {
        &&&
      },
      HostFunction::PromiseBatchThen => {
        &&&
      },
      HostFunction::PromiseBatchActionCreateAccount => {
        &&&
      },
      HostFunction::PromiseBatchActionDeployContract => {
        &&&
      },
      HostFunction::PromiseBatchActionFunctionCall => {
        &&&
      },
      HostFunction::PromiseBatchActionTransfer => {
        &&&
      },
      HostFunction::PromiseBatchActionDeleteAccount => {
        &&&
      },
      HostFunction::PromiseResultsCount => {
        &&&
      },
      HostFunction::PromiseResult => {
        &&&
      },
      HostFunction::PromiseReturn => {
        &&&
      },
      HostFunction::StorageWrite => {
        &&&
      },
      HostFunction::StorageRead => {
        &&&
      },
      HostFunction::StorageRemove => {
        &&&
      },
      HostFunction::StorageHasKey => {
        &&&
      },
      HostFunction::Gas => {
        &&&
      },
    }
  }
}
