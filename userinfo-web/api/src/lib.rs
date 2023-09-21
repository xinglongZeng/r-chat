use actix_files::Files as Fs;
use actix_files::NamedFile;
use actix_http::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::{
    error, get, middleware, post, web, App, Error, HttpRequest, HttpResponse, HttpServer,
    Responder, Result,
};
use common::config::TcpSocketConfig;
use derive_more::Display;
use entity::userinfo;
use entity::userinfo::Model;
use env_logger::Env;
use listenfd::ListenFd;
use log::debug;
use serde::{Deserialize, Serialize};
use service::sea_orm::{Database, DbErr};
use service::userinfo_service::Service;
use std::env;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use tera::Tera;

const PAGE_SIZE: u64 = 5;

#[derive(Debug, Clone)]
struct AppState {
    templates: Tera,
    // conn: Arc<DatabaseConnection>,
    user_service: Arc<Service>,
}

#[derive(Debug, Deserialize)]
pub struct PageParams {
    page_index: Option<u64>,
    page_size: Option<u64>,
}

pub struct ServerEnvConfig {
    pub socket_config: TcpSocketConfig,
    web_host: String,
    web_port: String,
    database_url: String,
}

impl ServerEnvConfig {
    fn new() -> Self {
        dotenvy::dotenv().ok();

        let tcp_host = env::var("TCP_HOST").expect("TCP_HOST is not set in .env file");
        let tcp_port = env::var("TCP_PORT").expect("TCP_PORT is not set in .env file");
        let web_host = env::var("WEB_HOST").expect("WEB_HOST is not set in .env file");
        let web_port = env::var("WEB_PORT").expect("WEB_PORT is not set in .env file");
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");

        let socket_config = TcpSocketConfig { tcp_host, tcp_port };

        ServerEnvConfig {
            socket_config,
            web_host,
            web_port,
            database_url,
        }
    }
}

#[get("/")]
async fn user_index(req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let template = &data.templates;

    // get page params from httpRequest. 从HttpRequest中获取分页参数
    let params = web::Query::<PageParams>::from_query(req.query_string()).unwrap();

    // get page index. 获取page index ，如果不存在则设置为1
    let page = params.page_index.unwrap_or(1);

    // get page size.
    let page_size = params.page_size.unwrap_or(PAGE_SIZE);

    // invoke service to query data . 调用service来查询分页数据
    let (page_data, num_page) = data
        .user_service
        .dao
        .find_in_page(page, page_size)
        .await
        .expect("Cannot find user_index in page");

    // send page_data to html. 将分页数据传入html页面中
    let mut ctx = tera::Context::new();
    // 分页的数据
    ctx.insert("page_data", &page_data);
    // 要查询的分页的index
    ctx.insert("page_index", &page);
    // 页面size
    ctx.insert("page_size", &page_size);
    // 查询出的分页页数
    ctx.insert("num_page", &num_page);

    let body = template
        .render("index.html.tera", &ctx)
        .map_err(|m| error::ErrorInternalServerError(m))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[get("/new")]
async fn new(data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let template = &data.templates;

    // send page_data to html. 将分页数据传入html页面中
    let ctx = tera::Context::new();

    let body = template
        .render("new.html.tera", &ctx)
        .map_err(|m| error::ErrorInternalServerError(m))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

async fn not_found(data: web::Data<AppState>, request: HttpRequest) -> Result<HttpResponse, Error> {
    let mut ctx = tera::Context::new();
    ctx.insert("uri", request.uri().path());

    let template = &data.templates;
    let body = template
        .render("error/404.html.tera", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[derive(Debug, Display, derive_more::Error)]
pub enum MyError {
    #[display(fmt = "internal error")]
    InternalError,

    #[display(fmt = "bad request")]
    BadClientData,

    #[display(fmt = "timeout")]
    Timeout,

    #[display(fmt = "Validation error on field: {}", field)]
    ValidationError { field: String },
}

impl error::ResponseError for MyError {
    fn status_code(&self) -> StatusCode {
        match *self {
            MyError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            MyError::BadClientData => StatusCode::BAD_REQUEST,
            MyError::Timeout => StatusCode::GATEWAY_TIMEOUT,
            MyError::ValidationError { .. } => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserInfoVo {
    id: Option<i32>,
    name: String,
    pwd: String,
}

impl From<Model> for UserInfoVo {
    fn from(m: Model) -> Self {
        UserInfoVo {
            id: Some(m.id),
            name: m.name,
            pwd: m.pwd,
        }
    }
}

impl From<UserInfoVo> for Model {
    fn from(value: UserInfoVo) -> Self {
        Model {
            id: if value.id.is_some() {
                value.id.unwrap()
            } else {
                0
            },
            name: value.name,
            pwd: value.pwd,
        }
    }
}

impl Responder for UserInfoVo {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();
        // Create response and set content type
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

// registry account
#[post("/insert")]
async fn registry_account(
    param: web::Form<UserInfoVo>,
    data: web::Data<AppState>,
) -> Result<UserInfoVo, MyError> {
    debug!("invoke registry_account data:{:?} ", param);

    let model = userinfo::Model::from(param.0);

    // invoke service to query data . 调用service来查询分页数据
    let result = data.user_service.dao.insert(model).await;
    match result {
        Ok(t) => Ok(UserInfoVo::from(t)),
        Err(e) => Err(MyError::ValidationError {
            field: e.to_string(),
        }),
    }
}

#[get("/account_index")]
async fn account_index() -> impl Responder {
    NamedFile::open_async("static/index.html").await
}

fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(user_index);
    cfg.service(registry_account);
    cfg.service(account_index);
    cfg.service(new);
}

#[actix_web::main]
pub async fn api_start_web_server_new(user_service: Arc<Service>) -> std::io::Result<()> {
    // set logger level to debug
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    // set config of env . 设置环境变量
    env::set_var("RUST_LOG", "debug");

    // start trace info collect.  开启堆栈信息收集
    tracing_subscriber::fmt::init();

    // get env vars   读取.env文件中的变量，相当于读取配置文件
    dotenvy::dotenv().ok();

    let host = env::var("WEB_HOST").expect("HOST is not set in .env file");
    let port = env::var("WEB_PORT").expect("PORT is not set in .env file");
    let server_url = format!("{host}:{port}");

    // load tera templates
    let templates = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();

    // build app state. 构建app的state，以便各个线程共享AppState
    let state = AppState {
        templates,
        user_service,
    };

    // create server
    let mut server = HttpServer::new(move || {
        App::new()
            // mount dir static
            .service(Fs::new("static", "./userinfo-web/api/static"))
            // app_data could share state for each thread
            .app_data(web::Data::new(state.clone()))
            .wrap(middleware::Logger::default())
            .default_service(web::route().to(not_found))
            .configure(init)
    });

    let mut listenfd = ListenFd::from_env();

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => server.bind(&server_url)?,
    };

    server.run().await
}
