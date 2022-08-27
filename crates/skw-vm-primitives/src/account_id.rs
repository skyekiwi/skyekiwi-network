use crate::{crypto::PublicKey};

#[derive(Eq, Ord, Hash, Clone, Debug, PartialEq, PartialOrd,)]
pub struct AccountId(pub Box<PublicKey>);

#[cfg(feature = "borsh")]
mod borsh {
	use super::AccountId;
    use crate::crypto::PublicKey;

	use std::io::{Error, Write};

	use borsh::{BorshDeserialize, BorshSerialize};

	impl BorshSerialize for AccountId {
		fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
			self.0.serialize(writer)
		}
	}

	impl BorshDeserialize for AccountId {
		fn deserialize(buf: &mut &[u8]) -> Result<Self, std::io::Error> {
			let account_id = Box::<PublicKey>::deserialize(buf)?;
			Ok(Self(account_id))
		}
	}

	// #[cfg(test)]
	// mod tests {
	// 	use super::{
	// 		super::tests::{BAD_ACCOUNT_IDS, OK_ACCOUNT_IDS},
	// 		*,
	// 	};

	// 	#[test]
	// 	fn test_is_valid_account_id() {
	// 		for account_id in OK_ACCOUNT_IDS.iter().cloned() {
	// 			let parsed_account_id = account_id.parse::<AccountId>().unwrap_or_else(|err| {
	// 				panic!("Valid account id {:?} marked invalid: {}", account_id, err)
	// 			});

	// 			let str_serialized_account_id = account_id.try_to_vec().unwrap();

	// 			let deserialized_account_id = AccountId::try_from_slice(&str_serialized_account_id)
	// 				.unwrap_or_else(|err| {
	// 					panic!("failed to deserialize account ID {:?}: {}", account_id, err)
	// 				});
	// 			assert_eq!(deserialized_account_id, parsed_account_id);

	// 			let serialized_account_id =
	// 				deserialized_account_id.try_to_vec().unwrap_or_else(|err| {
	// 					panic!("failed to serialize account ID {:?}: {}", account_id, err)
	// 				});
	// 			assert_eq!(serialized_account_id, str_serialized_account_id);
	// 		}

	// 		for account_id in BAD_ACCOUNT_IDS.iter().cloned() {
	// 			let str_serialized_account_id = account_id.try_to_vec().unwrap();

	// 			assert!(
	// 				AccountId::try_from_slice(&str_serialized_account_id).is_err(),
	// 				"successfully deserialized invalid account ID {:?}",
	// 				account_id
	// 			);
	// 		}
	// 	}
	// }

}

#[cfg(feature = "serde")]
mod serde {
	use super::AccountId;
    use crate::crypto::PublicKey;

	use serde::{de, ser};

	impl ser::Serialize for AccountId {
		fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
		where
			S: ser::Serializer,
		{
			self.0.serialize(serializer)
		}
	}

	impl<'de> de::Deserialize<'de> for AccountId {
		fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
		where
			D: de::Deserializer<'de>,
		{
			let account_id = Box::<PublicKey>::deserialize(deserializer)?;
			Ok(AccountId(account_id))
		}
	}

	// #[cfg(test)]
	// mod tests {
	// 	use super::{
	// 		super::tests::{BAD_ACCOUNT_IDS, OK_ACCOUNT_IDS},
	// 		AccountId,
	// 	};
	// 	use serde_json::json;

	// 	#[test]
	// 	fn test_is_valid_account_id() {
	// 		for account_id in OK_ACCOUNT_IDS.iter().cloned() {
	// 			let parsed_account_id = account_id.parse::<AccountId>().unwrap_or_else(|err| {
	// 				panic!("Valid account id {:?} marked invalid: {}", account_id, err)
	// 			});

	// 			let deserialized_account_id: AccountId = serde_json::from_value(json!(account_id))
	// 				.unwrap_or_else(|err| {
	// 					panic!("failed to deserialize account ID {:?}: {}", account_id, err)
	// 				});
	// 			assert_eq!(deserialized_account_id, parsed_account_id);

	// 			let serialized_account_id = serde_json::to_value(&deserialized_account_id)
	// 				.unwrap_or_else(|err| {
	// 					panic!("failed to serialize account ID {:?}: {}", account_id, err)
	// 				});
	// 			assert_eq!(serialized_account_id, json!(account_id));
	// 		}

	// 		for account_id in BAD_ACCOUNT_IDS.iter().cloned() {
	// 			assert!(
	// 				serde_json::from_value::<AccountId>(json!(account_id)).is_err(),
	// 				"successfully deserialized invalid account ID {:?}",
	// 				account_id
	// 			);
	// 		}
	// 	}
	// }

}

impl AccountId {

	pub fn new(public_key: PublicKey) -> Self {
		Self(Box::new(public_key.clone()))
	}

    pub fn len(&self) -> usize {
        self.0.len()
    }

	pub fn is_system(account: &Self) -> bool {
		PublicKey::is_system(account.0.as_ref())
	}

    pub fn system() -> Self {
		AccountId(Box::new(PublicKey::system()))
    }

	pub fn test() -> Self {
		AccountId(Box::new(PublicKey::test()))
    }

	pub fn test2() -> Self {
		AccountId(Box::new(PublicKey::test2()))
    }

	pub fn validate(_: &Self) -> Result<(), ()> {
		Ok(())
	}

	// Todo: this is a test func
	pub fn from_bytes(bytes: [u8; 32]) -> Self {
		AccountId(Box::new(PublicKey::from_bytes(bytes)))
	}
}

impl From<&AccountId> for String {
    fn from(account: &AccountId) -> Self {
        account.0.to_string()
    }
}

impl std::ops::Deref for AccountId {
	type Target = PublicKey;

	fn deref(&self) -> &Self::Target {
		self.0.as_ref()
	}
}

impl AsRef<PublicKey> for AccountId {
	fn as_ref(&self) -> &PublicKey {
		self
	}
}

impl std::borrow::Borrow<PublicKey> for AccountId {
	fn borrow(&self) -> &PublicKey {
		self
	}
}

impl std::fmt::Display for AccountId {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		std::fmt::Display::fmt(&self.0, f)
	}
}