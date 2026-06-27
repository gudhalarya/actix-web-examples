use std::env;

use actix_web::{App, HttpResponse, HttpServer, web};
use sqlx::{PgPool, postgres::PgPoolOptions};
//We will suse custom error handling here
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Hashing Error")]
    HashingError(#[from] argon2::Error),

    #[error("Database Error")]
    DatabaseError(#[from] sqlx::Error),
}

impl actix_web::ResponseError for AppError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        match self {
            AppError::HashingError(_) => {
                HttpResponse::InternalServerError().json("Hashing error occured")
            }
            AppError::DatabaseError(_) => {
                HttpResponse::InternalServerError().json("Database error occured")
            }
        }
    }
}

async fn get_db() -> Result<PgPool, AppError> {
    let db_url =
        env::var("DATABASE_URL").expect("Could not find the database url in the env file ");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect(&db_url)
        .await?;
    Ok(pool)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = get_db().await.expect("Could not get the database");
    HttpServer::new(move || App::new().app_data(web::Data::new(pool.clone())))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
