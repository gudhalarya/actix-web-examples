use std::env;

use actix_web::{App, HttpResponse, HttpServer, Responder, delete, get, post, web};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions, prelude::FromRow};

//This is the basics of the crud apis ------------>
//Database first
async fn get_db() -> PgPool {
    let db_url = env::var("DATABASE_URL").expect("Database url not found ");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&db_url)
        .await
        .expect("Could not connect to the backend");

    pool
}

//models are here ------------->
#[derive(Deserialize, Debug)]
pub struct CreateBooks {
    pub name: String,
    pub price: f64,
    pub available: bool,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Books {
    pub id: i32,
    pub name: String,
    pub price: f64,
    pub available: bool,
}
//This is for the health check only -------------->
#[get("/health_check")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Successfull")
}

#[post("/create_books")]
async fn create_books(
    pool: web::Data<PgPool>,
    payload: web::Json<CreateBooks>,
) -> Result<impl Responder, actix_web::Error> {
    let result = sqlx::query_as::<_, Books>(
        "INSERT INTO books (name, price, available) VALUES ($1,$2,$3) RETURNING * ",
    )
    .bind(&payload.name)
    .bind(payload.price)
    .bind(payload.available)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    Ok(HttpResponse::Ok().json(result))
}

#[get("/get_books")]
async fn get_books(pool: web::Data<PgPool>) -> Result<impl Responder, actix_web::Error> {
    let result = sqlx::query_as::<_, Books>("SELECT * FROM books")
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    Ok(HttpResponse::Ok().json(result))
}

#[delete("/delete_books/{id}")]
async fn delete_books(
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
) -> Result<impl Responder, actix_web::Error> {
    sqlx::query("DELETE FROM books WHERE id = $1")
        .bind(*id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    Ok(HttpResponse::Ok().body("Book deleted succesfully"))
}

//This is the main fn here ------------>
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = get_db().await;
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(health_check)
            .service(create_books)
            .service(get_books)
            .service(delete_books)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
