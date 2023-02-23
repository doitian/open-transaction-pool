use crate::plugin::host_service::ServiceHandler;
use crate::plugin::plugin_proxy::{MsgHandler, PluginState, RequestHandler};
use crate::plugin::Plugin;

use otx_format::jsonrpc_types::tx_view::otx_to_tx_view;
use otx_format::jsonrpc_types::OpenTransaction;
use otx_plugin_protocol::{MessageFromHost, MessageFromPlugin, PluginInfo};
use utils::aggregator::{OtxAggregator, SecpSignInfo};

use ckb_types::core::service::Request;
use ckb_types::H256;

use crossbeam_channel::{bounded, select, unbounded};
use dashmap::DashMap;

use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

#[derive(Clone)]
pub struct Context {
    pub plugin_name: String,
    pub otx_set: Arc<DashMap<H256, OpenTransaction>>,
    pub secp_sign_info: SecpSignInfo,
    pub ckb_uri: String,
    pub service_handler: ServiceHandler,
}

impl Context {
    fn new(
        plugin_name: &str,
        secp_sign_info: SecpSignInfo,
        ckb_uri: &str,
        service_handler: ServiceHandler,
    ) -> Self {
        Context {
            plugin_name: plugin_name.to_owned(),
            otx_set: Arc::new(DashMap::new()),
            secp_sign_info,
            ckb_uri: ckb_uri.to_owned(),
            service_handler,
        }
    }
}

impl Context {}

pub struct DustCollector {
    state: PluginState,
    info: PluginInfo,

    /// Send request to plugin thread, and expect a response.
    request_handler: RequestHandler,

    /// Send notifaction/response to plugin thread.
    msg_handler: MsgHandler,

    _thread: JoinHandle<()>,
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
        let (msg_handler, request_handler, thread) = DustCollector::start_process(Context::new(
            name,
            secp_sign_info,
            ckb_uri,
            service_handler,
        ))?;
        Ok(DustCollector {
            state,
            info,
            msg_handler,
            request_handler,
            _thread: thread,
        })
    }
}

impl DustCollector {
    fn start_process(
        context: Context,
    ) -> Result<(MsgHandler, RequestHandler, JoinHandle<()>), String> {
        // the host request channel receives request from host to plugin
        let (host_request_sender, host_request_receiver) = bounded(1);
        // the channel sends notifications or responses from the host to plugin
        let (host_msg_sender, host_msg_receiver) = unbounded();

        let plugin_name = context.plugin_name.to_owned();
        // this thread processes information from host to plugin
        let thread = thread::spawn(move || {
            let do_select = || -> Result<bool, String> {
                select! {
                    // request from host to plugin
                    recv(host_request_receiver) -> msg => {
                        match msg {
                            Ok(Request { responder, arguments }) => {
                                log::debug!("dust collector receives request arguments: {:?}", arguments);
                                // handle
                                let response = (0, MessageFromPlugin::Ok);
                                responder.send(response).map_err(|err| err.to_string())?;
                                Ok(false)
                            }
                            Err(err) => Err(err.to_string())
                        }
                    }
                    // repsonse/notification from host to plugin
                    recv(host_msg_receiver) -> msg => {
                        match msg {
                            Ok(msg) => {
                                log::debug!("dust collector receivers msg: {:?}", msg);
                                match msg {
                                    (_, MessageFromHost::NewInterval(elapsed)) => {
                                        Self::on_new_intervel(context.clone(), elapsed);
                                    }
                                    (_, MessageFromHost::NewOtx(otx)) => {
                                        Self::on_new_open_tx(context.clone(), otx);
                                    }
                                    (_, MessageFromHost::CommitOtx(otx_hashes)) => {
                                        Self::on_commit_open_tx(context.clone(), otx_hashes);
                                    }
                                    _ => unreachable!(),
                                }
                                Ok(false)
                            }
                            Err(err) => Err(err.to_string())
                        }
                    }
                }
            };
            loop {
                match do_select() {
                    Ok(true) => {
                        break;
                    }
                    Ok(false) => (),
                    Err(err) => {
                        log::error!("plugin {} error: {}", plugin_name, err);
                        break;
                    }
                }
            }
        });

        Ok((host_msg_sender, host_request_sender, thread))
    }

    fn on_new_open_tx(context: Context, otx: OpenTransaction) {
        let otx_hash = otx_to_tx_view(otx.clone()).unwrap().hash;
        context.otx_set.insert(otx_hash, otx);
    }

    fn on_commit_open_tx(context: Context, otx_hashes: Vec<H256>) {
        log::debug!("dust collector remove committed otx: {:?}", otx_hashes);
        otx_hashes.iter().for_each(|otx_hash| {
            context.otx_set.remove(otx_hash);
        })
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
