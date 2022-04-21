use bdk::bitcoin::consensus::deserialize;
use sea_orm::{entity::prelude::*, ActiveValue};

use crate::{seconds_since_epoch, to_vec_unsafe};

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "transaction"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    pub id: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub node_id: String,
    pub txid: String,
    pub raw_tx: Option<Vec<u8>>,
    pub received: Option<i64>,
    pub sent: Option<i64>,
    pub fee: Option<i64>,
    pub confirmation_time: Option<Vec<u8>>,
}

impl Model {
    pub fn to_transaction_details(&self) -> Result<bdk::TransactionDetails, bdk::Error> {
        let confirmation_time = self
            .confirmation_time
            .as_ref()
            .map(|ct| serde_json::from_slice(ct).unwrap());

        Ok(bdk::TransactionDetails {
            transaction: self
                .raw_tx
                .as_ref()
                .map(|raw_tx| deserialize(raw_tx).unwrap()),
            txid: deserialize(&to_vec_unsafe(&self.txid))?,
            received: self.received.unwrap() as u64,
            sent: self.sent.unwrap() as u64,
            fee: self.fee.map(|fee| fee as u64),
            confirmation_time,
        })
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    CreatedAt,
    UpdatedAt,
    NodeId,
    Txid,
    RawTx,
    Received,
    Sent,
    Fee,
    ConfirmationTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    Id,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = String;
    fn auto_increment() -> bool {
        false
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::String(None).def().unique(),
            Self::CreatedAt => ColumnType::BigInteger.def(),
            Self::UpdatedAt => ColumnType::BigInteger.def(),
            Self::NodeId => ColumnType::String(None).def(),
            Self::Txid => ColumnType::String(None).def(),
            Self::RawTx => ColumnType::Binary.def(),
            Self::Received => ColumnType::BigInteger.def(),
            Self::Sent => ColumnType::BigInteger.def(),
            Self::Fee => ColumnType::BigInteger.def(),
            Self::ConfirmationTime => ColumnType::Binary.def(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No RelationDef")
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: ActiveValue::Set(Uuid::new_v4().to_string()),
            ..<Self as ActiveModelTrait>::default()
        }
    }

    fn before_save(mut self, insert: bool) -> Result<Self, DbErr> {
        let now: i64 = seconds_since_epoch();
        self.updated_at = ActiveValue::Set(now);
        if insert {
            self.created_at = ActiveValue::Set(now);
        }
        Ok(self)
    }
}
