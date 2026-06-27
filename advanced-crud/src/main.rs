use actix_web::{App, HttpServer, web};
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Hashing error")]
    HashingError(#[from] argon2::password_hash::Error),
    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error),
}

impl actix_web::ResponseError for AppError {
    // fixed trait name
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        match self {
            AppError::HashingError(_) => {
                actix_web::HttpResponse::InternalServerError().json("Hashing error occurred")
            }
            AppError::DatabaseError(_) => {
                actix_web::HttpResponse::InternalServerError().json("Database error occurred")
            }
        }
    }
}

async fn get_db() -> Result<PgPool, AppError> {
    let db_url = env::var("DATABASE_URL").expect("Could not find the database url in the env file");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&db_url)
        .await?;
    Ok(pool)
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string(); // convert to String
    Ok(hashed_password)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = get_db().await.expect("Could not connect to the db");
    HttpServer::new(move || App::new().app_data(web::Data::new(pool.clone())))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
