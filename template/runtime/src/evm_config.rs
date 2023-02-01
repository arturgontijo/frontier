use pallet_evm::EvmConfig;

pub const fn limitless() -> EvmConfig {
	let mut c = EvmConfig::london();
	c.create_contract_limit = None;
	c.gas_call = 0; // Artur
	c.gas_expbyte = 0; // Artur
	c.gas_transaction_create = 0; // Artur
	c.gas_transaction_call = 0; // Artur
	c.gas_transaction_zero_data = 0; // Artur
	c.gas_transaction_non_zero_data = 0; // Artur
	c.gas_access_list_address = 0;
	c.gas_access_list_storage_key = 0;
	c.has_base_fee = false;
	c
}

pub static EVM_LIMITLESS_CONFIG: EvmConfig = limitless();
