use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::{fmt, io};

use anyhow::Context;
use chrono::{DateTime, Utc};
use num_rational::Rational;
use serde::de::{self, DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Serializer;
use sha2::digest::Digest;
use smart_default::SmartDefault;
use tracing::{warn};

use crate::genesis_validate::validate_genesis;
use skw_vm_primitives::{
    serialize::{u128_dec_format, u128_dec_format_compatible},
    state_record::StateRecord,
    contract_runtime::{
        CryptoHash, Balance, BlockNumber, Gas,
    },
};

const MAX_GAS_PRICE: Balance = 10_000_000_000_000_000_000_000;

#[derive(Debug, Clone, SmartDefault, Serialize, Deserialize)]
pub struct GenesisConfig {
    /// Official time of blockchain start.
    #[default(Utc::now())]
    pub genesis_time: DateTime<Utc>,
    /// ID of the blockchain. This must be unique for every blockchain.
    /// If your testnet blockchains do not have unique chain IDs, you will have a bad time.
    pub chain_id: String,
    /// Height of genesis block.
    pub genesis_height: BlockNumber,
    /// Initial gas limit.
    pub gas_limit: Gas,
    /// Minimum gas price. It is also the initial gas price.
    #[serde(with = "u128_dec_format_compatible")]
    pub min_gas_price: Balance,
    #[serde(with = "u128_dec_format")]
    #[default(MAX_GAS_PRICE)]
    pub max_gas_price: Balance,
    /// Gas price adjustment rate
    #[default(Rational::from_integer(0))]
    pub gas_price_adjustment_rate: Rational,
    /// Total supply of tokens at genesis.
    #[serde(with = "u128_dec_format")]
    pub total_supply: Balance,
}

/// Records in storage at genesis (get split into shards at genesis creation).
#[derive(
    Debug,
    Clone,
    SmartDefault,
    derive_more::AsRef,
    derive_more::AsMut,
    derive_more::From,
    Serialize,
    Deserialize,
)]
pub struct GenesisRecords(pub Vec<StateRecord>);

/// `Genesis` has an invariant that `total_supply` is equal to the supply seen in the records.
/// However, we can't enfore that invariant. All fields are public, but the clients are expected to
/// use the provided methods for instantiation, serialization and deserialization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Genesis {
    #[serde(flatten)]
    pub config: GenesisConfig,
    pub records: GenesisRecords,
    /// Genesis object may not contain records.
    /// In this case records can be found in records_file.
    /// The idea is that all records consume too much memory,
    /// so they should be processed in streaming fashion with for_each_record.
    #[serde(skip)]
    pub records_file: PathBuf,
}

impl AsRef<GenesisConfig> for &Genesis {
    fn as_ref(&self) -> &GenesisConfig {
        &self.config
    }
}

impl GenesisConfig {
    /// Parses GenesisConfig from a JSON string.
    ///
    /// It panics if the contents cannot be parsed from JSON to the GenesisConfig structure.
    pub fn from_json(value: &str) -> Self {
        serde_json::from_str(value).expect("Failed to deserialize the genesis config.")
    }

    /// Reads GenesisConfig from a JSON file.
    ///
    /// It panics if file cannot be open or read, or the contents cannot be parsed from JSON to the
    /// GenesisConfig structure.
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let file = File::open(path).with_context(|| "Could not open genesis config file.")?;
        let reader = BufReader::new(file);
        let genesis_config: GenesisConfig = serde_json::from_reader(reader)
            .with_context(|| "Failed to deserialize the genesis records.")?;
        Ok(genesis_config)
    }

    /// Writes GenesisConfig to the file.
    pub fn to_file<P: AsRef<Path>>(&self, path: P) {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(self).expect("Error serializing the genesis config."),
        )
        .expect("Failed to create / write a genesis config file.");
    }
}

