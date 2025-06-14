//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.0

use sea_orm::entity::prelude::*;

use crate::{
    db::parse_datetime_str,
    model::transactions::content_description::{
        ContentDescriptionUuid, ModelContentDescription,
    },
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "content_description")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub uuid: ContentDescriptionUuid,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    pub datetime_created: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::profile_content_descriptions::Entity")]
    ProfileContentDescriptions,
    #[sea_orm(has_many = "super::special_content::Entity")]
    SpecialContent,
    #[sea_orm(has_many = "super::text_content::Entity")]
    TextContent,
}

impl Related<super::profile_content_descriptions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProfileContentDescriptions.def()
    }
}

impl Related<super::special_content::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SpecialContent.def()
    }
}

impl Related<super::text_content::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TextContent.def()
    }
}

impl Related<super::profile::Entity> for Entity {
    fn to() -> RelationDef {
        super::profile_content_descriptions::Relation::Profile.def()
    }
    fn via() -> Option<RelationDef> {
        Some(
            super::profile_content_descriptions::Relation::ContentDescription
                .def()
                .rev(),
        )
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for ModelContentDescription {
    fn from(
        Model {
            uuid,
            description,
            datetime_created,
        }: Model,
    ) -> Self {
        ModelContentDescription {
            uuid,
            description,
            datetime_created: parse_datetime_str(&datetime_created),
        }
    }
}
