use actix_web::{http::header, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use env_logger::Env;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::env;

const CHAR_SET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

struct AppState {
    pool: SqlitePool,
    convert_url: String,
}
#[derive(Serialize, Deserialize)]
struct CreateShortenPayload {
    url: String,
}

async fn base() -> impl Responder {
    HttpResponse::Ok().body("Shorten service!")
}

async fn short_url(path: web::Path<(String,)>, data: web::Data<AppState>) -> impl Responder {
    let payload = path.into_inner();
    let result = sqlx::query!("SELECT url FROM Urls WHERE id = ?", payload.0)
        .fetch_one(&data.pool)
        .await;
    match result {
        Ok(result) => match result.url {
            Some(url) => HttpResponse::TemporaryRedirect()
                .append_header((header::LOCATION, url))
                .finish(),
            None => HttpResponse::NotFound().body("Not found"),
        },
        Err(_) => HttpResponse::NotFound().body("Notfound"),
    }
}

async fn create_shorten(
    payload: web::Json<CreateShortenPayload>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut rng = rand::thread_rng();
    let id_content: String = (0..6)
        .map(|_| {
            let idx = rng.gen_range(0..CHAR_SET.len());
            CHAR_SET[idx] as char
        })
        .collect();
    sqlx::query!("INSERT INTO Urls VALUES (?, ?)", id_content, payload.url)
        .execute(&data.pool)
        .await
        .unwrap();
    HttpResponse::Ok().body(format!("{}/{}", data.convert_url, id_content))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = SqlitePool::connect(&format!("{}?mode=rwc", database_url))
        .await
        .unwrap();
    sqlx::query!("CREATE TABLE IF NOT EXISTS Urls (id TEXT, url TEXT);")
        .execute(&pool)
        .await
        .unwrap();

    let host = env::var("HOST").expect("HOST must be set");
    let port = env::var("PORT")
        .expect("PORT must be set")
        .parse::<u16>()
        .expect("PORT must be a number");
    let convert_url = env::var("CONVERT_URL").expect("CONVERT_URL must be set");
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    HttpServer::new(move || {
        App::new()
            .route("/", web::get().to(base))
            .route("/{id}", web::get().to(short_url))
            .route("/shorten", web::post().to(create_shorten))
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(web::Data::new(AppState {
                pool: pool.clone(),
                convert_url: convert_url.clone(),
            }))
    })
    .bind((host, port))?
    .run()
    .await
}
