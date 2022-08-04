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


// impl AccountId {
	// 	pub fn as_str(&self) -> &str {
// 		self
// 	}

// 	pub fn is_top_level(&self) -> bool {
// 		!self.is_system() && !self.contains('.')
// 	}

// 	pub fn is_sub_account_of(&self, parent: &AccountId) -> bool {
// 		self.strip_suffix(parent.as_str())
// 			.map_or(false, |s| !s.is_empty() && s.find('.') == Some(s.len() - 1))
// 	}

// 	pub fn is_implicit(&self) -> bool {
// 		self.len() == 64 && self.as_bytes().iter().all(|b| matches!(b, b'a'..=b'f' | b'0'..=b'9'))
// 	}

// 	pub fn is_system(&self) -> bool {
// 		self.as_str() == "system"
// 	}

// 	pub fn validate(account_id: &str) -> Result<(), ParseAccountError> {
// 		if account_id.len() < AccountId::MIN_LEN {
// 			Err(ParseAccountError { kind: ParseErrorKind::TooShort, char: None })
// 		} else if account_id.len() > AccountId::MAX_LEN {
// 			Err(ParseAccountError { kind: ParseErrorKind::TooLong, char: None })
// 		} else {
// 			// Adapted from https://github.com/near/near-sdk-rs/blob/fd7d4f82d0dfd15f824a1cf110e552e940ea9073/near-sdk/src/environment/env.rs#L819

// 			// NOTE: We don't want to use Regex here, because it requires extra time to compile it.
// 			// The valid account ID regex is /^(([a-z\d]+[-_])*[a-z\d]+\.)*([a-z\d]+[-_])*[a-z\d]+$/
// 			// Instead the implementation is based on the previous character checks.

// 			// We can safely assume that last char was a separator.
// 			let mut last_char_is_separator = true;

// 			let mut this = None;
// 			for (i, c) in account_id.chars().enumerate() {
// 				this.replace((i, c));
// 				let current_char_is_separator = match c {
// 					'a'..='z' | '0'..='9' => false,
// 					'-' | '_' | '.' => true,
// 					_ => {
// 						return Err(ParseAccountError {
// 							kind: ParseErrorKind::InvalidChar,
// 							char: this,
// 						});
// 					}
// 				};
// 				if current_char_is_separator && last_char_is_separator {
// 					return Err(ParseAccountError {
// 						kind: ParseErrorKind::RedundantSeparator,
// 						char: this,
// 					});
// 				}
// 				last_char_is_separator = current_char_is_separator;
// 			}

// 			if last_char_is_separator {
// 				return Err(ParseAccountError {
// 					kind: ParseErrorKind::RedundantSeparator,
// 					char: this,
// 				});
// 			}
// 			Ok(())
// 		}
// 	}

// 	pub fn new_unvalidated(account_id: String) -> Self {
// 		Self(account_id.into_boxed_str())
// 	}
// }


// impl FromStr for AccountId {
// 	type Err = ParseAccountError;

// 	fn from_str(account_id: &str) -> Result<Self, Self::Err> {
// 		Self::validate(account_id)?;
// 		Ok(Self(account_id.into()))
// 	}
// }

// impl TryFrom<Box<str>> for AccountId {
// 	type Error = ParseAccountError;

// 	fn try_from(account_id: Box<str>) -> Result<Self, Self::Error> {
// 		Self::validate(&account_id)?;
// 		Ok(Self(account_id))
// 	}
// }

// impl TryFrom<String> for AccountId {
// 	type Error = ParseAccountError;

// 	fn try_from(account_id: String) -> Result<Self, Self::Error> {
// 		Self::validate(&account_id)?;
// 		Ok(Self(account_id.into_boxed_str()))
// 	}
// }

// impl fmt::Display for AccountId {
// 	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
// 		fmt::Display::fmt(&self.0, f)
// 	}
// }

// impl From<AccountId> for String {
// 	fn from(account_id: AccountId) -> Self {
// 	    account_id.0.into_string()
// 	}
// }

// impl From<AccountId> for Box<str> {
// 	fn from(value: AccountId) -> Box<str> {
// 		value.0
// 	}
// }

// pub mod errors {
// 	use std::fmt;
// 	use std::fmt::Write;

// 	/// An error which can be returned when parsing a NEAR Account ID.
// 	#[derive(Eq, Clone, Debug, PartialEq)]
// 	pub struct ParseAccountError {
// 		pub(crate) kind: ParseErrorKind,
// 		pub(crate) char: Option<(usize, char)>,
// 	}

