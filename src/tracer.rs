use http::{Request, StatusCode, Uri};
use http_client::HttpClient;
use simple_error::{box_err, SimpleResult};

use crate::structs::*;

pub struct OtlpTracer {}

impl OtlpTracer {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn upload_traces(&self, resource_spans: Vec<ResourceSpan>) -> SimpleResult<()> {
        let root = ResourceSpansRoot {
            resource_spans
        };
        let request_body = miniserde::json::to_string(&root);
        let request_body_bytes = request_body.as_bytes().to_vec();
        let uri: Uri = "http://tempo.node.external/v1/traces".parse()?;
        //let uri: Uri = "http://localhost:4318/v1/traces".parse()?;
        let request: Request<Vec<u8>> = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Content-Type", "application/json")
            .header("Content-Length", request_body_bytes.len().to_string())
            .header("Host", uri.host().unwrap_or_default())
            .body(request_body_bytes)?;
        let mut stream = HttpClient::create_connection(&request).await?;
        let response = HttpClient::request(&mut stream, &request).await?;
        let response_body = String::from_utf8(response.body().clone())?;
        if response.status() != StatusCode::OK {
            return Err(box_err!(format!("failed to upload traces: {} {}", response.status(), response_body)));
        }
        Ok(())
    }
}