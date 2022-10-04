use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};

use crate::seconds_since_epoch;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "i16", db_type = "SmallInteger")]
pub enum NodeStatus {
    #[sea_orm(num_value = 0)]
    Stopped,
    #[sea_orm(num_value = 1)]
    Running,
}

impl From<NodeStatus> for i16 {
    fn from(status: NodeStatus) -> i16 {
        match status {
            NodeStatus::Stopped => 0,
            NodeStatus::Running => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "i16", db_type = "SmallInteger")]
pub enum NodeRole {
    #[sea_orm(num_value = 0)]
    Default,
}

impl From<NodeRole> for i16 {
    fn from(role: NodeRole) -> i16 {
        match role {
            NodeRole::Default => 0,
        }
    }
}

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "node"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub role: i16,
    pub username: String,
    pub alias: String,
    pub network: String,
    pub listen_addr: String,
    pub listen_port: i32,
    pub created_at: i64,
    pub updated_at: i64,
    pub status: i16,
}

impl Model {
    pub fn get_role(&self) -> NodeRole {
        match self.role {
            0 => NodeRole::Default,
            _ => panic!("invalid role"),
        }
    }

    pub fn get_status(&self) -> NodeStatus {
        match self.status {
            0 => NodeStatus::Stopped,
            1 => NodeStatus::Running,
            _ => panic!("invalid status"),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    Role,
    Username,
    Alias,
    Network,
    ListenAddr,
    ListenPort,
    CreatedAt,
    UpdatedAt,
    Status,
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
            Self::Role => ColumnType::SmallInteger.def(),
            Self::Username => ColumnType::String(None).def().unique(),
            Self::Alias => ColumnType::String(None).def(),
            Self::Network => ColumnType::String(None).def(),
            Self::ListenAddr => ColumnType::String(None).def(),
            Self::ListenPort => ColumnType::Integer.def(),
            Self::CreatedAt => ColumnType::BigInteger.def(),
            Self::UpdatedAt => ColumnType::BigInteger.def(),
            Self::Status => ColumnType::SmallInteger.def(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No RelationDef")
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn before_save(mut self, insert: bool) -> Result<Self, DbErr> {
        let now: i64 = seconds_since_epoch();
        self.updated_at = ActiveValue::Set(now);
        if insert {
            self.created_at = ActiveValue::Set(now);
        }
        Ok(self)
    }
}
