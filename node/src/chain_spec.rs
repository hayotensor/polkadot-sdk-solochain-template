use sc_service::ChainType;
use solochain_template_runtime::{AccountId, Signature, WASM_BINARY};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_core::{
	sr25519, Pair, Public, OpaquePeerId,
	crypto::Ss58Codec
};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;


/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn authority_keys_from_ss58(s_aura: &str, s_grandpa: &str) -> (AuraId, GrandpaId) {
	(
		aura_from_ss58_addr(s_aura),
		grandpa_from_ss58_addr(s_grandpa),
	)
}

pub fn aura_from_ss58_addr(s: &str) -> AuraId {
	Ss58Codec::from_ss58check(s).unwrap()
}

pub fn grandpa_from_ss58_addr(s: &str) -> GrandpaId {
	Ss58Codec::from_ss58check(s).unwrap()
}

pub fn get_test_accounts() -> Vec<AccountId> {
	let test_accounts = vec![
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
	];
	test_accounts
}

pub fn get_testnet_faucets() -> Vec<AccountId> {
	let faucets = vec![
		AccountId::from_ss58check("5FWa18zzvnACNpM7WmwZqhtBKeeG3e6XvKvoFjxY1QHp4HwY").unwrap()
	];
	faucets
}

// generate predictable peer ids
fn peer(id: u8) -> OpaquePeerId {
	let peer_id = format!("12D{id}KooWGFuUunX1AzAzjs3CgyqTXtPWX3AqRhJFbesGPGYHJQTP"); 
	OpaquePeerId(peer_id.into())
}

pub fn development_config() -> Result<ChainSpec, String> {
	let mut accounts = (0..255).map(|x| get_account_id_from_seed::<sr25519::Public>(&x.to_string())).collect::<Vec<_>>();
	accounts.extend(get_test_accounts());
	accounts.extend(get_testnet_faucets());
	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		None,
	)
	.with_name("Development")
	.with_id("dev")
	.with_chain_type(ChainType::Development)
	.with_genesis_config_patch(testnet_genesis(
		// Initial PoA authorities
		vec![authority_keys_from_seed("Alice")],
		// Sudo account
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		// Pre-funded accounts
		accounts,
		true,
	))
	.build())
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let mut accounts = (0..255).map(|x| get_account_id_from_seed::<sr25519::Public>(&x.to_string())).collect::<Vec<_>>();
	accounts.extend(get_test_accounts());
	accounts.extend(get_testnet_faucets());
	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		None,
	)
	.with_name("Local Testnet")
	.with_id("local_testnet")
	.with_chain_type(ChainType::Local)
	.with_genesis_config_patch(testnet_genesis(
		// Initial PoA authorities
		vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
		// Sudo account
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		// Pre-funded accounts
		accounts,
		true,
	))
	.build())
}

pub fn gavin_config() -> Result<ChainSpec, String> {
	let mut accounts = (0..255).map(|x| get_account_id_from_seed::<sr25519::Public>(&x.to_string())).collect::<Vec<_>>();
	accounts.extend(get_test_accounts());
	accounts.extend(get_testnet_faucets());
	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		None,
	)
	.with_name("Gavin Testnet")
	.with_id("gavin_testnet")
	.with_chain_type(ChainType::Development)
	.with_genesis_config_patch(testnet_genesis(
		// Initial PoA authorities
		vec![
			authority_keys_from_ss58(
				"5F46bJk2dcCmhu7s8phKsRwZCoBpi8xwgS4xknnSviqn8wwA",
				"5FjbWKESKnQpJF2BjCZ8YxqCkWK2xq9kAijcpey5jYrMTb4F",
			),
			authority_keys_from_ss58(
				"5EX5TgeLSf55eZZrfG1GDPba6b3YXJvc4CoqzBkQoiX6KVKn",
				"5HLfb4bHmQJKToTAfK4SumF3AKT17752KU63ytvgxUo8a4cD",
			),
			authority_keys_from_ss58(
				"5CrPkhgMsYHX9NgoX3bMkSGSattgw9ukVkeF8wiv7Ewnb7vv",
				"5EQzoKrJJEz8ALXnDSQFi6rv8EkvNDHrW9pVTgQ5KCtTcC37",
			),
			authority_keys_from_ss58(
				"5DxxktpYcLXtAR6BzsosXbakUFN6cHxJEyfQPPZW1c8jiK7B",
				"5HdjyBj6qMEnzsutuKvybSpSFkEaXN16KgUFqJQBxaQVPMWy",
			),
		],
		// Sudo account
		// get_account_id_from_seed::<sr25519::Public>("Alice"),
		AccountId::from_ss58check("5F46bJk2dcCmhu7s8phKsRwZCoBpi8xwgS4xknnSviqn8wwA").unwrap(),
		// Pre-funded accounts
		// vec![
		// 	get_account_id_from_seed::<sr25519::Public>("Alice"),
		// 	get_account_id_from_seed::<sr25519::Public>("Bob"),
		// 	get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
		// 	get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
		// ],
		accounts,
		true,
	))
	.build())
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> serde_json::Value {
	let subnet_path: Vec<u8> = "bigscience/bloom-560m".into();
	let mut peer_index: u8 = 0;
	serde_json::json!({
		"balances": {
			// Configure endowed accounts with initial balance of 1 << 60.
			"balances": endowed_accounts.iter().cloned().map(|k| (k, 10000000000000000000000000_u128)).collect::<Vec<_>>(),
		},
		"aura": {
			"authorities": initial_authorities.iter().map(|x| (x.0.clone())).collect::<Vec<_>>(),
		},
		"grandpa": {
			"authorities": initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect::<Vec<_>>(),
		},
		"sudo": {
			// Assign network admin rights.
			"key": Some(root_key),
		},
		"sudo": {
			"subnet_path": subnet_path,
			"memory_mb": 560,
			"subnet_nodes": endowed_accounts.iter().cloned().map(|k| {
				peer_index += 1;
				(
					k, 
					peer(peer_index),
				)
			}).collect::<Vec<_>>(),
			"accounts": endowed_accounts.iter().collect::<Vec<_>>(),
		},
	})
}
