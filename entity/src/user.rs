use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};

use crate::seconds_since_epoch;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "i16", db_type = "SmallInteger")]
pub enum UserRole {
    #[sea_orm(num_value = 0)]
    Default,
}

impl From<UserRole> for i16 {
    fn from(role: UserRole) -> i16 {
        match role {
            UserRole::Default => 0,
        }
    }
}

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "user"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub role: i16,
    pub username: String,
    pub hashed_password: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Model {
    pub fn get_role(&self) -> UserRole {
        match self.role {
            0 => UserRole::Default,
            _ => panic!("invalid role"),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    Role,
    Username,
    HashedPassword,
    CreatedAt,
    UpdatedAt,
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
            Self::HashedPassword => ColumnType::String(None).def(),
            Self::CreatedAt => ColumnType::BigInteger.def(),
            Self::UpdatedAt => ColumnType::BigInteger.def(),
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
            role: ActiveValue::Set(UserRole::Default.into()),
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
