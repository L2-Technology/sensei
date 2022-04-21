use bdk::bitcoin::consensus::deserialize;
use sea_orm::{entity::prelude::*, ActiveValue};

use crate::{seconds_since_epoch, to_vec_unsafe};

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "utxo"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    pub id: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub node_id: String,
    pub value: i64,
    pub keychain: String,
    pub vout: i32,
    pub txid: String,
    pub script: String,
    pub is_spent: bool,
}

impl Model {
    pub fn to_local_utxo(&self) -> Result<bdk::LocalUtxo, bdk::Error> {
        Ok(bdk::LocalUtxo {
            outpoint: bdk::bitcoin::OutPoint {
                txid: deserialize(&to_vec_unsafe(&self.txid))?,
                vout: self.vout as u32,
            },
            txout: bdk::bitcoin::TxOut {
                value: self.value as u64,
                script_pubkey: deserialize(&to_vec_unsafe(&self.script))?,
            },
            keychain: serde_json::from_str(&self.keychain)?,
            is_spent: self.is_spent,
        })
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    CreatedAt,
    UpdatedAt,
    NodeId,
    Value,
    Keychain,
    Vout,
    Txid,
    Script,
    IsSpent,
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
            Self::Value => ColumnType::BigInteger.def(),
            Self::Keychain => ColumnType::String(None).def(),
            Self::Vout => ColumnType::Integer.def(),
            Self::Txid => ColumnType::String(None).def(),
            Self::Script => ColumnType::String(None).def(),
            Self::IsSpent => ColumnType::Boolean.def(),
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
