use sea_orm::{entity::prelude::*, ActiveValue};

use crate::seconds_since_epoch;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "script_pubkey"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    pub id: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub node_id: String,
    pub keychain: String,
    pub child: i32,
    pub script: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    CreatedAt,
    UpdatedAt,
    NodeId,
    Keychain,
    Child,
    Script,
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
            Self::Keychain => ColumnType::String(None).def(),
            Self::Child => ColumnType::BigInteger.def(),
            Self::Script => ColumnType::String(None).def(),
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
