use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("accounts")
                    .if_not_exists()
                    .col(pk_uuid("id").default(Expr::cust("uuidv7()")))
                    .col(string("login").unique_key().string_len(32))
                    .col(string("password").string_len(255))
                    .col(string_null("display_name").string_len(32))
                    .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("roles")
                    .if_not_exists()
                    .col(pk_uuid("id").default(Expr::cust("uuidv7()")))
                    .col(string("name"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("savefiles")
                    .if_not_exists()
                    .col(pk_uuid("id").default(Expr::cust("uuidv7()")))
                    .col(big_integer("games_played").default(Expr::value(0)))
                    .col(big_integer("points").default(Expr::value(0)))
                    .col(big_integer("cards_had").default(Expr::value(0)))
                    .col(integer("wins").default(Expr::value(0)))
                    .col(integer("loses").default(Expr::value(0)))
                    .col(integer("max_points").default(Expr::value(0)))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_savefiles_account_id")
                            .from("savefiles", "id")
                            .to("accounts", "id")
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("sessions")
                    .if_not_exists()
                    .col(pk_uuid("id").default(Expr::cust("uuidv7()")))
                    .col(uuid("account_id"))
                    .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone("expires_at"))
                    .col(boolean("is_revoked").default(Expr::value(false)))
                    .col(string("token").string_len(255))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sessions_account_id")
                            .from("sessions", "account_id")
                            .to("accounts", "id")
                            .on_update(ForeignKeyAction::Restrict)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("roles_assigned")
                    .if_not_exists()
                    .col(pk_uuid("id").default(Expr::cust("uuidv7()")))
                    .col(uuid("account_id"))
                    .col(uuid("role_id"))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_roles_assigned_account_id")
                            .from("roles_assigned", "account_id")
                            .to("accounts", "id")
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_roles_assigned_role_id")
                            .from("roles_assigned", "role_id")
                            .to("roles", "id")
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table("roles_assigned")
                    .name("fk_roles_assigned_role_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table("roles_assigned")
                    .name("fk_roles_assigned_account_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table("sessions")
                    .name("fk_sessions_account_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table("savefiles")
                    .name("fk_savefiles_account_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table("roles_assigned").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("sessions").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("savefiles").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("roles").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("accounts").to_owned())
            .await
    }
}