impl GenesisRecords {
    /// Parses GenesisRecords from a JSON string.
    ///
    /// It panics if the contents cannot be parsed from JSON to the GenesisConfig structure.
    pub fn from_json(value: &str) -> Self {
        serde_json::from_str(value).expect("Failed to deserialize the genesis records.")
    }

    /// Reads GenesisRecords from a JSON file.
    ///
    /// It panics if file cannot be open or read, or the contents cannot be parsed from JSON to the
    /// GenesisConfig structure.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let reader = BufReader::new(File::open(path).expect("Could not open genesis config file."));
        serde_json::from_reader(reader).expect("Failed to deserialize the genesis records.")
    }

    /// Writes GenesisRecords to the file.
    pub fn to_file<P: AsRef<Path>>(&self, path: P) {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(self).expect("Error serializing the genesis records."),
        )
        .expect("Failed to create / write a genesis records file.");
    }
}

/// Visitor for records.
/// Reads records one by one and passes them to sink.
/// If full genesis file is passed, reads records from "records" field and
/// IGNORES OTHER FIELDS.
struct RecordsProcessor<F> {
    sink: F,
}

impl<'de, F: FnMut(StateRecord)> Visitor<'de> for RecordsProcessor<&'_ mut F> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(
            "either:\
        1. array of StateRecord\
        2. map with records field which is array of StateRecord",
        )
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while let Some(record) = seq.next_element::<StateRecord>()? {
            (self.sink)(record)
        }
        Ok(())
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut me = Some(self);
        let mut has_records_field = false;
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "records" => {
                    let me =
                        me.take().ok_or_else(|| de::Error::custom("duplicate field: records"))?;
                    map.next_value_seed(me)?;
                    has_records_field = true;
                }
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        if has_records_field {
            Ok(())
        } else {
            Err(de::Error::custom("missing field: records"))
        }
    }
}

impl<'de, F: FnMut(StateRecord)> DeserializeSeed<'de> for RecordsProcessor<&'_ mut F> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

fn stream_records_from_file(
    reader: impl Read,
    mut callback: impl FnMut(StateRecord),
) -> serde_json::Result<()> {
    let mut deserializer = serde_json::Deserializer::from_reader(reader);
    let records_processor = RecordsProcessor { sink: &mut callback };
    deserializer.deserialize_any(records_processor)
}

pub struct GenesisJsonHasher {
    digest: sha2::Sha256,
}

impl GenesisJsonHasher {
    pub fn new() -> Self {
        Self { digest: sha2::Sha256::new() }
    }

    pub fn process_config(&mut self, config: &GenesisConfig) {
        let mut ser = Serializer::pretty(&mut self.digest);
        config.serialize(&mut ser).expect("Error serializing the genesis config.");
    }

    pub fn process_record(&mut self, record: &StateRecord) {
        let mut ser = Serializer::pretty(&mut self.digest);
        record.serialize(&mut ser).expect("Error serializing the genesis record.");
    }

    pub fn process_genesis(&mut self, genesis: &Genesis) {
        self.process_config(&genesis.config);
        genesis.for_each_record(|record: &StateRecord| {
            self.process_record(record);
        });
    }

    pub fn finalize(self) -> CryptoHash {
        self.digest.finalize().into()
    }
}

pub enum GenesisValidationMode {
    Full,
    UnsafeFast,
}

impl Genesis {
    pub fn new(config: GenesisConfig, records: GenesisRecords) -> Self {
        Self::new_validated(config, records, GenesisValidationMode::Full)
    }

    pub fn new_with_path<P: AsRef<Path>>(config: GenesisConfig, records_file: P) -> Self {
        Self::new_with_path_validated(config, records_file, GenesisValidationMode::Full)
    }

    /// Reads Genesis from a single file.
    pub fn from_file<P: AsRef<Path>>(path: P, genesis_validation: GenesisValidationMode) -> Self {
        let reader = BufReader::new(File::open(path).expect("Could not open genesis config file."));
        let genesis: Genesis =
            serde_json::from_reader(reader).expect("Failed to deserialize the genesis records.");
        // As serde skips the `records_file` field, we can assume that `Genesis` has `records` and
        // doesn't have `records_file`.
        Self::new_validated(genesis.config, genesis.records, genesis_validation)
    }

