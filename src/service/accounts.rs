use crate::domain;

pub type Accounts<R> = super::Service<domain::accounts::Account, R>;