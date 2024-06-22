use std::net::TcpListener;

use sqlx::PgPool;

use zero2prod::configuration::get_configuration;
use zero2prod::startup;

struct TestApp {
    address: String,
    db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let tcp_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = tcp_listener.local_addr().unwrap().port();
    let configuration = get_configuration().unwrap();
    let connection_pool = PgPool::connect(
        &configuration.database.connection_string()
    )
        .await
        .expect("Failed to connect to Postgres");

    tokio::spawn(
        startup::run(tcp_listener, connection_pool.clone())
            .expect("Failed to bind address")
    );

    TestApp {
        address: format!("http://127.0.0.1:{port}"),
        db_pool: connection_pool,
    }
}

#[tokio::test]
async fn health_check_works() {
    let uri = spawn_app().await.address;

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
    let app = spawn_app().await;
    // let configuration = get_configuration().expect("Failed to read configuration");
    // let connection_string = configuration.database.connection_string();
    // let connection = PgConnection::connect(&connection_string)
    //     .await.expect("Failed to connect to Postgres.");

    let response = reqwest::Client::new()
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("name=le%20guin&email=ursula_le_guin%40gmail.com")
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let url = spawn_app().await.address;
    let test_cases = vec![
        ("naem=le%20guin", "missing email"),
        ("email=ursula_le_guin%40gmail.com", "missing name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = reqwest::Client::new()
            .post(&format!("{}/subscriptions", &url))
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
