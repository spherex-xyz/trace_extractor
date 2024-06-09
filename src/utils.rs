use foundry_common::provider::ethers::{ProviderBuilder, RetryProvider};
use foundry_compilers::EvmVersion;
use foundry_config::Config;
use foundry_evm::revm::primitives::SpecId;

pub fn get_provider(config: &Config) -> RetryProvider {
    let url = config.get_rpc_url_or_localhost_http().unwrap();
    let chain = config.chain.unwrap_or_default().named().unwrap();
    ProviderBuilder::new(url.as_ref())
        .chain(chain)
        .build()
        .unwrap()
}

pub fn evm_spec(evm: &EvmVersion) -> SpecId {
    match evm {
        EvmVersion::Istanbul => SpecId::ISTANBUL,
        EvmVersion::Berlin => SpecId::BERLIN,
        EvmVersion::London => SpecId::LONDON,
        EvmVersion::Paris => SpecId::SHANGHAI,
        _ => panic!("Unsupported EVM version"),
    }
}
