//! HTTP route handlers.

use std::sync::Arc;

use axum::{
  extract::{Multipart, State},
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use dft2dxf_core::{
  convert_bytes, inspect_bytes, validate_bytes, ConvertOptions, ConvertOutput, InspectOutput,
  ValidateOutput,
};
use serde::Serialize;
use tokio::sync::TryAcquireError;

use crate::state::AppState;

/// Health check response.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
  /// Service status.
  pub status: &'static str,
  /// Crate version.
  pub version: &'static str,
}

/// Ready probe response.
#[derive(Debug, Serialize)]
pub struct ReadyResponse {
  /// Ready status.
  pub status: &'static str,
  /// Available worker permits.
  pub available_workers: usize,
}

/// JSON API error body.
#[derive(Debug, Serialize)]
pub struct ErrorBody {
  /// Error message.
  pub error: String,
}

/// JSON convert response envelope.
#[derive(Debug, Serialize)]
pub struct ConvertResponse {
  /// Conversion summary and payloads.
  #[serde(flatten)]
  pub output: ConvertOutput,
  /// Base64-encoded DXF bytes.
  pub dxf_base64: String,
  /// Base64-encoded SVG when requested.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub svg_base64: Option<String>,
}

/// `GET /health`
pub async fn health() -> Json<HealthResponse> {
  Json(HealthResponse {
    status: "ok",
    version: env!("CARGO_PKG_VERSION"),
  })
}

/// `GET /ready`
pub async fn ready(State(state): State<Arc<AppState>>) -> Json<ReadyResponse> {
  Json(ReadyResponse {
    status: "ready",
    available_workers: state.pool.available_permits(),
  })
}

/// `POST /v1/convert`
///
/// # Errors
///
/// Returns [`ApiError`] when the upload is invalid or conversion fails.
pub async fn convert(
  State(state): State<Arc<AppState>>,
  mut multipart: Multipart,
) -> Result<Json<ConvertResponse>, ApiError> {
  let (bytes, include_svg, include_cam_json, sheet, units) = read_upload(&mut multipart).await?;
  let limits = state.limits;
  let permit = state.pool.clone().try_acquire_owned().map_err(|err| match err {
    TryAcquireError::Closed => ApiError::service_unavailable("worker pool closed"),
    TryAcquireError::NoPermits => ApiError::service_unavailable("worker pool at capacity"),
  })?;

  let options = ConvertOptions {
    limits,
    sheet,
    units,
    include_svg,
    include_cam_json,
  };

  let output = tokio::task::spawn_blocking(move || convert_bytes(&bytes, &options))
    .await
    .map_err(|err| ApiError::internal(err.to_string()))?
    .map_err(|err| ApiError::from_core(&err))?;

  drop(permit);

  let dxf_base64 = STANDARD.encode(&output.dxf);
  let svg_base64 = output
    .svg
    .as_ref()
    .map(|svg| STANDARD.encode(svg.as_bytes()));

  Ok(Json(ConvertResponse {
    output,
    dxf_base64,
    svg_base64,
  }))
}

/// `POST /v1/inspect`
///
/// # Errors
///
/// Returns [`ApiError`] when the upload is invalid or inspection fails.
pub async fn inspect(
  State(state): State<Arc<AppState>>,
  mut multipart: Multipart,
) -> Result<Json<InspectOutput>, ApiError> {
  let (bytes, _, _, sheet, units) = read_upload(&mut multipart).await?;
  let options = ConvertOptions {
    limits: state.limits,
    sheet,
    units,
    include_svg: false,
    include_cam_json: false,
  };
  let output = inspect_bytes(&bytes, &options).map_err(|err| ApiError::from_core(&err))?;
  Ok(Json(output))
}

/// `POST /v1/validate`
///
/// # Errors
///
/// Returns [`ApiError`] when the upload is invalid or validation fails.
pub async fn validate(
  State(state): State<Arc<AppState>>,
  mut multipart: Multipart,
) -> Result<Json<ValidateOutput>, ApiError> {
  let (bytes, _, _, sheet, units) = read_upload(&mut multipart).await?;
  let options = ConvertOptions {
    limits: state.limits,
    sheet,
    units,
    include_svg: false,
    include_cam_json: false,
  };
  let output = validate_bytes(&bytes, &options).map_err(|err| ApiError::from_core(&err))?;
  Ok(Json(output))
}

async fn read_upload(
  multipart: &mut Multipart,
) -> Result<(Vec<u8>, bool, bool, Option<u32>, Option<String>), ApiError> {
  let mut file_bytes = None;
  let mut include_svg = false;
  let mut include_cam_json = false;
  let mut sheet = None;
  let mut units = None;

  while let Some(field) = multipart
    .next_field()
    .await
    .map_err(|err| ApiError::bad_request(err.to_string()))?
  {
    let name = field.name().unwrap_or_default().to_string();
    match name.as_str() {
      "file" => {
        file_bytes = Some(
          field
            .bytes()
            .await
            .map_err(|err| ApiError::bad_request(err.to_string()))?
            .to_vec(),
        );
      }
      "include_svg" => {
        let value = field.text().await.unwrap_or_default();
        include_svg = value == "true" || value == "1";
      }
      "include_cam_json" => {
        let value = field.text().await.unwrap_or_default();
        include_cam_json = value == "true" || value == "1";
      }
      "sheet" => {
        let value = field.text().await.unwrap_or_default();
        sheet = value.parse().ok();
      }
      "units" => {
        units = field.text().await.ok();
      }
      _ => {}
    }
  }

  let bytes = file_bytes.ok_or_else(|| ApiError::bad_request("missing file field"))?;
  Ok((bytes, include_svg, include_cam_json, sheet, units))
}

/// HTTP API error type.
pub struct ApiError {
  status: StatusCode,
  message: String,
}

impl ApiError {
  fn bad_request(message: impl Into<String>) -> Self {
    Self {
      status: StatusCode::BAD_REQUEST,
      message: message.into(),
    }
  }

  fn internal(message: impl Into<String>) -> Self {
    Self {
      status: StatusCode::INTERNAL_SERVER_ERROR,
      message: message.into(),
    }
  }

  fn service_unavailable(message: impl Into<String>) -> Self {
    Self {
      status: StatusCode::SERVICE_UNAVAILABLE,
      message: message.into(),
    }
  }

  fn from_core(err: &dft2dxf_core::CoreError) -> Self {
    Self {
      status: StatusCode::UNPROCESSABLE_ENTITY,
      message: err.to_string(),
    }
  }
}

impl IntoResponse for ApiError {
  fn into_response(self) -> Response {
    (
      self.status,
      Json(ErrorBody {
        error: self.message,
      }),
    )
      .into_response()
  }
}
