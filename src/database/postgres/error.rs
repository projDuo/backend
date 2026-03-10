use super::RepositoryError;
use sea_orm::{DbErr, RuntimeErr, sqlx::{self, postgres::PgDatabaseError}};

impl From<DbErr> for RepositoryError {
    fn from(err: DbErr) -> Self {
        match err {
            DbErr::RecordNotFound(msg) => RepositoryError::NotFound(msg),

            DbErr::Query(RuntimeErr::SqlxError(sqlx::Error::Database(ref db_err)))
            | DbErr::Exec(RuntimeErr::SqlxError(sqlx::Error::Database(ref db_err))) => {
                if let Some(db_err) = db_err.try_downcast_ref::<PgDatabaseError>() {
                    match db_err.code() {
                        "23505" => {
                            let constraint = db_err.constraint().unwrap_or("unknown_constraint");
                            RepositoryError::UniqueViolation(constraint.to_string())
                        }
                        "23503" => {
                                let constraint = db_err.constraint().unwrap_or("unknown_constraint");
                                RepositoryError::ForeignKeyViolation(constraint.to_string())
                        }
                        "23502" => {
                            let column = db_err.column().unwrap_or("unknown_field");
                            RepositoryError::NotNullViolation(column.to_string())
                        }
                        _ => RepositoryError::Internal(db_err.message().to_string()),
                    }
                } else {
                    RepositoryError::Internal(db_err.message().to_string())
                }
            }

            DbErr::ConnectionAcquire(e) => RepositoryError::ConnectionError(e.to_string()),

            DbErr::RecordNotInserted => RepositoryError::Internal("Record could not be inserted".to_string()),
            DbErr::RecordNotUpdated => RepositoryError::Internal("Record could not be updated".to_string()),

            _ => RepositoryError::Internal(err.to_string()),
        }
    }
}