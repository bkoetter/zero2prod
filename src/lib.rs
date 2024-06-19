use std::net::TcpListener;

use actix_web::{App, HttpResponse, HttpServer, web};
use actix_web::dev::Server;

#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(tcp_listener: TcpListener) -> std::io::Result<Server> {
    Ok(
        HttpServer::new(|| {
            App::new()
                .route("/health_check", web::get().to(health_check))
                .route("/subscriptions", web::post().to(subscribe))
        })
            .listen(tcp_listener)?
            .run()
    )
}