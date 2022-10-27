use cumulus_primitives_core::ParaId;
use skyekiwi_runtime::{
	AccountId, BalancesConfig, GenesisConfig, Signature, SudoConfig,
	SystemConfig, WASM_BINARY, AuraId, SessionKeys, ParachainInfoConfig, EXISTENTIAL_DEPOSIT,
	PolkadotXcmConfig, SessionConfig, CollatorSelectionConfig
};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
// const DEFAULT_PROTOCOL_ID: &str = "skw";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;
pub const PARA_ID: u32 = 2000;

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_from_seed::<AuraId>(seed)
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn template_session_keys(keys: AuraId) -> SessionKeys {
	SessionKeys { aura: keys }
}


pub fn development_config() -> Result<ChainSpec, String> {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
		properties.insert("tokenSymbol".into(), "SKW".into());
		properties.insert("tokenDecimals".into(), 12.into());
		properties.insert("ss58Format".into(), 42.into());
	
	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed("Bob"),
					),
				],
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				PARA_ID.into(),
				get_account_id_from_seed::<sr25519::Public>("Alice")
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		// Extensions
		None,
		Extensions {
			relay_chain: "rococo-local".into(),
			para_id: PARA_ID,
		},
	))
}

// pub fn testnet_alpha_config() -> Result<ChainSpec, String> {
//     ChainSpec::from_json_bytes(&include_bytes!("../../../chain-spec/alphaRaw.json")[..])
// }

// pub fn testnet_alpha_config() -> Result<ChainSpec, String> {
// 	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

// 	Ok(ChainSpec::from_genesis(
// 		// Name
// 		"SkyeKiwi Testnet Alpha",
// 		// ID
// 		"skw_alpha",
// 		ChainType::Live,
// 		move || {
// 			testnet_genesis(
// 				wasm_binary,
// 				// Initial PoA authorities
// 				vec![
// 					authority_keys_from_seed("key1"),
// 					authority_keys_from_seed("key2"),
// 				],
// 				// Sudo account
// 				get_account_id_from_seed::<sr25519::Public>("key1"),
// 				// Pre-funded accounts
// 				vec![
// 					get_account_id_from_seed::<sr25519::Public>("key1"),
// 					get_account_id_from_seed::<sr25519::Public>("key2"),
// 				],
// 				true,
// 			)
// 		},
// 		// Bootnodes
// 		vec![],
// 		// Telemetry
// 		None,
// 		// Protocol ID
// 		None,
// 		// Properties
// 		None,
// 		// Extensions
// 		None,
// 		None,
// 	))
// }

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
	root_key: AccountId,
) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			code: WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		parachain_info: ParachainInfoConfig { parachain_id: id },
		collator_selection: CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
			..Default::default()
		},
		session: SessionConfig {
			keys: invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                 // account id
						acc,                         // validator id
						template_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		polkadot_xcm: PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},

		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
	}
}
