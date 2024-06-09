use ethers_core::types::{Address, Bytes, TxHash, H256, U256};
use foundry_common::types::ToEthers;
use foundry_evm::traces::{CallKind, CallTraceNode};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::From, fmt::Debug};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct AccountOverride {
    pub nonce: Option<u64>,
    pub code: Option<Bytes>,
    pub balance: Option<U256>,
    pub state: Option<HashMap<H256, H256>>,
    pub state_diff: Option<HashMap<H256, H256>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GatherJson {
    pub hash: TxHash,
    pub rpc: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CallTraceItem {
    pub from: Address,
    pub to: Address,
    pub call_type: CallKind,
    pub gas_used: u64,
    pub input: String,
    pub output: String,
    pub children: Vec<usize>,
    pub success: bool,
}

impl From<CallTraceNode> for CallTraceItem {
    fn from(value: CallTraceNode) -> Self {
        CallTraceItem {
            from: value.trace.caller.to_ethers(),
            to: value.trace.address.to_ethers(),
            call_type: value.trace.kind,
            gas_used: value.trace.gas_used,
            input: value.trace.data.to_string(),
            output: value.trace.output.to_string(),
            children: value.children,
            success: value.trace.success,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Default)]
pub struct RunResult {
    pub local_revert: bool,
    pub traces: Vec<CallTraceItem>,
}

impl Eq for RunResult {}
