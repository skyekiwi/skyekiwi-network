use borsh::{maybestd::io, BorshDeserialize, BorshSchema, BorshSerialize};
use serde::{de::{self}, Deserialize, Serialize};
use std::{fmt};

use super::public_key::{PublicKey, ParsePublicKeyError};

#[derive(
    Debug, Clone, PartialEq, PartialOrd, Ord, Eq, BorshSerialize, Serialize, Hash, BorshSchema,
)]
pub struct AccountId(Box<PublicKey>);

impl AccountId {
    pub fn new(public_key: PublicKey) -> Self {
		Self(Box::new(public_key.clone()))
	}

    pub fn is_system(account: &Self) -> bool {
		PublicKey::is_system(account.0.as_ref())
	}

    pub fn system() -> Self {
		AccountId(Box::new(PublicKey::system()))
    }

	pub fn test(n: u8) -> Self {
		AccountId(Box::new(PublicKey::test(n)))
    }

    /// Returns reference to the account ID bytes.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParsePublicKeyError> {
		let pk = PublicKey::from_bytes(&bytes)?;
		Ok(AccountId(Box::new(pk)))
	}
}

impl fmt::Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pk: String = self.as_ref().into();
        fmt::Display::fmt(&pk, f)
    }
}

impl From<&AccountId> for String {
    fn from(account: &AccountId) -> Self {
        account.as_ref().into()
    }
}

impl AsRef<PublicKey> for AccountId {
	fn as_ref(&self) -> &PublicKey {
		&self.0
	}
}

#[cfg(not(target_arch = "wasm32"))]
impl From<skw_vm_host::types::AccountId> for AccountId {
    fn from(account: skw_vm_host::types::AccountId) -> Self {
        Self::new(account.as_bytes()[..].to_vec().try_into().unwrap())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Into<skw_vm_host::types::AccountId> for AccountId {
    fn into(self) -> skw_vm_host::types::AccountId {
        skw_vm_host::types::AccountId::from_bytes(
            self.as_bytes().try_into().unwrap()
        ).unwrap()
    }

}

impl<'de> de::Deserialize<'de> for AccountId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let account_id = <PublicKey as Deserialize>::deserialize(deserializer)?;
        Ok(AccountId(Box::new(account_id)))
    }
}

impl BorshDeserialize for AccountId {
    fn deserialize(buf: &mut &[u8]) -> io::Result<Self> {
        let account_id = <PublicKey as BorshDeserialize>::deserialize(buf)?;
        Ok(Self(Box::new(account_id)))
    }
}
