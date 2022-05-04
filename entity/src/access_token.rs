use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};

use crate::seconds_since_epoch;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "access_token"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Deserialize, Serialize)]
pub struct Model {
    pub id: String,
    pub token: String,
    pub name: String,
    pub scope: String,
    pub single_use: bool,
    pub expires_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Model {
    pub fn is_expired(&self) -> bool {
        self.expires_at != 0 && seconds_since_epoch() > self.expires_at
    }

    pub fn has_access_to_scope(&self, scope: Option<&str>) -> bool {
        match scope {
            Some(scope) => {
                let scopes: Vec<&str> = self.scope.split(',').collect();
                scopes.contains(&"*") || scopes.contains(&scope)
            }
            None => true,
        }
    }

    pub fn is_valid(&self, scope: Option<&str>) -> bool {
        !self.is_expired() && self.has_access_to_scope(scope)
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    Token,
    Name,
    Scope,
    SingleUse,
    ExpiresAt,
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
            Self::CreatedAt => ColumnType::BigInteger.def(),
            Self::UpdatedAt => ColumnType::BigInteger.def(),
            Self::Token => ColumnType::String(None).def().unique(),
            Self::Name => ColumnType::String(None).def(),
            Self::Scope => ColumnType::String(None).def(),
            Self::SingleUse => ColumnType::Boolean.def(),
            Self::ExpiresAt => ColumnType::BigInteger.def(),
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
