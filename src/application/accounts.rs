
use crate::domain::accounts::*;
use async_trait::async_trait;
use secrecy::SecretString;
use uuid::Uuid;
use crate::domain::InternalError;

#[derive(Clone)]
pub struct Service<R: AccountsRepository> {
    repo: R,
}

impl<R> Service<R>
where R: AccountsRepository {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<R> AccountsService for Service<R>
where R: AccountsRepository + Send + Sync {
    async fn read_account(&self, id: Uuid) -> Result<Account, AccountError> {
        self.repo.find_by_id(id).await?
            .ok_or(AccountError::NotFound)
    }

    async fn read_account_by_login(&self, login: String) -> Result<Account, AccountError> {
        self.repo.find_by_login(login).await?
            .ok_or(AccountError::NotFound)
    }

    async fn read_account_by_id_or_login(&self, id: &str) -> Result<Account, AccountError> {
        if let Ok(v) = uuid::Uuid::try_parse(id) {
            self.read_account(v).await
        } else {
            self.read_account_by_login(id.to_string()).await
        }
    }

    async fn update_account(&self, cmd: UpdateAccountRequest) -> Result<Account, AccountError> {
        self.repo.update_account(cmd).await
    }

    async fn register(&self, cmd: CreateAccountRequest) -> Result<Account, AccountError> {
        self.repo.insert_account(cmd).await
    }

    async fn login(&self, login: String, password: SecretString) -> Result<Account, AccountError> {
        let account = self.repo.find_by_login(login).await?
            .ok_or(AccountError::InvalidCredentials)?;

        if !account.password.verify(password).await { 
            return Err(AccountError::InvalidCredentials);
        };

        Ok(account)
    }
    
    async fn is_login_taken(&self, login: String) -> Result<bool, InternalError> {
        let account = self.repo.find_by_login(login).await?;
        Ok(account.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::SecretString;
    use tokio::sync::Mutex;
    use std::collections::HashMap;

    struct DummyRepo {
        store: Mutex<HashMap<String, (Uuid, String, String)>>,
    }

    impl DummyRepo {
        fn new() -> Self { Self { store: Mutex::new(HashMap::new()) } }
    }

    #[async_trait]
    impl AccountsRepository for DummyRepo {
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Account>, InternalError> {
            let store = self.store.lock().await;
            for (_k, (uid, login, hash)) in store.iter() {
                if *uid == id {
                    let login_vo = crate::domain::accounts::value_objects::Login::from_raw(login.clone());
                    let pw = crate::domain::accounts::value_objects::Password::from_hash(hash.clone());
                    return Ok(Some(Account::new(*uid, login_vo, pw, None, chrono::Utc::now().into())));
                }
            }
            Ok(None)
        }

        async fn find_by_login(&self, login: String) -> Result<Option<Account>, InternalError> {
            let store = self.store.lock().await;
            if let Some((uid, _l, hash)) = store.get(&login) {
                let login_vo = crate::domain::accounts::value_objects::Login::from_raw(login.clone());
                let pw = crate::domain::accounts::value_objects::Password::from_hash(hash.clone());
                return Ok(Some(Account::new(*uid, login_vo, pw, None, chrono::Utc::now().into())));
            }
            Ok(None)
        }

        async fn insert_account(&self, cmd: CreateAccountRequest) -> Result<Account, AccountError> {
            let mut store = self.store.lock().await;
            if store.contains_key(&cmd.login.to_string()) {
                return Err(AccountError::Duplicate);
            }
            let id = Uuid::now_v7();
            store.insert(cmd.login.to_string(), (id, cmd.login.to_string(), cmd.password.to_string()));
            Ok(Account::new(id, cmd.login, cmd.password, None, chrono::Utc::now().into()))
        }

        async fn update_account(&self, _cmd: UpdateAccountRequest) -> Result<Account, AccountError> {
            Err(AccountError::NotFound)
        }

        async fn delete_account(&self, _id: Uuid) -> Result<(), AccountError> { Ok(()) }

        async fn find_many_accounts_by_id_public(&self, _ids: Vec<Uuid>) -> Result<Vec<crate::domain::accounts::query::AccountPublic>, InternalError> { Ok(vec![]) }
    }

    #[tokio::test]
    async fn register_and_login() {
        let repo = DummyRepo::new();
        let svc = Service::new(repo);

        let password = crate::domain::accounts::value_objects::Password::new(SecretString::new("Passw0rd!".to_string().into())).await.unwrap();

        let account = svc.register(CreateAccountRequest { login: crate::domain::accounts::value_objects::Login::from_raw("alice".to_string()), password: password.clone() }).await.expect("register");
        assert_eq!(account.login().to_string(), "alice");

        let logged = svc.login("alice".to_string(), SecretString::new("Passw0rd!".to_string().into())).await.expect("login");
        assert_eq!(*logged.id(), *account.id());
    }
}