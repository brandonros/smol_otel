use std::sync::Arc;

use http::{Request, StatusCode, Uri};
use http_client::HttpClient;
use simple_error::{box_err, SimpleResult};

use crate::span_builder::SpanBuilder;
use crate::structs::*;

#[derive(Debug)]
pub struct OtlpTracer {
    pub traces_endpoint: Uri,
    pub metrics_endpoint: Uri,
    pub service_name: String,
    pub headers: String,
}

impl OtlpTracer {
    pub fn new(traces_endpoint: &str, metrics_endpoint: &str, service_name: &str) -> SimpleResult<Self> {
        let headers= std::env::var("OTEL_EXPORTER_OTLP_HEADERS").unwrap_or("".to_string());
        let traces_endpoint: Uri = traces_endpoint.parse()?;
        let metrics_endpoint: Uri = metrics_endpoint.parse()?;
        Ok(Self { 
            traces_endpoint, 
            metrics_endpoint, 
            service_name: service_name.to_string(), 
            headers
        })
    }

    async fn send_request(&self, endpoint: &Uri, body: String, error_context: &str) -> SimpleResult<()> {
        log::info!("sending request to {}", endpoint);

        let request_body_bytes = body.as_bytes().to_vec();
        let mut request_builder = Request::builder()
            .method("POST")
            .uri(endpoint)
            .header("Content-Type", "application/json")
            .header("Content-Length", request_body_bytes.len().to_string())
            .header("Host", endpoint.host().unwrap_or_default());

        if !self.headers.is_empty() {
            for header in self.headers.split(',') {
                if let Some((key, value)) = header.split_once('=') {
                    request_builder = request_builder.header(key.trim(), value.trim());
                }
            }
        }

        let request: Request<Vec<u8>> = request_builder.body(request_body_bytes)?;
        let mut stream = HttpClient::create_connection(&request).await?;
        let response = HttpClient::request(&mut stream, &request).await?;
        log::info!("response: {:02x?}", response);
        let response_body = String::from_utf8(response.body().clone())?;

        if response.status() != StatusCode::OK {
            return Err(box_err!(format!("failed to upload {}: {} {}", error_context, response.status(), response_body)));
        }

        Ok(())
    }

    pub async fn upload_traces(&self, resource_spans: Vec<ResourceSpan>) -> SimpleResult<()> {
        let root = ResourceSpansRoot { resource_spans };
        let request_body = miniserde::json::to_string(&root);
        self.send_request(&self.traces_endpoint, request_body, "traces").await
    }

    pub async fn upload_metrics(&self, resource_metrics: Vec<ResourceMetrics>) -> SimpleResult<()> {
        log::info!("uploading metrics");
        let root = ResourceMetricsRoot { resource_metrics };
        let request_body = miniserde::json::to_string(&root);
        self.send_request(&self.metrics_endpoint, request_body, "metrics").await
    }

    pub fn span(self: &Arc<Self>, name: &str) -> SpanBuilder {
        SpanBuilder::new(self.clone(), name)
    }
}