    /// Reads Genesis from config and records files.
    pub fn from_files<P1, P2>(
        config_path: P1,
        records_path: P2,
        genesis_validation: GenesisValidationMode,
    ) -> Self
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let config = GenesisConfig::from_file(config_path).unwrap();
        Self::new_with_path_validated(config, records_path, genesis_validation)
    }

    fn new_validated(
        config: GenesisConfig,
        records: GenesisRecords,
        genesis_validation: GenesisValidationMode,
    ) -> Self {
        let genesis = Self { config, records, records_file: PathBuf::new() };
        genesis.validate(genesis_validation)
    }

    fn new_with_path_validated<P: AsRef<Path>>(
        config: GenesisConfig,
        records_file: P,
        genesis_validation: GenesisValidationMode,
    ) -> Self {
        let genesis = Self {
            config,
            records: GenesisRecords(vec![]),
            records_file: records_file.as_ref().to_path_buf(),
        };
        genesis.validate(genesis_validation)
    }

    fn validate(self, genesis_validation: GenesisValidationMode) -> Self {
        match genesis_validation {
            GenesisValidationMode::Full => {
                validate_genesis(&self);
            }
            GenesisValidationMode::UnsafeFast => {
                warn!(target: "genesis", "Skipped genesis validation");
            }
        }
        self
    }
    /// Writes Genesis to the file.
    pub fn to_file<P: AsRef<Path>>(&self, path: P) {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(self).expect("Error serializing the genesis config."),
        )
        .expect("Failed to create / write a genesis config file.");
    }

    /// Hash of the json-serialized input.
    /// DEVNOTE: the representation is not unique, and could change on upgrade.
    pub fn json_hash(&self) -> CryptoHash {
        let mut hasher = GenesisJsonHasher::new();
        hasher.process_genesis(self);
        hasher.finalize()
    }

    fn stream_records_with_callback(&self, callback: impl FnMut(StateRecord)) -> io::Result<()> {
        let reader = BufReader::new(File::open(&self.records_file)?);
        stream_records_from_file(reader, callback).map_err(io::Error::from)
    }

    /// If records vector is empty processes records stream from records_file.
    /// May panic if records_file is removed or is in wrong format.
    pub fn for_each_record(&self, mut callback: impl FnMut(&StateRecord)) {
        if self.records.as_ref().is_empty() {
            let callback_move = |record: StateRecord| {
                callback(&record);
            };
            self.stream_records_with_callback(callback_move)
                .expect("error while streaming records");
        } else {
            for record in self.records.as_ref() {
                callback(record);
            }
        }
    }
}

pub fn get_initial_supply(records: &[StateRecord]) -> Balance {
    let mut total_supply = 0;
    for record in records {
        if let StateRecord::Account { account, .. } = record {
            total_supply += account.amount() + account.locked();
        }
    }
    total_supply
}

// #[cfg(test)]
// mod test {
//     use crate::genesis_config::RecordsProcessor;
//     use skw_vm_primitives::state_record::StateRecord;
//     use serde::Deserializer;

//     fn stream_records_from_json_str(genesis: &str) -> serde_json::Result<()> {
//         let mut deserializer = serde_json::Deserializer::from_reader(genesis.as_bytes());
//         let records_processor = RecordsProcessor { sink: &mut |_record: StateRecord| {} };
//         deserializer.deserialize_any(records_processor)
//     }

//     #[test]
//     fn test_genesis_with_empty_records() {
//         let genesis = r#"{
//             "a": [1, 2],
//             "b": "random",
//             "records": []
//         }"#;
//         stream_records_from_json_str(genesis).expect("error reading empty records");
//     }

