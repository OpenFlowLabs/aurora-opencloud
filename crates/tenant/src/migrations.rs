use sea_orm_migration::prelude::*;
use tonic::async_trait;

pub use sea_orm_migration::MigratorTrait;

pub struct Migrator;

impl MigratorTrait for Migrator {
    #[doc = " Vector of migrations in time sequence"]
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(InitMigration)]
    }
}

#[derive(DeriveMigrationName)]
pub struct InitMigration;

#[async_trait]
impl MigrationTrait for InitMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tenant::Table)
                    .col(ColumnDef::new(Tenant::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Tenant::Name).string().not_null())
                    .col(ColumnDef::new(Tenant::Parent).uuid().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk_tenant_self")
                    .from(Tenant::Table, Tenant::Parent)
                    .to(Tenant::Table, Tenant::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Principal::Table)
                    .col(
                        ColumnDef::new(Principal::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Principal::Tenant).not_null().uuid())
                    .col(
                        ColumnDef::new(Principal::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Principal::Email)
                            .string()
                            .null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Principal::EmailConfirmed)
                            .boolean()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PublicKey::Table)
                    .col(
                        ColumnDef::new(PublicKey::Fingerprint)
                            .string()
                            .primary_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PublicKey::Key).not_null().string())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .primary_key(
                        Index::create()
                            .primary()
                            .col(PrincipalsPublicKeys::Principal)
                            .col(PrincipalsPublicKeys::Fingerprint),
                    )
                    .table(PrincipalsPublicKeys::Table)
                    .col(
                        ColumnDef::new(PrincipalsPublicKeys::Principal)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PrincipalsPublicKeys::Fingerprint)
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .from(PrincipalsPublicKeys::Table, PrincipalsPublicKeys::Principal)
                    .to(Principal::Table, Principal::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .from(
                        PrincipalsPublicKeys::Table,
                        PrincipalsPublicKeys::Fingerprint,
                    )
                    .to(PublicKey::Table, PublicKey::Fingerprint)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(UserConfirmations::Table)
                    .col(ColumnDef::new(UserConfirmations::Tenant).not_null().uuid())
                    .col(
                        ColumnDef::new(UserConfirmations::Token)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(UserConfirmations::Email).string().not_null())
                    .primary_key(
                        Index::create()
                            .col(UserConfirmations::Token)
                            .col(UserConfirmations::Email),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_confirmation_tenant")
                    .from(UserConfirmations::Table, UserConfirmations::Tenant)
                    .to(Tenant::Table, Tenant::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_confirmation_principal")
                    .from(UserConfirmations::Table, UserConfirmations::Email)
                    .to(Principal::Table, Principal::Email)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Principal::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Tenant::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Principal {
    Table,
    Id,
    Tenant,
    Name,
    Email,
    EmailConfirmed,
}

#[derive(Iden)]
enum PublicKey {
    Table,
    Fingerprint,
    Key,
}

#[derive(Iden)]
enum Tenant {
    Table,
    Id,
    Name,
    Parent,
}

#[derive(Iden)]
enum PrincipalsPublicKeys {
    Table,
    Principal,
    Fingerprint,
}

#[derive(Iden)]
enum UserConfirmations {
    Table,
    Tenant,
    Token,
    Email,
}
