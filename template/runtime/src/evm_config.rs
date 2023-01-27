use pallet_evm::EvmConfig;

pub const fn limitless() -> EvmConfig {
	EvmConfig {
		gas_ext_code: 0,      // Artur
		gas_ext_code_hash: 0, // Artur
		gas_balance: 0,       // Artur
		gas_sload: 10,        // Artur
		gas_sload_cold: 210,
		gas_sstore_set: 2000,      // Artur
		gas_sstore_reset: 290,     // Artur
		refund_sstore_clears: 480, // Artur
		max_refund_quotient: 2,
		gas_suicide: 500,
		gas_suicide_new_account: 2500,
		gas_call: 0,                      // Artur
		gas_expbyte: 5,                   // Artur
		gas_transaction_create: 5300,     // Artur
		gas_transaction_call: 2100,       // Artur
		gas_transaction_zero_data: 2,     // Artur
		gas_transaction_non_zero_data: 4, // Artur
		gas_access_list_address: 240,
		gas_access_list_storage_key: 190,
		gas_account_access_cold: 260,
		gas_storage_read_warm: 10,
		sstore_gas_metering: false,
		sstore_revert_under_stipend: false,
		increase_state_access_gas: false,
		decrease_clears_refund: false,
		disallow_executable_format: false,
		err_on_call_with_more_gas: false, // Artur
		empty_considered_exists: true,
		create_increase_nonce: false,
		call_l64_after_gas: false,
		stack_limit: 1024,
		memory_limit: usize::MAX,
		call_stack_limit: 1024,
		create_contract_limit: Some(0xFFFF), // Artur
		call_stipend: 230,                   // Artur
		has_delegate_call: true,
		has_create2: true,
		has_revert: true,
		has_return_data: true,
		has_bitwise_shifting: true,
		has_chain_id: true,
		has_self_balance: true,
		has_ext_code_hash: true,
		has_base_fee: false,
		estimate: false,
	}
}

pub static EVM_LIMITLESS_CONFIG: EvmConfig = limitless();
