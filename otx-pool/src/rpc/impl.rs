use super::{OtxPoolRpc, OtxPoolRpcImpl};

use otx_format::types::OpenTxStatus;

use ckb_jsonrpc_types::JsonBytes;
use ckb_types::H256;
use jsonrpc_core::Result as RpcResult;

impl OtxPoolRpc for OtxPoolRpcImpl {
    fn submit_otx(&self, otx: JsonBytes) -> RpcResult<H256> {
        self.otx_pool.insert(otx).map_err(Into::into)
    }

    fn query_otx_status_by_id(&self, id: H256) -> RpcResult<Option<OpenTxStatus>> {
        Ok(self.otx_pool.get_otx_by_id(id).map(|otx| otx.status))
    }
}
