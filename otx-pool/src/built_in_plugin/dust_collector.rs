use super::{BuiltInPlugin, Context};
use crate::notify::RuntimeHandle;
use crate::plugin::host_service::ServiceHandler;
use crate::plugin::plugin_proxy::{MsgHandler, PluginState, RequestHandler};
use crate::plugin::Plugin;

use utils::aggregator::{OtxAggregator, SecpSignInfo};

use ckb_types::core::service::Request;
use ckb_types::H256;
use otx_format::jsonrpc_types::tx_view::otx_to_tx_view;
use otx_format::jsonrpc_types::OpenTransaction;
use otx_plugin_protocol::{MessageFromHost, MessageFromPlugin, PluginInfo};

use dashmap::DashSet;

use std::path::PathBuf;
use std::sync::Arc;
use std::thread::JoinHandle;

pub struct DustCollector {
    state: PluginState,
    info: PluginInfo,

    /// Send request to stdin thread, and expect a response from stdout thread.
    request_handler: RequestHandler,

    /// Send notifaction/response to stdin thread.
    msg_handler: MsgHandler,

    _thread: JoinHandle<()>,

    _raw_otxs: Arc<DashSet<OpenTransaction>>,
    _secp_sign_info: Arc<SecpSignInfo>,
    _ckb_uri: Arc<String>,
}

impl Plugin for DustCollector {
    fn get_name(&self) -> String {
        self.info.name.clone()
    }

    fn msg_handler(&self) -> MsgHandler {
        self.msg_handler.clone()
    }

    fn request_handler(&self) -> RequestHandler {
        self.request_handler.clone()
    }

    fn get_info(&self) -> PluginInfo {
        self.info.clone()
    }

    fn get_state(&self) -> PluginState {
        self.state.clone()
    }
}

impl DustCollector {
    pub fn new(
        runtime_handle: RuntimeHandle,
        service_handler: ServiceHandler,
        secp_sign_info: SecpSignInfo,
        ckb_uri: &str,
    ) -> Result<DustCollector, String> {
        let name = "dust collector";
        let state = PluginState::new(PathBuf::default(), true, true);
        let info = PluginInfo::new(
            name,
            "Collect micropayment otx and aggregate them into ckb tx.",
            "1.0",
        );
        let raw_otxs = Arc::new(DashSet::default());
        let secp_sign_info = Arc::new(secp_sign_info);
        let ckb_uri = Arc::new(ckb_uri.to_owned());
        let context = Context::new(
            raw_otxs.clone(),
            secp_sign_info.clone(),
            ckb_uri.clone(),
            service_handler.clone(),
        );
        let (msg_handler, request_handler, thread) =
            DustCollector::start_process(context, name, runtime_handle, service_handler)?;
        Ok(DustCollector {
            state,
            info,
            msg_handler,
            request_handler,
            _thread: thread,
            _raw_otxs: raw_otxs,
            _secp_sign_info: secp_sign_info,
            _ckb_uri: ckb_uri,
        })
    }
}

impl BuiltInPlugin for DustCollector {
    fn on_new_open_tx(context: Context, otx: OpenTransaction) {
        context.otx_set.insert(otx);
    }

    fn on_new_intervel(context: Context, elapsed: u64) {
        if elapsed % 10 == 0 && context.otx_set.len() > 1 {
            log::debug!("otx set len: {:?}", context.otx_set.len());

            // merge_otx
            let _aggregator = OtxAggregator::new(
                context.secp_sign_info.secp_address(),
                context.secp_sign_info.privkey(),
                &context.ckb_uri,
            );
            let otx_list: Vec<OpenTransaction> =
                context.otx_set.iter().map(|otx| otx.clone()).collect();
            let hashes: Vec<H256> = context
                .otx_set
                .iter()
                .map(|otx| {
                    let tx_view = otx_to_tx_view(otx.clone()).unwrap();
                    tx_view.hash
                })
                .collect();
            let merged_otx = OtxAggregator::merge_otxs(otx_list);
            log::debug!("merged_otx: {}", merged_otx.is_ok());
            if let Ok(_merged_otx) = merged_otx {
                // add inputs and outputs

                // send_ckb

                // call host service
                let message = MessageFromPlugin::SendCkbTx((H256::default(), hashes));
                if let Some(MessageFromHost::Ok) = Request::call(&context.service_handler, message)
                {
                }
            }
            context.otx_set.clear()
        }
    }
}
