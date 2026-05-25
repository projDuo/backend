use serde::Serialize;

use super::super::{
    account::output::AccountReadPublic, 
    activity::output::Activity,
};

#[derive(Debug, Clone, Serialize)]
pub struct Authorized { //Структура, що описує подію привітання
    pub heartbeat_interval: u64, 
    pub account: AccountReadPublic, 
    pub session: Option<Activity>,
}
impl Authorized {
    pub fn new(
        heartbeat_interval: u64, 
        account: AccountReadPublic, 
        session: Option<Activity>,
    ) -> Self {
        Self {
            heartbeat_interval,
            account,
            session
        }
    }
}