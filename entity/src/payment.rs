use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};

use crate::seconds_since_epoch;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "payment"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Deserialize, Serialize)]
pub struct Model {
    pub id: String,
    pub node_id: String,
    pub payment_hash: String,
    pub status: String,
    pub origin: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub created_by_node_id: String,
    pub received_by_node_id: Option<String>,
    pub amt_msat: Option<i64>,
    pub fee_paid_msat: Option<i64>,
    pub preimage: Option<String>,
    pub secret: Option<String>,
    pub label: Option<String>,
    pub invoice: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    NodeId,
    CreatedByNodeId,
    ReceivedByNodeId,
    PaymentHash,
    Preimage,
    Secret,
    Status,
    Origin,
    AmtMsat,
    FeePaidMsat,
    CreatedAt,
    UpdatedAt,
    Label,
    Invoice,
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
            Self::Id => ColumnType::String(None).def(),
            Self::NodeId => ColumnType::String(None).def(),
            Self::CreatedByNodeId => ColumnType::String(None).def(),
            Self::PaymentHash => ColumnType::String(None).def(),
            Self::Status => ColumnType::String(None).def(),
            Self::Origin => ColumnType::String(None).def(),
            Self::CreatedAt => ColumnType::BigInteger.def(),
            Self::UpdatedAt => ColumnType::BigInteger.def(),
            Self::ReceivedByNodeId => ColumnType::String(None).def().null(),
            Self::AmtMsat => ColumnType::BigInteger.def().null(),
            Self::FeePaidMsat => ColumnType::BigInteger.def().null(),
            Self::Preimage => ColumnType::String(None).def().null(),
            Self::Secret => ColumnType::String(None).def().null(),
            Self::Label => ColumnType::String(None).def().null(),
            Self::Invoice => ColumnType::String(None).def().null(),
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
