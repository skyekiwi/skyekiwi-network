use borsh::{maybestd::io, BorshDeserialize, BorshSerialize, BorshSchema};
use bs58::decode::Error as B58Error;
use std::convert::TryFrom;

/// PublicKey curve
#[derive(Debug, Clone, Copy, PartialOrd, Ord, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
#[repr(u8)]
pub enum CurveType {
    ED25519 = 0,
    SECP256K1 = 1,
    SR25519 = 2,
}

impl CurveType {
    fn from_u8(val: u8) -> Result<Self, ParsePublicKeyError> {
        match val {
            0 => Ok(CurveType::ED25519),
            1 => Ok(CurveType::SECP256K1),
            2 => Ok(CurveType::SR25519),
            _ => Err(ParsePublicKeyError { kind: ParsePublicKeyErrorKind::UnknownCurve }),
        }
    }

    /// Get the length of bytes associated to this CurveType
    const fn data_len(&self) -> usize {
        match self {
            CurveType::ED25519 => 32,
            CurveType::SECP256K1 => 64,
            CurveType::SR25519 => 32,
        }
    }
}

impl std::str::FromStr for CurveType {
    type Err = ParsePublicKeyError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq_ignore_ascii_case("ed25519") {
            Ok(CurveType::ED25519)
        } else if value.eq_ignore_ascii_case("secp256k1") {
            Ok(CurveType::SECP256K1)
        } else if value.eq_ignore_ascii_case("sr25519") {
            Ok(CurveType::SR25519)
        }else {
            Err(ParsePublicKeyError { kind: ParsePublicKeyErrorKind::UnknownCurve })
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, BorshSerialize, Hash, BorshSchema)]
pub struct PublicKey {
    data: Vec<u8>,
}

impl PublicKey {
    fn split_key_type_data(value: &str) -> Result<(CurveType, &str), ParsePublicKeyError> {
        if let Some(idx) = value.find(':') {
            let (prefix, key_data) = value.split_at(idx);
            Ok((prefix.parse::<CurveType>()?, &key_data[1..]))
        } else {
            // If there is no Default is SR25519.
            Ok((CurveType::SR25519, value))
        }
    }

    fn from_parts(curve: CurveType, data: Vec<u8>) -> Result<Self, ParsePublicKeyError> {
        let expected_length = curve.data_len();
        if data.len() != expected_length {
            return Err(ParsePublicKeyError {
                kind: ParsePublicKeyErrorKind::InvalidLength(data.len()),
            });
        }
        let mut bytes = Vec::with_capacity(1 + expected_length);
        bytes.push(curve as u8);
        bytes.extend(data);

        Ok(Self { data: bytes })
    }

    /// Returns a byte slice of this `PublicKey`'s contents.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Converts a `PublicKey` into a byte vector.
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    /// Get info about the CurveType for this public key
    pub fn curve_type(&self) -> CurveType {
        CurveType::from_u8(self.data[0]).unwrap_or_else(|_| crate::env::abort())
    }


    pub fn is_system(key: &PublicKey) -> bool {
        key.data == PublicKey::system().data
    }

    pub fn system() -> Self {
        // The PalletId: modlscontrac
        Self::from_parts(CurveType::SR25519, 
        [
            109, 111, 100, 108, 115, 99, 111, 110, 116,
            114,  97,  99,   0,   0,  0,   0,   0,   0,
            0,   0,   0,   0,   0,  0,   0,   0,   0,
            0,   0,   0,   0,   0
        ].to_vec()).unwrap()
    }

    pub fn test(n: u8) -> Self {
        Self::from_parts(CurveType::SR25519, [n; 32].to_vec()).unwrap()
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, ParsePublicKeyError> {
        match data[0] {
            0 => Self::from_parts( CurveType::ED25519, data[1..].to_vec() ),
            1 => Self::from_parts( CurveType::SECP256K1, data[1..].to_vec() ),
            2 => Self::from_parts( CurveType::SR25519, data[1..].to_vec() ),
            _ => Err(ParsePublicKeyError { kind: ParsePublicKeyErrorKind::UnknownCurve })
        }
    }
}

impl From<PublicKey> for Vec<u8> {
    fn from(v: PublicKey) -> Vec<u8> {
        v.data
    }
}

impl TryFrom<Vec<u8>> for PublicKey {
    type Error = ParsePublicKeyError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        if data.is_empty() {
            return Err(ParsePublicKeyError {
                kind: ParsePublicKeyErrorKind::InvalidLength(data.len()),
            });
        }

        let curve = CurveType::from_u8(data[0])?;
        if data.len() != curve.data_len() + 1 {
            return Err(ParsePublicKeyError {
                kind: ParsePublicKeyErrorKind::InvalidLength(data.len()),
            });
        }
        Ok(Self { data })
    }
}

