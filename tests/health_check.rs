use std::net::TcpListener;

use sqlx::{Connection, PgConnection};

use zero2prod::configuration::get_configuration;
use zero2prod::startup;

fn spawn_app() -> String {
    let tcp_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = tcp_listener.local_addr().unwrap().port();
    tokio::spawn(startup::run(tcp_listener).expect("Failed to bind address"));
    format!("http://127.0.0.1:{port}")
}

#[tokio::test]
async fn health_check_works() {
    let uri = spawn_app();

    let response = reqwest::Client::new()
        .get(&format!("{uri}/health_check"))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let uri = spawn_app();
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    let _connection = PgConnection::connect(&connection_string)
        .await.expect("Failed to connect to Postgres.");

    let response = reqwest::Client::new()
        .post(&format!("{}/subscriptions", &uri))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("name=le%20guin&email=ursula_le_guin%40gmail.com")
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let uri = spawn_app();
    let test_cases = vec![
        ("naem=le%20guin", "missing email"),
        ("email=ursula_le_guin%40gmail.com", "missing name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = reqwest::Client::new()
            .post(&format!("{}/subscriptions", &uri))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.", error_message
        );
    }
}