// 	impl ParseAccountError {
// 		/// Returns the specific cause why parsing the Account ID failed.
// 		pub fn kind(&self) -> &ParseErrorKind {
// 			&self.kind
// 		}
// 	}

// 	impl std::error::Error for ParseAccountError {}
// 	impl fmt::Display for ParseAccountError {
// 		fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
// 			let mut buf = self.kind.to_string();
// 			if let Some((idx, char)) = self.char {
// 				write!(buf, " {:?} at index {}", char, idx)?
// 			}
// 			buf.fmt(f)
// 		}
// 	}

// 	/// A list of errors that occur when parsing an invalid Account ID.
// 	///
// 	/// Also see [Error kind precedence](crate::AccountId#error-kind-precedence).
// 	#[non_exhaustive]
// 	#[derive(Eq, Clone, Debug, PartialEq)]
// 	pub enum ParseErrorKind {
// 		/// The Account ID is too long.
// 		///
// 		/// Returned if the `AccountId` is longer than [`AccountId::MAX_LEN`](crate::AccountId::MAX_LEN).
// 		TooLong,
// 		/// The Account ID is too short.
// 		///
// 		/// Returned if the `AccountId` is shorter than [`AccountId::MIN_LEN`](crate::AccountId::MIN_LEN).
// 		TooShort,
// 		/// The Account ID has a redundant separator.
// 		///
// 		/// This variant would be returned if the Account ID either begins with,
// 		/// ends with or has separators immediately following each other.
// 		///
// 		/// Cases: `jane.`, `angela__moss`, `tyrell..wellick`
// 		RedundantSeparator,
// 		/// The Account ID contains an invalid character.
// 		///
// 		/// This variant would be returned if the Account ID contains an upper-case character, non-separating symbol or space.
// 		///
// 		/// Cases: `ƒelicia.near`, `user@app.com`, `Emily.near`.
// 		InvalidChar,
// 	}

// 	impl fmt::Display for ParseErrorKind {
// 		fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
// 			match self {
// 				ParseErrorKind::TooLong => "the Account ID is too long".fmt(f),
// 				ParseErrorKind::TooShort => "the Account ID is too short".fmt(f),
// 				ParseErrorKind::RedundantSeparator => "the Account ID has a redundant separator".fmt(f),
// 				ParseErrorKind::InvalidChar => "the Account ID contains an invalid character".fmt(f),
// 			}
// 		}
// 	}

// }

// pub use errors::{ParseAccountError, ParseErrorKind};

// #[cfg(test)]
// mod tests {
//     use super::*;

//     pub const OK_ACCOUNT_IDS: [&str; 24] = [
//         "aa",
//         "a-a",
//         "a-aa",
//         "100",
//         "0o",
//         "com",
//         "near",
//         "bowen",
//         "b-o_w_e-n",
//         "b.owen",
//         "bro.wen",
//         "a.ha",
//         "a.b-a.ra",
//         "system",
//         "over.9000",
//         "google.com",
//         "illia.cheapaccounts.near",
//         "0o0ooo00oo00o",
//         "alex-skidanov",
//         "10-4.8-2",
//         "b-o_w_e-n",
//         "no_lols",
//         "0123456789012345678901234567890123456789012345678901234567890123",
//         // Valid, but can't be created
//         "near.a",
//     ];

//     pub const BAD_ACCOUNT_IDS: [&str; 24] = [
//         "a",
//         "A",
//         "Abc",
//         "-near",
//         "near-",
//         "-near-",
//         "near.",
//         ".near",
//         "near@",
//         "@near",
//         "неар",
//         "@@@@@",
//         "0__0",
//         "0_-_0",
//         "0_-_0",
//         "..",
//         "a..near",
//         "nEar",
//         "_bowen",
//         "hello world",
//         "abcdefghijklmnopqrstuvwxyz.abcdefghijklmnopqrstuvwxyz.abcdefghijklmnopqrstuvwxyz",
//         "01234567890123456789012345678901234567890123456789012345678901234",
//         // `@` separators are banned now
//         "some-complex-address@gmail.com",
//         "sub.buy_d1gitz@atata@b0-rg.c_0_m",
//     ];