impl serde::Serialize for PublicKey {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
    where
        S: serde::Serializer,
    {
        let s = hex::encode(self.as_bytes().to_vec()); 
        serializer.serialize_str(&s)
    }
}

impl<'de> serde::Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as serde::Deserializer<'de>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <String as serde::Deserialize>::deserialize(deserializer)?;
        let bytes: Vec<u8> = (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect();
        let res = PublicKey::from_bytes(&bytes).map_err(|err | serde::de::Error::custom(err.to_string()))?;
        Ok(res)
    }
}

impl BorshDeserialize for PublicKey {
    fn deserialize(buf: &mut &[u8]) -> io::Result<Self> {
        <Vec<u8> as BorshDeserialize>::deserialize(buf).and_then(|s| {
            Self::try_from(s).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        })
    }
}

impl From<&PublicKey> for String {
    fn from(str_public_key: &PublicKey) -> Self {
        match str_public_key.curve_type() {
            CurveType::ED25519 => {
                ["ed25519:", &bs58::encode(&str_public_key.data[1..]).into_string()].concat()
            }
            CurveType::SECP256K1 => {
                ["secp256k1:", &bs58::encode(&str_public_key.data[1..]).into_string()].concat()
            }
            CurveType::SR25519 => {
                ["sr25519:", &bs58::encode(&str_public_key.data[1..]).into_string()].concat()
            }
        }
    }
}

impl std::str::FromStr for PublicKey {
    type Err = ParsePublicKeyError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (curve, key_data) = PublicKey::split_key_type_data(value)?;
        let data = bs58::decode(key_data).into_vec()?;
        Self::from_parts(curve, data)
    }
}
#[derive(Debug)]
pub struct ParsePublicKeyError {
    kind: ParsePublicKeyErrorKind,
}

#[derive(Debug)]
enum ParsePublicKeyErrorKind {
    InvalidLength(usize),
    Base58(B58Error),
    UnknownCurve,
}

impl std::fmt::Display for ParsePublicKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ParsePublicKeyErrorKind::InvalidLength(l) => {
                write!(f, "invalid length of the public key, expected 32 got {}", l)
            }
            ParsePublicKeyErrorKind::Base58(e) => write!(f, "base58 decoding error: {}", e),
            ParsePublicKeyErrorKind::UnknownCurve => write!(f, "unknown curve kind"),
        }
    }
}

impl From<B58Error> for ParsePublicKeyError {
    fn from(e: B58Error) -> Self {
        Self { kind: ParsePublicKeyErrorKind::Base58(e) }
    }
}

impl std::error::Error for ParsePublicKeyError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;
    use std::str::FromStr;

    fn expected_key() -> PublicKey {
        let mut key = vec![CurveType::SR25519 as u8];
        key.extend(
            bs58::decode("6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp").into_vec().unwrap(),
        );
        key.try_into().unwrap()
    }

    #[test]
    fn test_public_key_deser() {
        let key: PublicKey =
            serde_json::from_str("\"sr25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp\"")
                .unwrap();
        assert_eq!(key, expected_key());
    }

    #[test]
    fn test_public_key_ser() {
        let key: PublicKey = expected_key();
        let actual: String = serde_json::to_string(&key).unwrap();
        assert_eq!(actual, "\"sr25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp\"");
    }

    #[test]
    fn test_public_key_from_str() {
        let key =
            PublicKey::from_str("sr25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp").unwrap();
        assert_eq!(key, expected_key());
    }

    #[test]
    fn test_public_key_to_string() {
        let key: PublicKey = expected_key();
        let actual: String = String::try_from(&key).unwrap();
        assert_eq!(actual, "sr25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp");
    }

    #[test]
    fn test_public_key_borsh_format_change() {
        // Original struct to reference Borsh serialization from
        #[derive(BorshSerialize, BorshDeserialize)]
        struct PublicKeyRef(Vec<u8>);

        let mut data = vec![CurveType::ED25519 as u8];
        data.extend(
            bs58::decode("6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp").into_vec().unwrap(),
        );

        // Test internal serialization of Vec<u8> is the same:
        let old_key = PublicKeyRef(data.clone());
        let old_encoded_key = old_key.try_to_vec().unwrap();
        let new_key: PublicKey = data.try_into().unwrap();
        let new_encoded_key = new_key.try_to_vec().unwrap();
        assert_eq!(old_encoded_key, new_encoded_key);
        assert_eq!(
            &new_encoded_key,
            &bs58::decode("279Zpep9MBBg4nKsVmTQE7NbXZkWdxti6HS1yzhp8qnc1ExS7gU")
                .into_vec()
                .unwrap()
        );

        let decoded_key = PublicKey::try_from_slice(&new_encoded_key).unwrap();
        assert_eq!(decoded_key, new_key);
    }
}