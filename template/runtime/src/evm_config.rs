use pallet_evm::EvmConfig;

pub const fn limitless() -> EvmConfig {
	EvmConfig {
		gas_ext_code: 0,
		gas_ext_code_hash: 0,
		gas_balance: 0,
		gas_sload: 0,
		gas_sload_cold: 0,
		gas_sstore_set: 0,
		gas_sstore_reset: 0,
		refund_sstore_clears: 0,
		max_refund_quotient: 1,
		gas_suicide: 0,
		gas_suicide_new_account: 0,
		gas_call: 0,
		gas_expbyte: 0,
		gas_transaction_create: 0,
		gas_transaction_call: 0,
		gas_transaction_zero_data: 0,
		gas_transaction_non_zero_data: 0,
		gas_access_list_address: 0,
		gas_access_list_storage_key: 0,
		gas_account_access_cold: 0,
		gas_storage_read_warm: 0,
		sstore_gas_metering: false,
		sstore_revert_under_stipend: false,
		increase_state_access_gas: false,
		decrease_clears_refund: false,
		disallow_executable_format: false,
		err_on_call_with_more_gas: false,
		empty_considered_exists: true,
		create_increase_nonce: false,
		call_l64_after_gas: false,
		stack_limit: 1024,
		memory_limit: usize::MAX,
		call_stack_limit: 1024,
		create_contract_limit: Some(0xFFFF),
		call_stipend: 0,
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
