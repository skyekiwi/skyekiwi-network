use crate::{AccountId, Balance, Gas, PublicKey};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub receipt_indices: Vec<u64>,
    pub receiver_id: AccountId,
    pub actions: Vec<VmAction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum VmAction {
    CreateAccount,
    DeployContract {
        code: Vec<u8>,
    },
    FunctionCall {
        function_name: String,
        args: Vec<u8>,
        gas: Gas,
        deposit: Balance,
    },
    Transfer {
        deposit: Balance,
    },
    DeleteAccount {
        beneficiary_id: AccountId,
    },
}
