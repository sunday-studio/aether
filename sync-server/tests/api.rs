use std::sync::Arc;

use aether_sync_server::{handlers, storage::Storage};
use axum::{
    body::{to_bytes, Body},
    http::{header, Request, Response, StatusCode},
    Router,
};
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tempfile::TempDir;
use tokio::{net::TcpListener, task::JoinHandle, time::Duration};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, Message},
};
use tower::ServiceExt;

const SEED: &str = "test server seed phrase";

struct TestServer {
    app: Router,
    url: String,
    _data: TempDir,
    task: JoinHandle<()>,
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn test_server() -> TestServer {
    let data = tempfile::tempdir().expect("temporary data directory");
    let storage = Storage::new(&data.path().join("sync.db"), data.path()).expect("storage");
    storage.initialize_salt().expect("salt");
    let (broadcast, _) = tokio::sync::broadcast::channel(16);
    let app = handlers::router(Arc::new(storage), broadcast, Arc::from(SEED));

    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("listener address");
    let server_app = app.clone();
    let task = tokio::spawn(async move {
        axum::serve(listener, server_app.into_make_service())
            .await
            .expect("test server should run");
    });

    TestServer {
        app,
        url: format!("http://{address}"),
        _data: data,
        task,
    }
}

async fn response(app: &Router, request: Request<Body>) -> Response<Body> {
    app.clone().oneshot(request).await.expect("response")
}

async fn response_json(response: Response<Body>) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body"),
    )
    .expect("json response")
}

async fn register(app: &Router, device_id: &str) -> String {
    let request = Request::post("/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "device_id": device_id,
                "hostname": "test-host",
                "server_seed_phrase": SEED,
            })
            .to_string(),
        ))
        .expect("registration request");
    let response = response(app, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    response_json(response).await["device_token"]
        .as_str()
        .expect("device token")
        .to_owned()
}

fn authorized_request(
    method: &str,
    uri: &str,
    device_id: &str,
    token: &str,
    body: Body,
) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header("x-aether-device-id", device_id)
        .header(header::CONTENT_TYPE, "application/json")
        .body(body)
        .expect("authorized request")
}

