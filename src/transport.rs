use reqwest::{blocking::Client, StatusCode};
use serde_json::Value;
use thiserror::Error;

pub trait ConformanceTransport {
    fn get_json(&self, path: &str) -> Result<Value, TransportError>;
}

pub struct HttpTransport {
    base_url: String,
    client: Client,
}

impl HttpTransport {
    pub fn new(base_url: impl Into<String>) -> Result<Self, TransportError> {
        let normalized = normalize_base_url(base_url.into())?;
        let client = Client::builder()
            .build()
            .map_err(|error| TransportError::Http(error.to_string()))?;

        Ok(Self {
            base_url: normalized,
            client,
        })
    }
}

impl ConformanceTransport for HttpTransport {
    fn get_json(&self, path: &str) -> Result<Value, TransportError> {
        let path = normalize_path(path);
        let url = format!("{}{}", self.base_url, path);

        let response = self
            .client
            .get(&url)
            .send()
            .map_err(|error| TransportError::Http(error.to_string()))?;

        if response.status() != StatusCode::OK {
            return Err(TransportError::Protocol(format!(
                "unexpected status {} for {path}",
                response.status()
            )));
        }

        response
            .json::<Value>()
            .map_err(|error| TransportError::Protocol(error.to_string()))
    }
}

fn normalize_base_url(input: String) -> Result<String, TransportError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(TransportError::Protocol(
            "base URL cannot be empty".to_owned(),
        ));
    }

    let without_trailing = trimmed.trim_end_matches('/').to_owned();
    if !(without_trailing.starts_with("http://") || without_trailing.starts_with("https://")) {
        return Err(TransportError::Protocol(
            "base URL must start with http:// or https://".to_owned(),
        ));
    }

    Ok(without_trailing)
}

fn normalize_path(path: &str) -> String {
    if path.starts_with('/') {
        path.to_owned()
    } else {
        format!("/{path}")
    }
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("http transport error: {0}")]
    Http(String),

    #[error("transport protocol error: {0}")]
    Protocol(String),
}

#[cfg(test)]
mod tests {
    use crate::transport::normalize_base_url;

    #[test]
    fn normalize_base_url_trims_and_strips_trailing_slash() {
        let normalized = normalize_base_url(" https://localhost:18789/ ".to_owned())
            .expect("base url should normalize");
        assert_eq!(normalized, "https://localhost:18789");
    }

    #[test]
    fn normalize_base_url_rejects_non_http_scheme() {
        let error = normalize_base_url("ws://localhost".to_owned()).expect_err("should fail");
        assert_eq!(
            error.to_string(),
            "transport protocol error: base URL must start with http:// or https://"
        );
    }
}
