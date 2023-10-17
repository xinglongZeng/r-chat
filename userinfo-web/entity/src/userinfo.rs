use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "user_info")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    #[serde(skip_deserializing)]
    pub id: i32,
    pub name: String,
    pub pwd: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
