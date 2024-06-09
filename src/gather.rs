use crate::{
    model::{CallTraceItem, RunResult},
    utils::{evm_spec, get_provider},
};
use ethers_core::types::TxHash;
use ethers_core::types::{Block, Transaction};
use ethers_providers::Middleware;
use foundry_common::types::ToAlloy;
use foundry_config::Config;
use foundry_evm::executors::{Executor, ExecutorBuilder, RawCallResult};
use foundry_evm::revm::primitives::EnvWithHandlerCfg;
use foundry_evm_core::{backend::Backend, opts::EvmOpts, utils::configure_tx_env};

pub async fn gather(rpc: String, hash: TxHash) -> Result<RunResult, ()> {
    let figment = Config::figment().merge(("eth_rpc_url", rpc));
    let mut evm_opts = figment.extract::<EvmOpts>().unwrap();
    let config = Config::from_provider(figment).sanitized();

    let provider = get_provider(&config);
    let mut tx = provider.get_transaction(hash).await.unwrap().unwrap();
    let block = provider
        .get_block_with_txs(tx.block_number.unwrap())
        .await
        .unwrap()
        .unwrap();

    evm_opts.fork_url = Some(config.get_rpc_url_or_localhost_http().unwrap().into_owned());
    evm_opts.fork_block_number = Some(tx.block_number.unwrap().as_u64() - 1);

    let env = evm_opts.evm_env().await.unwrap();
    let db = Backend::spawn(evm_opts.get_fork(&config, env.clone()));

    let spec = evm_spec(&config.evm_version);

    let builder = ExecutorBuilder::default().spec(spec);

    let mut executor = builder.build(env, db.clone());

    let mut env = executor.env.clone();

    tx = execute_block_until_tx(
        block,
        tx.transaction_index.unwrap().as_u64(),
        &mut env,
        &mut executor,
    );

    let result = {
        executor.set_tracing(true);

        configure_tx_env(&mut env, &tx.clone().to_alloy());

        let mut run_result: RunResult = RunResult {
            ..Default::default()
        };

        if let Some(_) = tx.to {
            let RawCallResult {
                reverted,
                traces,
                exit_reason: _,
                ..
            } = executor.commit_tx_with_env(env).unwrap();
            run_result.local_revert = reverted;
            run_result.traces = traces
                .unwrap()
                .into_nodes()
                .iter()
                .cloned()
                .map(CallTraceItem::from)
                .collect();
        } else {
            let RawCallResult { traces, .. }: RawCallResult =
                executor.deploy_with_env(env, None).unwrap().raw;
            run_result.local_revert = false;
            run_result.traces = traces
                .unwrap()
                .into_nodes()
                .iter()
                .cloned()
                .map(CallTraceItem::from)
                .collect();
        }

        run_result
    };

    Ok(result)
}

/// Execute the block until the transaction is found, and return the transaction
/// This updates the executor state
fn execute_block_until_tx(
    block: Block<Transaction>,
    transaction_index: u64,
    env: &mut EnvWithHandlerCfg,
    executor: &mut Executor,
) -> Transaction {
    for (_, temp_tx) in block.transactions.into_iter().enumerate() {
        if temp_tx
            .transaction_index
            .unwrap()
            .as_u64()
            .eq(&transaction_index)
        {
            return temp_tx;
        }
        configure_tx_env(env, &temp_tx.clone().to_alloy());
        if let Some(_) = temp_tx.to {
            let _ = executor.commit_tx_with_env(env.clone());
        } else {
            let _ = executor.deploy_with_env(env.clone(), None);
        }
    }
    panic!("Transaction not found");
}