//     #[test]
//     #[should_panic(expected = "missing field: records")]
//     fn test_genesis_with_no_records() {
//         let genesis = r#"{
//             "a": [1, 2],
//             "b": "random"
//         }"#;
//         stream_records_from_json_str(genesis).unwrap();
//     }

//     #[test]
//     #[should_panic(expected = "duplicate field: records")]
//     fn test_genesis_with_several_records_fields() {
//         let genesis = r#"{
//             "a": [1, 2],
//             "records": [{
//                     "Account": {
//                         "account_id": "01.near",
//                         "account": {
//                               "amount": "49999999958035075000000000",
//                               "locked": "0",
//                               "code_hash": "11111111111111111111111111111111",
//                               "storage_usage": 264
//                         }
//                     }
//                 }],
//             "b": "random",
//             "records": [{
//                     "Account": {
//                         "account_id": "01.near",
//                         "account": {
//                               "amount": "49999999958035075000000000",
//                               "locked": "0",
//                               "code_hash": "11111111111111111111111111111111",
//                               "storage_usage": 264
//                         }
//                     }
//                 }]
//         }"#;
//         stream_records_from_json_str(genesis).unwrap();
//     }

//     #[test]
//     fn test_genesis_with_fields_after_records() {
//         let genesis = r#"{
//             "a": [1, 2],
//             "b": "random",
//             "records": [
//                 {
//                     "Account": {
//                         "account_id": "01.near",
//                         "account": {
//                               "amount": "49999999958035075000000000",
//                               "locked": "0",
//                               "code_hash": "11111111111111111111111111111111",
//                               "storage_usage": 264
//                         }
//                     }
//                 }
//             ],
//             "c": {
//                 "d": 1,
//                 "e": []
//             }
//         }"#;
//         stream_records_from_json_str(genesis).expect("error reading records with a field after");
//     }

//     #[test]
//     fn test_genesis_with_fields_before_records() {
//         let genesis = r#"{
//             "a": [1, 2],
//             "b": "random",
//             "c": {
//                 "d": 1,
//                 "e": []
//             },
//             "records": [
//                 {
//                     "Account": {
//                         "account_id": "01.near",
//                         "account": {
//                               "amount": "49999999958035075000000000",
//                               "locked": "0",
//                               "code_hash": "11111111111111111111111111111111",
//                               "storage_usage": 264
//                         }
//                     }
//                 }
//             ]
//         }"#;
//         stream_records_from_json_str(genesis).expect("error reading records from genesis");
//     }

//     #[test]
//     fn test_genesis_with_several_records() {
//         let genesis = r#"{
//             "a": [1, 2],
//             "b": "random",
//             "c": {
//                 "d": 1,
//                 "e": []
//             },
//             "records": [
//                 {
//                     "Account": {
//                         "account_id": "01.near",
//                         "account": {
//                               "amount": "49999999958035075000000000",
//                               "locked": "0",
//                               "code_hash": "11111111111111111111111111111111",
//                               "storage_usage": 264
//                         }
//                     }
//                 },
//                 {
//                     "Account": {
//                         "account_id": "01.near",
//                         "account": {
//                               "amount": "49999999958035075000000000",
//                               "locked": "0",
//                               "code_hash": "11111111111111111111111111111111",
//                               "storage_usage": 264
//                         }
//                     }
//                 }
//             ]
//         }"#;
//         stream_records_from_json_str(genesis).expect("error reading records from genesis");
//     }
// }

// test genesis_config::test::test_genesis_with_fields_after_records ... FAILED
// test genesis_config::test::test_genesis_with_several_records ... FAILED
// test genesis_config::test::test_genesis_with_fields_before_records ... FAILED
// test genesis_config::test::test_genesis_with_several_records_fields - should panic ... FAILED

// test genesis_config::test::test_genesis_with_empty_records ... ok
// test genesis_config::test::test_genesis_with_no_records - should panic ... ok

// test genesis_validate::test::test_more_than_one_contract - should panic ... ok
// test genesis_validate::test::test_total_supply_not_match - should panic ... ok