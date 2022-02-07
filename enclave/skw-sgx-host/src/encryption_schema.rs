use std::convert::TryInto;
use std::vec::Vec;

pub type BoxPublicKey = [u8; 32];
pub struct EncryptionSchema {
	is_public: bool,
	members: Vec<BoxPublicKey>,
}

impl EncryptionSchema {
	pub fn from_raw(schema: &[u8]) -> Self {
		
		let is_public = match schema[0..2] {
			[0x0, 0x0] => false,
			[0x1, 0x1] => true,
			_ => unreachable!()
		};

		let len = schema.len();
		let mut offset = 2;
		let mut members: Vec<BoxPublicKey> = Vec::new();

		while offset < len && offset +  32 <= len {
			let m: BoxPublicKey = schema[offset..offset + 32].try_into().unwrap();
			members.push(m);
			offset += 32;
		}

		Self {
			is_public,
			members,
		}
	}

	pub fn to_vec(&self) -> Vec<u8> {
		let mut result = Vec::new();
		match self.is_public {
			true => { result.push(0x1); result.push(0x1); },
			false => { result.push(0x0); result.push(0x0); }
		}

		for member in &self.members {
			result = [ &result[..], &member[..] ].concat().to_vec();
		}

		result
	}

	pub fn new(is_public: bool, members: Vec<BoxPublicKey>) -> Self {
		Self {
			is_public,
			members,
		}
	}

	pub fn get_members_count(&self) -> u64 {
		self.members.len() as u64
	}

	pub fn get_members(&self) -> &[BoxPublicKey] {
		&self.members
	}

	pub fn get_is_public(&self) -> bool {
		self.is_public
	}

	pub fn add_member(&mut self, key: BoxPublicKey) {
		self.members.push(key);
	}
}
