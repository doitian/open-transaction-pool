use ckb_types::{h256, H256};
use once_cell::sync::OnceCell;

pub const SIGHASH_TYPE_HASH: H256 =
    h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8");
pub const XUDT_DEVNET_TYPE_HASH: H256 =
    h256!("0x73e5467341b55ffd7bdeb5b6f32aa0e9433baf6808f8c5f2472dbc36b1ab04f7");
pub const ANYONE_CAN_PAY_DEVNET_TYPE_HASH: H256 =
    h256!("0x6283a479a3cf5d4276cd93594de9f1827ab9b55c7b05b3d28e4c2e0a696cfefd");
pub const OMNI_LOCK_DEVNET_TYPE_HASH: H256 =
    h256!("0xbb4469004225b39e983929db71fe2253cba1d49a76223e9e1d212cdca1f79f28");

pub const SECP_DATA_TX_HASH: H256 =
    h256!("0x8592d17f7d574cf51b744d66fe9e14a09b915ecaf7ff40450d270c8b2a7a1372");
pub const SECP_DATA_TX_IDX: usize = 3;

pub const OMNI_OPENTX_TX_HASH: H256 =
    h256!("0x8592d17f7d574cf51b744d66fe9e14a09b915ecaf7ff40450d270c8b2a7a1372");
pub const OMNI_OPENTX_TX_IDX: usize = 9;

pub const XUDT_TX_HASH: H256 =
    h256!("0x8592d17f7d574cf51b744d66fe9e14a09b915ecaf7ff40450d270c8b2a7a1372");
pub const XUDT_TX_IDX: usize = 10;

pub static CKB_URI: OnceCell<String> = OnceCell::new();
