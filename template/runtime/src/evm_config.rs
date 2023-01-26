use pallet_evm::EvmConfig;

pub const fn limitless() -> EvmConfig {
	let mut c = EvmConfig::london();
	c.create_contract_limit = None;
	c
}

pub static EVM_LIMITLESS_CONFIG: EvmConfig = limitless();