//     #[test]
//     fn test_is_valid_account_id() {
//         for account_id in OK_ACCOUNT_IDS.iter().cloned() {
//             if let Err(err) = AccountId::validate(account_id) {
//                 panic!("Valid account id {:?} marked invalid: {}", account_id, err.kind());
//             }
//         }

//         for account_id in BAD_ACCOUNT_IDS.iter().cloned() {
//             if let Ok(_) = AccountId::validate(account_id) {
//                 panic!("Invalid account id {:?} marked valid", account_id);
//             }
//         }
//     }

//     #[test]
//     fn test_err_kind_classification() {
//         let id = "ErinMoriarty.near".parse::<AccountId>();
//         debug_assert!(
//             matches!(
//                 id,
//                 Err(ParseAccountError { kind: ParseErrorKind::InvalidChar, char: Some((0, 'E')) })
//             ),
//             "{:?}",
//             id
//         );

//         let id = "-KarlUrban.near".parse::<AccountId>();
//         debug_assert!(
//             matches!(
//                 id,
//                 Err(ParseAccountError {
//                     kind: ParseErrorKind::RedundantSeparator,
//                     char: Some((0, '-'))
//                 })
//             ),
//             "{:?}",
//             id
//         );

//         let id = "anthonystarr.".parse::<AccountId>();
//         debug_assert!(
//             matches!(
//                 id,
//                 Err(ParseAccountError {
//                     kind: ParseErrorKind::RedundantSeparator,
//                     char: Some((12, '.'))
//                 })
//             ),
//             "{:?}",
//             id
//         );

//         let id = "jack__Quaid.near".parse::<AccountId>();
//         debug_assert!(
//             matches!(
//                 id,
//                 Err(ParseAccountError {
//                     kind: ParseErrorKind::RedundantSeparator,
//                     char: Some((5, '_'))
//                 })
//             ),
//             "{:?}",
//             id
//         );
//     }

//     #[test]
//     fn test_is_valid_top_level_account_id() {
//         let ok_top_level_account_ids = &[
//             "aa",
//             "a-a",
//             "a-aa",
//             "100",
//             "0o",
//             "com",
//             "near",
//             "bowen",
//             "b-o_w_e-n",
//             "0o0ooo00oo00o",
//             "alex-skidanov",
//             "b-o_w_e-n",
//             "no_lols",
//             "0123456789012345678901234567890123456789012345678901234567890123",
//         ];
//         for account_id in ok_top_level_account_ids {
//             assert!(
//                 account_id
//                     .parse::<AccountId>()
//                     .map_or(false, |account_id| account_id.is_top_level()),
//                 "Valid top level account id {:?} marked invalid",
//                 account_id
//             );
//         }

//         let bad_top_level_account_ids = &[
//             "ƒelicia.near", // fancy ƒ!
//             "near.a",
//             "b.owen",
//             "bro.wen",
//             "a.ha",
//             "a.b-a.ra",
//             "some-complex-address@gmail.com",
//             "sub.buy_d1gitz@atata@b0-rg.c_0_m",
//             "over.9000",
//             "google.com",
//             "illia.cheapaccounts.near",
//             "10-4.8-2",
//             "a",
//             "A",
//             "Abc",
//             "-near",
//             "near-",
//             "-near-",
//             "near.",
//             ".near",
//             "near@",
//             "@near",
//             "неар",
//             "@@@@@",
//             "0__0",
//             "0_-_0",
//             "0_-_0",
//             "..",
//             "a..near",
//             "nEar",
//             "_bowen",
//             "hello world",
//             "abcdefghijklmnopqrstuvwxyz.abcdefghijklmnopqrstuvwxyz.abcdefghijklmnopqrstuvwxyz",
//             "01234567890123456789012345678901234567890123456789012345678901234",
//             // Valid regex and length, but reserved
//             "system",
//         ];
//         for account_id in bad_top_level_account_ids {
//             assert!(
//                 !account_id
//                     .parse::<AccountId>()
//                     .map_or(false, |account_id| account_id.is_top_level()),
//                 "Invalid top level account id {:?} marked valid",
//                 account_id
//             );
//         }
//     }

