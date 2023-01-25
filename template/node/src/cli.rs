use crate::service::EthConfiguration;

/// Available Sealing methods.
#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum Sealing {
	// Seal using rpc method.
	Manual,
	// Seal when transaction is executed.
	Instant,
}

impl Default for Sealing {
	fn default() -> Sealing {
		Sealing::Manual
	}
}

#[derive(Debug, clap::Parser)]
pub struct Cli {
	#[command(subcommand)]
	pub subcommand: Option<Subcommand>,

	#[allow(missing_docs)]
	#[command(flatten)]
	pub run: sc_cli::RunCmd,

	/// Choose sealing method.
	#[arg(long, value_enum, ignore_case = true)]
	pub sealing: Option<Sealing>,

	#[command(flatten)]
	pub eth: EthConfiguration,

	// /// Enable EVM tracing module on a non-authority node.
	// #[arg(
	// 	long,
	// 	conflicts_with = "collator",
	// 	conflicts_with = "validator",
	// 	value_delimiter = ','
	// )]
	// pub ethapi: Vec<EthApi>,
	//
	// /// Number of concurrent tracing tasks. Meant to be shared by both "debug" and "trace" modules.
	// #[arg(long, default_value = "10")]
	// pub ethapi_max_permits: u32,
	//
	// /// Maximum number of trace entries a single request of `trace_filter` is allowed to return.
	// /// A request asking for more or an unbounded one going over this limit will both return an
	// /// error.
	// #[arg(long, default_value = "500")]
	// pub ethapi_trace_max_count: u32,
	//
	// /// Duration (in seconds) after which the cache of `trace_filter` for a given block will be
	// /// discarded.
	// #[arg(long, default_value = "300")]
	// pub ethapi_trace_cache_duration: u64,
	//
	// /// Size in bytes of the LRU cache for block data.
	// #[arg(long, default_value = "300000000")]
	// pub eth_log_block_cache: usize,
	//
	// /// Size in bytes of the LRU cache for transactions statuses data.
	// #[arg(long, default_value = "300000000")]
	// pub eth_statuses_cache: usize,
	//
	// /// Size in bytes of data a raw tracing request is allowed to use.
	// /// Bound the size of memory, stack and storage data.
	// #[arg(long, default_value = "20000000")]
	// pub tracing_raw_max_memory_usage: usize,
	//
	// /// Maximum number of logs in a query.
	// #[arg(long, default_value = "10000")]
	// pub max_past_logs: u32,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
	/// Key management cli utilities
	#[command(subcommand)]
	Key(sc_cli::KeySubcommand),

	/// Build a chain specification.
	BuildSpec(sc_cli::BuildSpecCmd),

	/// Validate blocks.
	CheckBlock(sc_cli::CheckBlockCmd),

	/// Export blocks.
	ExportBlocks(sc_cli::ExportBlocksCmd),

	/// Export the state of a given block into a chain spec.
	ExportState(sc_cli::ExportStateCmd),

	/// Import blocks.
	ImportBlocks(sc_cli::ImportBlocksCmd),

	/// Remove the whole chain.
	PurgeChain(sc_cli::PurgeChainCmd),

	/// Revert the chain to a previous state.
	Revert(sc_cli::RevertCmd),

	/// Sub-commands concerned with benchmarking.
	#[cfg(feature = "runtime-benchmarks")]
	#[command(subcommand)]
	Benchmark(frame_benchmarking_cli::BenchmarkCmd),

	/// Sub-commands concerned with benchmarking.
	#[cfg(not(feature = "runtime-benchmarks"))]
	Benchmark,

	/// Db meta columns information.
	FrontierDb(fc_cli::FrontierDbCmd),
}

/// EVM tracing CLI flags.
#[derive(Debug, PartialEq, Clone)]
pub enum EthApi {
	/// Enable EVM debug RPC methods.
	Debug,
	/// Enable EVM trace RPC methods.
	Trace,
}

impl std::str::FromStr for EthApi {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"debug" => Self::Debug,
			"trace" => Self::Trace,
			_ => {
				return Err(format!(
					"`{}` is not recognized as a supported Ethereum Api",
					s
				))
			}
		})
	}
}

/// EVM tracing CLI config.
pub struct EvmTracingConfig {
	/// Enabled EVM tracing flags.
	pub ethapi: Vec<EthApi>,
	/// Number of concurrent tracing tasks.
	pub ethapi_max_permits: u32,
	/// Maximum number of trace entries a single request of `trace_filter` is allowed to return.
	/// A request asking for more or an unbounded one going over this limit will both return an
	/// error.
	pub ethapi_trace_max_count: u32,
	/// Duration (in seconds) after which the cache of `trace_filter` for a given block will be
	/// discarded.
	pub ethapi_trace_cache_duration: u64,
	/// Size in bytes of the LRU cache for block data.
	pub eth_log_block_cache: usize,
	/// Size in bytes of the LRU cache for transactions statuses data.
	pub eth_statuses_cache: usize,
	/// Maximum number of logs in a query.
	pub max_past_logs: u32,
	/// Size in bytes of data a raw tracing request is allowed to use.
	/// Bound the size of memory, stack and storage data.
	pub tracing_raw_max_memory_usage: usize,
}
