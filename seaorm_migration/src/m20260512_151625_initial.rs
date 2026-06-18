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
                    .table("savefiles")
                    .if_not_exists()
                    .col(pk_uuid("id").default(Expr::cust("uuidv7()")))
                    .col(big_integer("games_played").default(Expr::value(0)))
                    .col(big_integer("points").default(Expr::value(0)))
                    .col(big_integer("cards_had").default(Expr::value(0)))
                    .col(integer("wins").default(Expr::value(0)))
                    .col(integer("loses").default(Expr::value(0)))
                    .col(big_integer("max_points").default(Expr::value(0)))
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
                    .table("game_histories")
                    .if_not_exists()
                    .col(pk_uuid("id").default(Expr::cust("uuidv7()")))
                    .col(uuid("account_id"))
                    .col(uuid("game_id"))
                    .col(integer("placement"))
                    .col(big_integer("points").default(Expr::value(0)))
                    .col(big_integer("cards_had").default(Expr::value(0)))
                    .col(string("participants").string_len(1024))
                    .col(timestamp_with_time_zone("finished_at").default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_game_histories_account_id")
                            .from("game_histories", "account_id")
                            .to("accounts", "id")
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_game_histories_account_id")
                    .table(Alias::new("game_histories"))
                    .col(Alias::new("account_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("chat_messages")
                    .if_not_exists()
                    .col(pk_uuid("id").default(Expr::cust("uuidv7()")))
                    .col(string("channel_id").string_len(36))
                    .col(uuid("author_id"))
                    .col(string("content").string_len(2000))
                    .col(timestamp_with_time_zone("posted_at").default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone_null("edited_at"))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_chat_messages_author_id")
                            .from("chat_messages", "author_id")
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
                    .table("muted")
                    .if_not_exists()
                    .col(pk_uuid("id").default(Expr::cust("uuidv7()")))
                    .col(uuid("blocker_id"))
                    .col(uuid("blocked_id"))
                    .col(timestamp_with_time_zone("blocked_at").default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_muted_blocker_id")
                            .from("muted", "blocker_id")
                            .to("accounts", "id")
                            .on_update(ForeignKeyAction::Restrict)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_muted_blocked_id")
                            .from("muted", "blocked_id")
                            .to("accounts", "id")
                            .on_update(ForeignKeyAction::Restrict)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_muted_blocker_id")
                    .table(Alias::new("muted"))
                    .col(Alias::new("blocker_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_muted_blocked_id")
                    .table(Alias::new("muted"))
                    .col(Alias::new("blocked_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uniq_muted_blocker_blocked")
                    .table(Alias::new("muted"))
                    .col(Alias::new("blocker_id"))
                    .col(Alias::new("blocked_id"))
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_chat_messages_channel_id")
                    .table(Alias::new("chat_messages"))
                    .col(Alias::new("channel_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_chat_messages_posted_at")
                    .table(Alias::new("chat_messages"))
                    .col(Alias::new("posted_at"))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table("chat_messages")
                    .name("fk_chat_messages_author_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table("muted")
                    .name("fk_muted_blocker_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table("muted")
                    .name("fk_muted_blocked_id")
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
                    .table("game_histories")
                    .name("fk_game_histories_account_id")
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
            .drop_table(Table::drop().table("chat_messages").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("muted").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("sessions").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("game_histories").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("savefiles").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("accounts").to_owned())
            .await
    }
}
