use crate::userinfo_dao::Dao;
use ::entity::userinfo;
use common::LoginReqData;

#[derive(Debug)]
pub struct Service {
    pub dao: Dao,
}

impl Service {
    pub async fn find_by_account_and_pwd(
        &self,
        param: &LoginReqData,
    ) -> Result<Option<userinfo::Model>, String> {
        let result = self
            .dao
            .find_by_name_and_pwd(param.account.clone(), param.account.clone())
            .await;

        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(e.to_string()),
        }
    }
}
