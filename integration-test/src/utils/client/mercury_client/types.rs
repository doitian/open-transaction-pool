#![allow(clippy::upper_case_acronyms)]

use super::uints::{Uint128, Uint16, Uint32, Uint64};

use ckb_jsonrpc_types::{BlockNumber, CellDep, CellOutput, OutPoint, Script, TransactionView};
use ckb_types::{bytes::Bytes, H160, H256};
use serde::{Deserialize, Serialize};

use std::cmp::{Eq, Ord, PartialEq, PartialOrd};
use std::collections::HashSet;
use std::result::Result;

pub const SECP256K1_WITNESS_LOCATION: (u32, u32) = (20, 65); // (offset, length)

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum AssetType {
    #[serde(alias = "ckb")]
    CKB,
    #[serde(alias = "udt")]
    UDT,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AssetInfo {
    pub asset_type: AssetType,
    pub udt_hash: H256,
}

impl AssetInfo {
    pub fn new_ckb() -> Self {
        AssetInfo::new(AssetType::CKB, H256::default())
    }

    pub fn new_udt(udt_hash: H256) -> Self {
        AssetInfo::new(AssetType::UDT, udt_hash)
    }

    fn new(asset_type: AssetType, udt_hash: H256) -> Self {
        AssetInfo {
            asset_type,
            udt_hash,
        }
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "type", content = "value")]
pub enum ExtraFilter {
    Dao(DaoInfo),
    Cellbase,
    /// Cell data or type is not empty, except Dao and Acp UDT cell.
    /// This is an important mark for accumulate_balance.
    Frozen,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ExtraType {
    #[serde(alias = "dao")]
    Dao,
    #[serde(alias = "cellbase")]
    Cellbase,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum StructureType {
    #[serde(alias = "native")]
    Native,
    #[serde(alias = "double_entry")]
    DoubleEntry,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum IOType {
    #[serde(alias = "input")]
    Input,
    #[serde(alias = "output")]
    Output,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "type", content = "value")]
pub enum TxView {
    TransactionWithRichStatus(TransactionWithRichStatus),
    TransactionInfo(TransactionInfo),
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum DaoState {
    Deposit(BlockNumber),
    // first is deposit block number and last is withdraw block number
    Withdraw(BlockNumber, BlockNumber),
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
#[serde(tag = "type", content = "value")]
pub enum JsonItem {
    Identity(String),
    Address(String),
    OutPoint(OutPoint),
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum TransactionStatus {
    Pending,
    Proposed,
    Committed,
    Rejected,
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum SinceFlag {
    #[serde(alias = "relative")]
    Relative,
    #[serde(alias = "absolute")]
    Absolute,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum SinceType {
    #[serde(alias = "block_number")]
    BlockNumber,
    #[serde(alias = "epoch_number")]
    EpochNumber,
    #[serde(alias = "timestamp")]
    Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum IdentityFlag {
    Ckb = 0x0,
    Ethereum = 0x1,
    Eos = 0x2,
    Tron = 0x3,
    Bitcoin = 0x4,
    Dogecoin = 0x5,
    OwnerLock = 0xFC,
    Exec = 0xFD,
    DI = 0xFE,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Identity(pub [u8; 21]);

impl Identity {
    pub fn new(flag: IdentityFlag, hash: H160) -> Self {
        let mut inner = vec![flag as u8];
        inner.extend_from_slice(&hash.0);
        Identity(to_fixed_array::<21>(&inner))
    }

    pub fn hash(&self) -> H160 {
        H160::from_slice(&self.0[1..21]).expect("build h160 from identity hash")
    }

    pub fn encode(&self) -> String {
        let mut identity_string: String = "0x".to_owned();
        identity_string.push_str(&hex::encode(self.0));
        identity_string
    }
}

impl std::convert::TryFrom<String> for Identity {
    type Error = TypeError;
    fn try_from(mut s: String) -> Result<Self, Self::Error> {
        let s = if s.starts_with("0x") {
            s.split_off(2)
        } else {
            s
        };

        if s.len() != 42 {
            return Err(TypeError::DecodeJson(
                "invalid identity item len".to_string(),
            ));
        }

        let ident = hex::decode(&s).map_err(|e| TypeError::DecodeHex(e.to_string()))?;
        Ok(Identity(to_fixed_array::<21>(&ident)))
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DaoInfo {
    pub state: DaoState,
    pub reward: Uint64,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransactionInfo {
    pub tx_hash: H256,
    pub records: Vec<Record>,
    pub fee: Uint64,
    pub burn: Vec<BurnInfo>,
    pub timestamp: Uint64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct TransactionWithRichStatus {
    pub transaction: Option<TransactionView>,
    pub tx_status: TxRichStatus,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct TxRichStatus {
    pub status: ckb_jsonrpc_types::Status,
    pub block_hash: Option<H256>,
    pub reason: Option<String>,
    pub timestamp: Option<Uint64>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub out_point: OutPoint,
    pub ownership: String,
    pub io_type: IOType,
    pub amount: Uint128,
    pub occupied: Uint64,
    pub asset_info: AssetInfo,
    pub extra: Option<ExtraFilter>,
    pub block_number: BlockNumber,
    pub epoch_number: Uint64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct BurnInfo {
    pub udt_hash: H256,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GetBalancePayload {
    pub item: JsonItem,
    pub asset_infos: HashSet<AssetInfo>,
    pub extra: Option<ExtraType>,
    pub tip_block_number: Option<BlockNumber>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct GetBalanceResponse {
    pub balances: Vec<Balance>,
    pub tip_block_number: BlockNumber,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Balance {
    pub ownership: String,
    pub asset_info: AssetInfo,
    pub free: Uint128,
    pub occupied: Uint128,
    pub frozen: Uint128,
}

impl Balance {
    pub fn new(ownership: String, asset_info: AssetInfo) -> Self {
        Balance {
            ownership,
            asset_info,
            free: 0u128.into(),
            occupied: 0u128.into(),
            frozen: 0u128.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct GetBlockInfoPayload {
    pub block_number: Option<BlockNumber>,
    pub block_hash: Option<H256>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BlockInfo {
    pub block_number: BlockNumber,
    pub block_hash: H256,
    pub parent_hash: H256,
    pub timestamp: Uint64,
    pub transactions: Vec<TransactionInfo>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GetTransactionInfoResponse {
    pub transaction: Option<TransactionInfo>,
    pub status: TransactionStatus,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QueryTransactionsPayload {
    pub item: JsonItem,
    pub asset_infos: HashSet<AssetInfo>,
    pub extra: Option<ExtraType>,
    pub block_range: Option<Range>,
    pub pagination: PaginationRequest,
    pub structure_type: StructureType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GetAccountInfoPayload {
    pub item: JsonItem,
    pub asset_info: AssetInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GetAccountInfoResponse {
    pub account_number: Uint32,
    pub account_address: String,
    pub account_type: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AdjustAccountPayload {
    pub item: JsonItem,
    pub from: Vec<JsonItem>,
    pub asset_info: AssetInfo,
    pub account_number: Option<Uint32>,
    pub extra_ckb: Option<Uint64>,
    pub fee_rate: Option<Uint64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransactionCompletionResponse {
    pub tx_view: TransactionView,
    pub script_groups: Vec<ScriptGroup>,
}

impl TransactionCompletionResponse {
    pub fn new(tx_view: TransactionView, script_groups: Vec<ScriptGroup>) -> Self {
        TransactionCompletionResponse {
            tx_view,
            script_groups,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct SudtIssuePayload {
    pub owner: String,
    pub from: Vec<JsonItem>,
    pub to: Vec<ToInfo>,
    pub output_capacity_provider: Option<OutputCapacityProvider>,
    pub fee_rate: Option<Uint64>,
    pub since: Option<SinceConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ScriptGroupType {
    Lock,
    Type,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct ScriptGroup {
    pub script: Script,
    pub group_type: ScriptGroupType,
    pub input_indices: Vec<Uint32>,
    pub output_indices: Vec<Uint32>,
}

impl ScriptGroup {
    pub fn add_group_inputs(&mut self, index: u32) {
        self.input_indices.push(index.into())
    }

    pub fn add_group_outputs(&mut self, index: u32) {
        self.output_indices.push(index.into())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct TransferPayload {
    pub asset_info: AssetInfo,
    pub from: Vec<JsonItem>,
    pub to: Vec<ToInfo>,
    pub output_capacity_provider: Option<OutputCapacityProvider>,
    pub pay_fee: Option<PayFee>,
    pub fee_rate: Option<Uint64>,
    pub since: Option<SinceConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct ToInfo {
    pub address: String,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq, Copy)]
pub enum OutputCapacityProvider {
    #[serde(alias = "from")]
    From,
    #[serde(alias = "to")]
    To,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum PayFee {
    #[serde(alias = "from")]
    From,
    #[serde(alias = "to")]
    To,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct SinceConfig {
    pub flag: SinceFlag,
    pub type_: SinceType,
    pub value: Uint64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct SimpleTransferPayload {
    pub asset_info: AssetInfo,
    pub from: Vec<String>,
    pub to: Vec<ToInfo>,
    pub fee_rate: Option<Uint64>,
    pub since: Option<SinceConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct MercuryInfo {
    pub mercury_version: String,
    pub ckb_node_version: String,
    pub network_type: NetworkType,
    pub enabled_extensions: Vec<Extension>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Extension {
    pub name: String,
    pub scripts: Vec<Script>,
    pub cell_deps: Vec<CellDep>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct DaoDepositPayload {
    pub from: Vec<JsonItem>,
    pub to: Option<String>,
    pub amount: Uint64,
    pub fee_rate: Option<Uint64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct DaoWithdrawPayload {
    pub from: Vec<JsonItem>,
    pub fee_rate: Option<Uint64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct DaoClaimPayload {
    pub from: Vec<JsonItem>,
    pub to: Option<String>,
    pub fee_rate: Option<Uint64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct GetSpentTransactionPayload {
    pub outpoint: OutPoint,
    pub structure_type: StructureType,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct CellInfo {
    cell_output: CellOutput,
    out_point: OutPoint,
    block_hash: H256,
    block_number: BlockNumber,
    data: Bytes,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
#[serde(tag = "type", content = "value")]
pub enum SyncState {
    ReadOnly,
    ParallelFirstStage(SyncProgress),
    ParallelSecondStage(SyncProgress),
    Serial(SyncProgress),
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct SyncProgress {
    pub current: String,
    pub target: String,
    pub progress: String,
}

impl SyncProgress {
    pub fn new(current: u64, target: u64, progress: String) -> Self {
        SyncProgress {
            current: current.to_string(),
            target: target.to_string(),
            progress,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, Hash, PartialEq, Eq)]
pub struct PaginationRequest {
    pub cursor: Option<Uint64>,
    pub order: Order,
    pub limit: Option<Uint16>,
    pub return_count: bool,
}

impl PaginationRequest {
    pub fn new(
        cursor: Option<u64>,
        order: Order,
        limit: Option<u16>,
        return_count: bool,
    ) -> PaginationRequest {
        PaginationRequest {
            cursor: cursor.map(Into::into),
            order,
            limit: limit.map(Into::into),
            return_count,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct PaginationResponse<T> {
    pub response: Vec<T>,
    pub next_cursor: Option<Uint64>,
    pub count: Option<Uint64>,
}

impl<T> PaginationResponse<T> {
    pub fn new(response: Vec<T>) -> Self {
        Self {
            response,
            next_cursor: None,
            count: None,
        }
    }
}

impl<T> Default for PaginationResponse<T> {
    fn default() -> Self {
        Self::new(vec![])
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Range {
    pub from: Uint64,
    pub to: Uint64,
}

#[derive(Copy, Clone, Default)]
pub struct LockFilter {
    pub is_acp: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum TypeError {
    DecodeJson(String),
    DecodeHex(String),
    InvalidRecordID(String),
    UnsupportIdentityFlag(u8),
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Order {
    #[serde(alias = "asc")]
    Asc,
    #[serde(alias = "desc")]
    Desc,
}

impl Default for Order {
    fn default() -> Self {
        Order::Asc
    }
}

impl Order {
    pub fn is_asc(&self) -> bool {
        *self == Order::Asc
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum NetworkType {
    Mainnet,
    Testnet,
    Staging,
    Dev,
}

pub fn to_fixed_array<const LEN: usize>(input: &[u8]) -> [u8; LEN] {
    assert_eq!(input.len(), LEN);
    let mut list = [0; LEN];
    list.copy_from_slice(input);
    list
}
