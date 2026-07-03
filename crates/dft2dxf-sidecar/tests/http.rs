//! HTTP sidecar integration tests.

use std::sync::Arc;

use axum::body::Body;
use dft2dxf_testkit::{build_minimal_dft, build_rectangle_emf, MinimalDftSpec};
use http_body_util::BodyExt;
use tower::ServiceExt;

use dft2dxf_sidecar::{app, state::AppState};

fn synthetic_dft_bytes() -> Vec<u8> {
  let dir = tempfile::tempdir().unwrap();
  let path = dir.path().join("sample.dft");
  let emf = build_rectangle_emf(0, 0, 50, 50);
  build_minimal_dft(&path, &MinimalDftSpec::one_sheet("S1", emf)).unwrap();
  std::fs::read(path).unwrap()
}

fn multipart_body(bytes: &[u8], extra_fields: &[(&str, &str)]) -> (String, Vec<u8>) {
  let boundary = "testboundary";
  let mut payload = Vec::new();
  for (name, value) in extra_fields {
    payload.extend_from_slice(
      format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n"
      )
      .as_bytes(),
    );
  }
  payload.extend_from_slice(
    format!(
      "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"sample.dft\"\r\nContent-Type: application/octet-stream\r\n\r\n"
    )
    .as_bytes(),
  );
  payload.extend_from_slice(bytes);
  payload.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
  (boundary.to_string(), payload)
}

#[tokio::test]
async fn health_returns_ok() {
  let router = app(Arc::new(AppState::from_env()));
  let response = router
    .oneshot(
      axum::http::Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn ready_returns_worker_count() {
  let router = app(Arc::new(AppState::from_env()));
  let response = router
    .oneshot(
      axum::http::Request::builder()
        .uri("/ready")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), axum::http::StatusCode::OK);
  let body = response.into_body().collect().await.unwrap().to_bytes();
  let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
  assert_eq!(json["status"], "ready");
  assert!(json["available_workers"].as_u64().is_some());
}

#[tokio::test]
async fn convert_returns_dxf_base64() {
  let router = app(Arc::new(AppState::from_env()));
  let bytes = synthetic_dft_bytes();
  let (boundary, payload) = multipart_body(&bytes, &[("include_svg", "true")]);

  let response = router
    .oneshot(
      axum::http::Request::builder()
        .method("POST")
        .uri("/v1/convert")
        .header(
          "content-type",
          format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(payload))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), axum::http::StatusCode::OK);
  let body = response.into_body().collect().await.unwrap().to_bytes();
  let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
  assert!(json.get("dxf_base64").is_some());
  assert!(json.get("svg_base64").is_some());
}

#[tokio::test]
async fn inspect_returns_solid_edge_summary() {
  let router = app(Arc::new(AppState::from_env()));
  let bytes = synthetic_dft_bytes();
  let (boundary, payload) = multipart_body(&bytes, &[]);

  let response = router
    .oneshot(
      axum::http::Request::builder()
        .method("POST")
        .uri("/v1/inspect")
        .header(
          "content-type",
          format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(payload))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), axum::http::StatusCode::OK);
  let body = response.into_body().collect().await.unwrap().to_bytes();
  let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
  assert_eq!(json["format"], "solid_edge");
  assert_eq!(json["sheets"], 1);
}

#[tokio::test]
async fn validate_returns_ok_status() {
  let router = app(Arc::new(AppState::from_env()));
  let bytes = synthetic_dft_bytes();
  let (boundary, payload) = multipart_body(&bytes, &[]);

  let response = router
    .oneshot(
      axum::http::Request::builder()
        .method("POST")
        .uri("/v1/validate")
        .header(
          "content-type",
          format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(payload))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), axum::http::StatusCode::OK);
  let body = response.into_body().collect().await.unwrap().to_bytes();
  let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
  assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn convert_missing_file_returns_bad_request() {
  let router = app(Arc::new(AppState::from_env()));
  let boundary = "emptyboundary";
  let payload = format!("--{boundary}--\r\n");

  let response = router
    .oneshot(
      axum::http::Request::builder()
        .method("POST")
        .uri("/v1/convert")
        .header(
          "content-type",
          format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(payload))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn convert_invalid_dft_returns_unprocessable() {
  let router = app(Arc::new(AppState::from_env()));
  let (boundary, payload) = multipart_body(b"not-a-dft", &[]);

  let response = router
    .oneshot(
      axum::http::Request::builder()
        .method("POST")
        .uri("/v1/convert")
        .header(
          "content-type",
          format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(payload))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(
    response.status(),
    axum::http::StatusCode::UNPROCESSABLE_ENTITY
  );
}

#[tokio::test]
async fn convert_returns_503_when_pool_exhausted() {
  let state = Arc::new(AppState::with_concurrency(1));
  let _permit = state.pool.clone().acquire_owned().await.unwrap();
  let router = app(state);
  let bytes = synthetic_dft_bytes();
  let (boundary, payload) = multipart_body(&bytes, &[]);

  let response = router
    .oneshot(
      axum::http::Request::builder()
        .method("POST")
        .uri("/v1/convert")
        .header(
          "content-type",
          format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(payload))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(
    response.status(),
    axum::http::StatusCode::SERVICE_UNAVAILABLE
  );
  let body = response.into_body().collect().await.unwrap().to_bytes();
  let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
  assert!(json["error"]
    .as_str()
    .unwrap()
    .contains("worker pool at capacity"));
}

#[tokio::test]
async fn convert_returns_503_when_pool_closed() {
  let state = Arc::new(AppState::with_concurrency(1));
  state.pool.close();
  let router = app(state);
  let bytes = synthetic_dft_bytes();
  let (boundary, payload) = multipart_body(&bytes, &[]);

  let response = router
    .oneshot(
      axum::http::Request::builder()
        .method("POST")
        .uri("/v1/convert")
        .header(
          "content-type",
          format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(payload))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(
    response.status(),
    axum::http::StatusCode::SERVICE_UNAVAILABLE
  );
  let body = response.into_body().collect().await.unwrap().to_bytes();
  let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
  assert!(json["error"]
    .as_str()
    .unwrap()
    .contains("worker pool closed"));
}
