use crate::userinfo_dao;
use ::entity::userinfo;
use sea_orm::DbConn;
use std::sync::Arc;
use common::LoginReqData;

pub struct Service {
    pub db: Arc<DbConn>,
}

impl Service {
    pub async fn find_by_account_and_pwd(
        &self,
        param: &LoginReqData,
    ) -> Result<Option<userinfo::Model>, String> {
        let result = userinfo_dao::Dao::find_by_name_and_pwd(
            self.db.as_ref(),
            param.account.clone(),
            param.pwd.clone(),
        )
        .await;
        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(e.to_string()),
        }
    }
}
