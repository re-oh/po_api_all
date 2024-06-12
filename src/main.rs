use actix_web::{
    delete, get, post,
    web::{self, get, Json},
    App, HttpResponse, HttpServer, Responder,
};
use content_manager::{ContentManager, Id, IntermediatePost, Post};
use database_manager::DatabaseManager;
use std::sync::Mutex;
mod content_manager;
mod database_manager;

struct AppState {
    content_manager: Mutex<ContentManager>,
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/posts/{post_id}")]
async fn get_post(data: web::Data<AppState>, path: web::Path<i32>) -> impl Responder {
    let mut content_manager = data.content_manager.lock().unwrap();
    let post_id = Id::PostId(path.into_inner());

    if let Some(post) = content_manager.get_post(post_id.clone()) {
        return HttpResponse::Ok().json(post);
    } else {
        return HttpResponse::NoContent().await.unwrap();
    };
}

#[get("/posts")]
async fn get_posts(data: web::Data<AppState>) -> impl Responder {
    let mut content_manager = data.content_manager.lock().unwrap();
    if let Some(posts) = content_manager.get_posts() {
        HttpResponse::Ok().json(posts)
    } else {
        HttpResponse::NoContent().await.unwrap()
    }
}

#[post("/posts")]
async fn create_post(
    data: web::Data<AppState>,
    post: web::Json<IntermediatePost>,
) -> impl Responder {
    let mut content_manager = data.content_manager.lock().unwrap();
    let inter_post = post.clone();

    if let Some(post) = content_manager.create_post(inter_post) {
        HttpResponse::Ok().json(post)
    } else {
        HttpResponse::BadRequest().await.unwrap()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone, PartialOrd)]
struct DeletePostInfo {
    post_id: i32,
    account_id: i32,
}

#[delete("/posts")]
async fn delete_post(
    data: web::Data<AppState>,
    context: web::Json<DeletePostInfo>,
) -> impl Responder {
    let mut content_manager = data.content_manager.lock().unwrap();
    let context = context.clone();

    if let Some(err) = content_manager.delete_post(
        Id::PostId(context.post_id),
        Id::AccountId(context.account_id),
    ) {
        HttpResponse::BadRequest().await.unwrap()
    } else {
        HttpResponse::Ok().await.unwrap()
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(AppState {
                content_manager: Mutex::new(ContentManager::new(DatabaseManager::connect(
                    "db.sqlite",
                ))),
            }))
            .service(health)
            .service(get_post)
            .service(get_posts)
            .service(create_post)
            .service(delete_post)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
