use ::entity::userinfo;
use ::entity::userinfo::{Entity, Model};
use sea_orm::ActiveValue::Set;
use sea_orm::DbErr;
use sea_orm::RuntimeErr::Internal;
use sea_orm::*;

pub struct Dao {}

impl Dao {
    // find all userinfo
    pub async fn find_all(db: &DbConn) -> Result<Vec<Model>, DbErr> {
        Entity::find().all(db).await
    }

    // find page in userinfo
    pub async fn find_in_page(
        db: &DbConn,
        page: u64,
        page_size: u64,
    ) -> Result<(Vec<Model>, u64), DbErr> {
        let paginator = Entity::find()
            .order_by_asc(userinfo::Column::Id)
            .paginate(db, page_size);

        let num_pages = paginator.num_pages().await?;

        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }

    // find by name
    pub async fn find_by_name(db: &DbConn, name: String) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(userinfo::Column::Name.eq(name))
            .one(db)
            .await
    }

    // find by name and password
    pub async fn find_by_name_and_pwd(
        db: &DbConn,
        name: String,
        pwd: String,
    ) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(
                userinfo::Column::Name
                    .eq(name)
                    .and(userinfo::Column::Pwd.eq(pwd)),
            )
            .one(db)
            .await
    }

    // find like name
    pub async fn find_like_name(db: &DbConn, name: &String) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(userinfo::Column::Name.contains(name.as_str()))
            .all(db)
            .await
    }

    // update by id
    pub async fn update_by_id(db: &DbConn, id: i32, param: Model) -> Result<Model, DbErr> {
        let data: userinfo::ActiveModel = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom(format!("cannot find userInfo ,id:{id}")))
            .map(Into::into)?;

        userinfo::ActiveModel {
            id: data.id,
            name: Set(param.name.to_owned()),
            pwd: Set(param.pwd.to_owned()),
        }
        .update(db)
        .await
    }

    pub async fn insert(db: &DbConn, param: Model) -> Result<Model, DbErr> {
        let exist = Dao::find_like_name(db, &param.name).await;
        if exist.is_ok() && exist.unwrap().len() > 0 {
            return Err(DbErr::Custom(format!(
                "name already exist ,name:{}",
                param.name
            )));
        }
        userinfo::ActiveModel::from(param).insert(db).await
    }
}
