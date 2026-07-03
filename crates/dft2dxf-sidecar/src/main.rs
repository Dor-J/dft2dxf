//! `dft2dxf` HTTP conversion sidecar binary.

use dft2dxf_sidecar::run_from_env;

#[tokio::main]
async fn main() {
  run_from_env().await.expect("server error");
}