//     #[test]
//     fn test_is_valid_sub_account_id() {
//         let ok_pairs = &[
//             ("test", "a.test"),
//             ("test-me", "abc.test-me"),
//             ("gmail.com", "abc.gmail.com"),
//             ("gmail.com", "abc-lol.gmail.com"),
//             ("gmail.com", "abc_lol.gmail.com"),
//             ("gmail.com", "bro-abc_lol.gmail.com"),
//             ("g0", "0g.g0"),
//             ("1g", "1g.1g"),
//             ("5-3", "4_2.5-3"),
//         ];
//         for (signer_id, sub_account_id) in ok_pairs {
//             assert!(
//                 matches!(
//                     (signer_id.parse::<AccountId>(), sub_account_id.parse::<AccountId>()),
//                     (Ok(signer_id), Ok(sub_account_id)) if sub_account_id.is_sub_account_of(&signer_id)
//                 ),
//                 "Failed to create sub-account {:?} by account {:?}",
//                 sub_account_id,
//                 signer_id
//             );
//         }

//         let bad_pairs = &[
//             ("test", ".test"),
//             ("test", "test"),
//             ("test", "a1.a.test"),
//             ("test", "est"),
//             ("test", ""),
//             ("test", "st"),
//             ("test5", "ббб"),
//             ("test", "a-test"),
//             ("test", "etest"),
//             ("test", "a.etest"),
//             ("test", "retest"),
//             ("test-me", "abc-.test-me"),
//             ("test-me", "Abc.test-me"),
//             ("test-me", "-abc.test-me"),
//             ("test-me", "a--c.test-me"),
//             ("test-me", "a_-c.test-me"),
//             ("test-me", "a-_c.test-me"),
//             ("test-me", "_abc.test-me"),
//             ("test-me", "abc_.test-me"),
//             ("test-me", "..test-me"),
//             ("test-me", "a..test-me"),
//             ("gmail.com", "a.abc@gmail.com"),
//             ("gmail.com", ".abc@gmail.com"),
//             ("gmail.com", ".abc@gmail@com"),
//             ("gmail.com", "abc@gmail@com"),
//             ("test", "a@test"),
//             ("test_me", "abc@test_me"),
//             ("gmail.com", "abc@gmail.com"),
//             ("gmail@com", "abc.gmail@com"),
//             ("gmail.com", "abc-lol@gmail.com"),
//             ("gmail@com", "abc_lol.gmail@com"),
//             ("gmail@com", "bro-abc_lol.gmail@com"),
//             ("gmail.com", "123456789012345678901234567890123456789012345678901234567890@gmail.com"),
//             (
//                 "123456789012345678901234567890123456789012345678901234567890",
//                 "1234567890.123456789012345678901234567890123456789012345678901234567890",
//             ),
//             ("aa", "ъ@aa"),
//             ("aa", "ъ.aa"),
//         ];
//         for (signer_id, sub_account_id) in bad_pairs {
//             assert!(
//                 !matches!(
//                     (signer_id.parse::<AccountId>(), sub_account_id.parse::<AccountId>()),
//                     (Ok(signer_id), Ok(sub_account_id)) if sub_account_id.is_sub_account_of(&signer_id)
//                 ),
//                 "Invalid sub-account {:?} created by account {:?}",
//                 sub_account_id,
//                 signer_id
//             );
//         }
//     }

//     #[test]
//     fn test_is_account_id_64_len_hex() {
//         let valid_64_len_hex_account_ids = &[
//             "0000000000000000000000000000000000000000000000000000000000000000",
//             "6174617461746174617461746174617461746174617461746174617461746174",
//             "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
//             "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//             "20782e20662e64666420482123494b6b6c677573646b6c66676a646b6c736667",
//         ];
//         for valid_account_id in valid_64_len_hex_account_ids {
//             assert!(
//                 matches!(
//                     valid_account_id.parse::<AccountId>(),
//                     Ok(account_id) if account_id.is_implicit()
//                 ),
//                 "Account ID {} should be valid 64-len hex",
//                 valid_account_id
//             );
//         }

//         let invalid_64_len_hex_account_ids = &[
//             "000000000000000000000000000000000000000000000000000000000000000",
//             "6.74617461746174617461746174617461746174617461746174617461746174",
//             "012-456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
//             "fffff_ffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
//             "oooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo",
//             "00000000000000000000000000000000000000000000000000000000000000",
//         ];
//         for invalid_account_id in invalid_64_len_hex_account_ids {
//             assert!(
//                 !matches!(
//                     invalid_account_id.parse::<AccountId>(),
//                     Ok(account_id) if account_id.is_implicit()
//                 ),
//                 "Account ID {} is not an implicit account",
//                 invalid_account_id
//             );
//         }
//     }
// }
