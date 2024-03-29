use actix_cors::Cors;
use actix_web::{http::header, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use env_logger::Env;
use rand::Rng;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::env;

const CHAR_SET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

struct AppState {
    pool: SqlitePool,
    convert_url: String,
    re: Regex,
}
#[derive(Serialize, Deserialize)]
struct CreateShortenPayload {
    url: String,
}

async fn base() -> impl Responder {
    HttpResponse::Ok().body("Shorten url service!")
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
    if !data.re.is_match(&payload.url) {
        return HttpResponse::BadRequest().body("Invalid url");
    }
    let result = sqlx::query!("SELECT id FROM Urls WHERE url = ?;", payload.url)
        .fetch_one(&data.pool)
        .await;
    match result {
        Ok(result) => match result.id {
            Some(id) => HttpResponse::Ok().body(format!("{}/{}", data.convert_url, id)),
            None => HttpResponse::NotFound().body("Not found"),
        },
        Err(_) => {
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
            return HttpResponse::Ok().body(format!("{}/{}", data.convert_url, id_content));
        }
    }
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
        let (allow_origin_ok, allow_origin) = match env::var("ALLOW_ORIGIN") {
            Ok(origin) => (true, origin),
            Err(_) => (false, "".to_string()),
        };
        App::new()
            .route("/", web::get().to(base))
            .route("/{id}", web::get().to(short_url))
            .route("/shorten", web::post().to(create_shorten))
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(web::Data::new(AppState {
                pool: pool.clone(),
                convert_url: convert_url.clone(),
                re: Regex::new(r"https?://[a-zA-Z0-9./?=_-]+").unwrap(),
            }))
            .wrap(
                Cors::default()
                    .allowed_origin_fn(move |origin, _req_head| {
                        origin.to_str().unwrap().starts_with(allow_origin.as_str())
                            && allow_origin_ok
                    })
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_header(header::CONTENT_TYPE),
            )
    })
    .bind((host, port))?
    .run()
    .await
}
