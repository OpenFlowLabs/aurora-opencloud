//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "tenant")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub parent: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::Parent",
        to = "Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    SelfRef,
}

impl Related<super::principal::Entity> for Entity {
    fn to() -> RelationDef {
        super::user_confirmations::Relation::Principal.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::user_confirmations::Relation::Tenant.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}