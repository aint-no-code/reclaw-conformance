use std::net::TcpStream;

use reqwest::{blocking::Client, StatusCode};
use serde_json::Value;
use thiserror::Error;
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};

pub trait ConformanceTransport {
    fn get_json(&self, path: &str) -> Result<Value, TransportError>;
    fn post_json(&self, path: &str, body: &Value) -> Result<(u16, Value), TransportError>;
    fn websocket_first_response(&self, frame: &Value) -> Result<Value, TransportError>;
    fn websocket_exchange(&self, frames: &[Value]) -> Result<Vec<Value>, TransportError>;
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

    fn post_json(&self, path: &str, body: &Value) -> Result<(u16, Value), TransportError> {
        let path = normalize_path(path);
        let url = format!("{}{}", self.base_url, path);

        let response = self
            .client
            .post(&url)
            .json(body)
            .send()
            .map_err(|error| TransportError::Http(error.to_string()))?;

        let status = u16::from(response.status());
        let payload = response
            .json::<Value>()
            .map_err(|error| TransportError::Protocol(error.to_string()))?;

        Ok((status, payload))
    }

    fn websocket_first_response(&self, frame: &Value) -> Result<Value, TransportError> {
        let ws_url = websocket_url(&self.base_url);
        let (mut socket, _) = connect(ws_url.as_str())
            .map_err(|error| TransportError::Http(format!("websocket connect failed: {error}")))?;

        send_ws_json(&mut socket, frame)?;
        read_ws_json(&mut socket)
    }

    fn websocket_exchange(&self, frames: &[Value]) -> Result<Vec<Value>, TransportError> {
        if frames.is_empty() {
            return Err(TransportError::Protocol(
                "websocket exchange requires at least one frame".to_owned(),
            ));
        }

        let ws_url = websocket_url(&self.base_url);
        let (mut socket, _) = connect(ws_url.as_str())
            .map_err(|error| TransportError::Http(format!("websocket connect failed: {error}")))?;

        let mut responses = Vec::with_capacity(frames.len());
        for frame in frames {
            send_ws_json(&mut socket, frame)?;
            responses.push(read_ws_json(&mut socket)?);
        }

        Ok(responses)
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

fn websocket_url(base_url: &str) -> String {
    if let Some(host) = base_url.strip_prefix("http://") {
        format!("ws://{host}/ws")
    } else if let Some(host) = base_url.strip_prefix("https://") {
        format!("wss://{host}/ws")
    } else {
        format!("{base_url}/ws")
    }
}

fn send_ws_json(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    payload: &Value,
) -> Result<(), TransportError> {
    let encoded = serde_json::to_string(payload).map_err(|error| {
        TransportError::Protocol(format!("failed to encode websocket frame: {error}"))
    })?;
    socket
        .send(Message::Text(encoded.into()))
        .map_err(|error| TransportError::Http(format!("websocket send failed: {error}")))
}

fn read_ws_json(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
) -> Result<Value, TransportError> {
    loop {
        let message = socket
            .read()
            .map_err(|error| TransportError::Http(format!("websocket read failed: {error}")))?;

        match message {
            Message::Text(text) => {
                return serde_json::from_str(text.as_ref()).map_err(|error| {
                    TransportError::Protocol(format!("invalid websocket frame JSON: {error}"))
                });
            }
            Message::Ping(payload) => {
                socket.send(Message::Pong(payload)).map_err(|error| {
                    TransportError::Http(format!("websocket pong failed: {error}"))
                })?;
            }
            Message::Pong(_) => continue,
            Message::Close(_) => {
                return Err(TransportError::Protocol(
                    "websocket closed before response".to_owned(),
                ));
            }
            Message::Binary(_) => {
                return Err(TransportError::Protocol(
                    "unexpected binary websocket frame".to_owned(),
                ));
            }
            Message::Frame(_) => continue,
        }
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
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    use serde_json::json;
    use tungstenite::{accept, Message};

    use crate::transport::{websocket_url, ConformanceTransport, HttpTransport};

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

    #[test]
    fn post_json_returns_status_and_payload() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let addr = listener
            .local_addr()
            .expect("listener should expose local addr");

        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("connection should arrive");
            let mut buffer = [0_u8; 4096];
            let _ = stream
                .read(&mut buffer)
                .expect("request should be readable");

            let body = r#"{"ok":false,"error":{"code":"NOT_FOUND"}}"#;
            let response = format!(
                "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .expect("response should be writable");
        });

        let transport =
            HttpTransport::new(format!("http://{addr}")).expect("transport should construct");
        let (status, payload) = transport
            .post_json("/channels/nonexistent/webhook", &json!({}))
            .expect("request should succeed");

        assert_eq!(status, 404);
        assert_eq!(payload["error"]["code"], "NOT_FOUND");
        let _ = server.join();
    }

    #[test]
    fn websocket_url_maps_http_scheme_to_ws() {
        assert_eq!(
            websocket_url("http://127.0.0.1:18789"),
            "ws://127.0.0.1:18789/ws"
        );
        assert_eq!(websocket_url("https://example.com"), "wss://example.com/ws");
    }

    #[test]
    fn websocket_first_response_returns_json_payload() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let addr = listener
            .local_addr()
            .expect("listener should expose local addr");

        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("connection should arrive");
            let mut ws = accept(stream).expect("websocket handshake should succeed");

            let request = ws.read().expect("request frame should arrive");
            let text = request.into_text().expect("request frame should be text");
            let parsed: serde_json::Value =
                serde_json::from_str(text.as_ref()).expect("frame JSON should parse");
            assert_eq!(parsed["id"], "bad-1");

            ws.send(Message::Text(
                json!({
                    "type": "res",
                    "id": "bad-1",
                    "ok": false,
                    "error": {
                        "code": "INVALID_REQUEST",
                        "message": "first request must be connect"
                    }
                })
                .to_string()
                .into(),
            ))
            .expect("response should be sent");
        });

        let transport =
            HttpTransport::new(format!("http://{addr}")).expect("transport should construct");
        let response = transport
            .websocket_first_response(&json!({
                "type": "req",
                "id": "bad-1",
                "method": "health",
                "params": {}
            }))
            .expect("response should be received");

        assert_eq!(response["id"], "bad-1");
        assert_eq!(response["error"]["code"], "INVALID_REQUEST");
        let _ = server.join();
    }
}
