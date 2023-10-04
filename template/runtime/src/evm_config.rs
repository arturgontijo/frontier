use pallet_evm::EvmConfig;

pub const fn limitless() -> EvmConfig {
	let mut c = EvmConfig::istanbul();
	c.create_contract_limit = None;
	// c.has_bitwise_shifting = false;
	// c.has_create2 = false;
	// c.has_ext_code_hash = false;
	// c.sstore_gas_metering = false;
	c
}

pub static EVM_CONSTANTINOPLE_CONFIG: EvmConfig = limitless();
