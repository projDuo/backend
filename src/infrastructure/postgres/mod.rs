pub mod entities;
pub mod accounts;
pub mod savefiles;
pub mod sessions;

use sea_orm::{Database, DatabaseConnection, DbErr};

use crate::domain::InternalError;

#[derive(Clone)]
pub struct Postgres {
    db: DatabaseConnection
}

impl Postgres {
    pub async fn new(path: &str) -> Result<Self, DbErr> {
        let connection = Database::connect(path).await?;
        Ok(Self{ db: connection })
    }

    pub async fn from_details(details: PostgresDetails) -> Result<Self, DbErr> {
        Self::new(details.path().as_str()).await
    } 
}

struct URI(String);

impl URI {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

pub struct PostgresDetails {
    host: String,
    name: String,
    port: u16,
    user: String,
    pass: String,
}

impl PostgresDetails {
    pub fn new(
        host: &str,
        name: &str,
        port: u16,
        user: &str,
        pass: &str,
    ) -> Self {
        Self { 
            host: host.to_string(),
            name: name.to_string(),
            port,
            user: user.to_string(),
            pass: pass.to_string()
        }
    }

    pub fn path(&self) -> URI {
        let uri = format!(
                "postgres://{}:{}@{}:{}/{}",
                self.user, self.pass, self.host, self.port, self.name
        );

        URI(uri)
    }
}

impl From<DbErr> for InternalError {
    fn from(value: DbErr) -> Self {
        Self(value.to_string())
    }
}