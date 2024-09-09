use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Info {
    Table,
    Id,
    UserId,
    ChatId,
    OmsCard,
    DateBirth,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                sea_query::Table::create()
                    .table(Info::Table)
                    .if_not_exists()
                    .col(pk_auto(Info::Id))
                    .col(integer(Info::UserId))
                    .col(integer(Info::ChatId))
                    .col(integer_null(Info::OmsCard))
                    .col(date_null(Info::DateBirth))
                    .to_owned()
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table(Info::Table).to_owned())
            .await
    }
}