#[tokio::test]
async fn readiness_registration_and_authentication_are_enforced() {
    let server = test_server().await;

    let readiness = response(
        &server.app,
        Request::get("/ready").body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(readiness.status(), StatusCode::OK);

    let rejected = response(
        &server.app,
        Request::post("/register")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "device_id": "device-one",
                    "hostname": "test-host",
                    "server_seed_phrase": "wrong seed phrase",
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(rejected.status(), StatusCode::UNAUTHORIZED);

    let token = register(&server.app, "device-one").await;
    let unauthorized = response(
        &server.app,
        authorized_request("GET", "/pull", "device-one", "not-the-token", Body::empty()),
    )
    .await;
    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

    let authorized = response(
        &server.app,
        authorized_request("GET", "/pull", "device-one", &token, Body::empty()),
    )
    .await;
    assert_eq!(authorized.status(), StatusCode::OK);
}

#[tokio::test]
async fn push_is_idempotent_and_pull_is_paginated() {
    let server = test_server().await;
    let token = register(&server.app, "device-one").await;
    let changes: Vec<Value> = (0..501)
        .map(|_| json!({ "nonce": "AQ==", "ciphertext": "YQ==" }))
        .collect();
    let payload = json!({
        "batch_id": "batch-one",
        "device_hostname": "test-host",
        "changes": changes,
    })
    .to_string();

    for _ in 0..2 {
        let pushed = response(
            &server.app,
            authorized_request(
                "POST",
                "/push",
                "device-one",
                &token,
                Body::from(payload.clone()),
            ),
        )
        .await;
        assert_eq!(pushed.status(), StatusCode::OK);
    }

    let first = response(
        &server.app,
        authorized_request("GET", "/pull", "device-one", &token, Body::empty()),
    )
    .await;
    assert_eq!(first.status(), StatusCode::OK);
    let first = response_json(first).await;
    assert_eq!(first["changes"].as_array().unwrap().len(), 500);
    assert_eq!(first["has_more"], true);
    let cursor = first["next_cursor"].as_object().expect("next cursor");
    let cursor = format!("{}:{}", cursor["received_at"], cursor["change_id"]);

    let second = response(
        &server.app,
        authorized_request(
            "GET",
            &format!("/pull?cursor={cursor}"),
            "device-one",
            &token,
            Body::empty(),
        ),
    )
    .await;
    assert_eq!(second.status(), StatusCode::OK);
    let second = response_json(second).await;
    assert_eq!(second["changes"].as_array().unwrap().len(), 1);
    assert_eq!(second["has_more"], false);
}

#[tokio::test]
async fn media_round_trip_requires_device_authentication() {
    let server = test_server().await;
    let token = register(&server.app, "device-one").await;
    let hash = format!("sha256:{}", "a".repeat(64));

    let unauthorized = response(
        &server.app,
        Request::put(format!("/media/{hash}"))
            .body(Body::from("encrypted media"))
            .unwrap(),
    )
    .await;
    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

    let uploaded = response(
        &server.app,
        authorized_request(
            "PUT",
            &format!("/media/{hash}"),
            "device-one",
            &token,
            Body::from("encrypted media"),
        ),
    )
    .await;
    assert_eq!(uploaded.status(), StatusCode::OK);

    let downloaded = response(
        &server.app,
        authorized_request(
            "GET",
            &format!("/media/{hash}"),
            "device-one",
            &token,
            Body::empty(),
        ),
    )
    .await;
    assert_eq!(downloaded.status(), StatusCode::OK);
    assert_eq!(
        to_bytes(downloaded.into_body(), usize::MAX).await.unwrap(),
        &b"encrypted media"[..]
    );
}

#[tokio::test]
async fn websocket_receives_a_sync_notification_from_another_device() {
    let server = test_server().await;
    let first_token = register(&server.app, "device-one").await;
    let second_token = register(&server.app, "device-two").await;
    let websocket_url = server.url.replacen("http://", "ws://", 1) + "/ws?hostname=test-host";
    let mut request = websocket_url
        .into_client_request()
        .expect("websocket request");
    request.headers_mut().insert(
        header::AUTHORIZATION,
        format!("Bearer {first_token}").parse().unwrap(),
    );
    request
        .headers_mut()
        .insert("x-aether-device-id", "device-one".parse().unwrap());
    let (mut websocket, _) = connect_async(request).await.expect("websocket connects");

    let mut connected = false;
    for _ in 0..20 {
        let devices = response(
            &server.app,
            authorized_request("GET", "/devices", "device-one", &first_token, Body::empty()),
        )
        .await;
        connected = response_json(devices).await["devices"]
            .as_array()
            .unwrap()
            .iter()
            .any(|device| device["id"] == "device-one" && device["connected"] == true);
        if connected {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(connected, "websocket device should be tracked as connected");

    let pushed = response(
        &server.app,
        authorized_request(
            "POST",
            "/push",
            "device-two",
            &second_token,
            Body::from(
                json!({
                    "batch_id": "batch-two",
                    "device_hostname": "test-host",
                    "changes": [{ "nonce": "AQ==", "ciphertext": "Yg==" }],
                })
                .to_string(),
            ),
        ),
    )
    .await;
    assert_eq!(pushed.status(), StatusCode::OK);

    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            match websocket
                .next()
                .await
                .expect("websocket message")
                .expect("websocket message should succeed")
            {
                Message::Text(message) if message == "sync" => break,
                Message::Ping(payload) => websocket
                    .send(Message::Pong(payload))
                    .await
                    .expect("websocket pong"),
                _ => {}
            }
        }
    })
    .await
    .expect("sync notification should arrive");
    websocket.close(None).await.expect("websocket closes");
}
