use crate::{config::ApiKey, error::AppError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashSet, io::Read, time::Duration};

pub const LINEAR_GRAPHQL_ENDPOINT: &str = "https://api.linear.app/graphql";
pub const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Serialize)]
pub struct Request<'a> {
    pub query: &'a str,
    #[serde(rename = "operationName")]
    pub operation_name: &'a str,
    pub variables: Value,
}

#[derive(Debug, Deserialize)]
pub struct Envelope<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQlError>>,
}

#[derive(Debug, Deserialize)]
pub struct GraphQlError {
    pub message: String,
}

pub fn decode_envelope<T: for<'de> Deserialize<'de>>(body: &[u8]) -> Result<T, AppError> {
    let envelope: Envelope<T> = serde_json::from_slice(body)
        .map_err(|_| AppError::operational("Linear returned an invalid GraphQL response"))?;
    if envelope
        .errors
        .as_ref()
        .is_some_and(|errors| !errors.is_empty())
    {
        return Err(AppError::operational("Linear rejected the GraphQL request"));
    }
    envelope
        .data
        .ok_or_else(|| AppError::operational("Linear returned no GraphQL data"))
}

/// Guards cursor-based loops against malformed or non-progressing page metadata.
#[derive(Default)]
pub struct PaginationGuard {
    pages: u8,
    cursors: HashSet<String>,
}

impl PaginationGuard {
    pub fn next_cursor(
        &mut self,
        has_next_page: bool,
        end_cursor: Option<&str>,
    ) -> Result<Option<String>, AppError> {
        self.pages = self.pages.saturating_add(1);
        if self.pages > 100 {
            return Err(AppError::operational("pagination exceeded 100 pages"));
        }
        if !has_next_page {
            return Ok(None);
        }
        let cursor = end_cursor
            .filter(|cursor| !cursor.is_empty())
            .ok_or_else(|| {
                AppError::operational("Linear returned a page without a continuation cursor")
            })?;
        if !self.cursors.insert(cursor.to_owned()) {
            return Err(AppError::operational(
                "Linear returned a repeated pagination cursor",
            ));
        }
        Ok(Some(cursor.to_owned()))
    }
}

/// Blocking GraphQL client. Tests may provide a local endpoint without exposing an end-user override.
pub struct LinearClient {
    endpoint: String,
    api_key: ApiKey,
}

impl LinearClient {
    pub fn new(api_key: ApiKey) -> Self {
        Self::with_endpoint(api_key, LINEAR_GRAPHQL_ENDPOINT)
    }
    #[cfg(test)]
    pub fn with_test_endpoint(api_key: ApiKey, endpoint: &str) -> Self {
        Self::with_endpoint(api_key, endpoint)
    }
    fn with_endpoint(api_key: ApiKey, endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_owned(),
            api_key,
        }
    }

    pub fn execute<T: for<'de> Deserialize<'de>>(
        &self,
        request: &Request<'_>,
    ) -> Result<T, AppError> {
        let body = serde_json::to_vec(request)
            .map_err(|_| AppError::operational("could not encode GraphQL request"))?;
        let agent = ureq::Agent::config_builder()
            .tls_config(
                ureq::tls::TlsConfig::builder()
                    .root_certs(ureq::tls::RootCerts::PlatformVerifier)
                    .build(),
            )
            .timeout_connect(Some(Duration::from_secs(10)))
            .timeout_per_call(Some(Duration::from_secs(30)))
            .max_redirects(0)
            .build()
            .new_agent();
        let mut response = agent
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .header("Authorization", self.api_key.authorization_value())
            .send(&body)
            .map_err(|error| match error {
                ureq::Error::StatusCode(401 | 403) => {
                    AppError::input("Linear authentication failed; check LINEAR_API_KEY")
                }
                _ => AppError::operational(format!("Linear request failed: {error}")),
            })?;
        let mut bytes = Vec::new();
        let mut reader = response.body_mut().as_reader();
        reader
            .by_ref()
            .take((MAX_RESPONSE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|_| AppError::operational("could not read Linear response"))?;
        if bytes.len() > MAX_RESPONSE_BYTES {
            return Err(AppError::operational(
                "Linear response exceeded 2 MiB limit",
            ));
        }
        decode_envelope(&bytes)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{config::ApiKey, error::ErrorKind};
    use std::{
        io::{BufRead, BufReader, Read, Write},
        net::TcpListener,
        thread::{self, JoinHandle},
    };

    #[derive(Debug, Deserialize)]
    struct Data {
        value: u8,
    }

    pub(crate) fn serve_once(status: &str, response_body: &str) -> (String, JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}/graphql", listener.local_addr().unwrap());
        let status = status.to_owned();
        let response_body = response_body.to_owned();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request = String::new();
            let mut content_length = 0;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" {
                    break;
                }
                if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    content_length = value.trim().parse().unwrap();
                }
                request.push_str(&line);
            }
            let mut body = vec![0; content_length];
            reader.read_exact(&mut body).unwrap();
            request.push_str("\r\n");
            request.push_str(&String::from_utf8(body).unwrap());
            write!(
                stream,
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response_body}",
                response_body.len()
            )
            .unwrap();
            request
        });
        (endpoint, handle)
    }

    #[test]
    fn rejects_graphql_errors_even_with_data() {
        assert!(
            decode_envelope::<Data>(br#"{"data":{"value":1},"errors":[{"message":"no"}]}"#)
                .is_err()
        );
    }

    #[test]
    fn decodes_data_and_guards_cursors() {
        assert_eq!(
            decode_envelope::<Data>(br#"{"data":{"value":1}}"#)
                .unwrap()
                .value,
            1
        );
        let mut guard = PaginationGuard::default();
        assert_eq!(
            guard.next_cursor(true, Some("one")).unwrap(),
            Some("one".into())
        );
        assert!(guard.next_cursor(true, Some("one")).is_err());
    }

    #[test]
    fn sends_graphql_request_with_secret_only_in_authorization_header() {
        let secret = "lin_api_test_secret";
        let (endpoint, captured) = serve_once("200 OK", r#"{"data":{"value":7}}"#);
        let client = LinearClient::with_test_endpoint(ApiKey::for_test(secret), &endpoint);

        let data: Data = client
            .execute(&Request {
                query: "query Test { value }",
                operation_name: "Test",
                variables: serde_json::json!({"input":"safe"}),
            })
            .unwrap();
        let request = captured.join().unwrap();
        let (headers, body) = request.split_once("\r\n\r\n").unwrap();

        assert_eq!(data.value, 7);
        assert!(
            headers
                .to_ascii_lowercase()
                .contains(&format!("authorization: {secret}"))
        );
        assert!(
            headers
                .to_ascii_lowercase()
                .contains("content-type: application/json")
        );
        assert!(!body.contains(secret));
        assert!(body.contains(r#""operationName":"Test""#));
    }

    #[test]
    fn authentication_failures_are_actionable_and_redacted() {
        let secret = "lin_api_test_secret";
        let (endpoint, captured) = serve_once("401 Unauthorized", r#"{"error":"no"}"#);
        let client = LinearClient::with_test_endpoint(ApiKey::for_test(secret), &endpoint);

        let error = client
            .execute::<Data>(&Request {
                query: "query Test { value }",
                operation_name: "Test",
                variables: serde_json::json!({}),
            })
            .unwrap_err();
        let _ = captured.join().unwrap();

        assert_eq!(error.kind, ErrorKind::Input);
        assert!(error.message.contains("LINEAR_API_KEY"));
        assert!(!error.message.contains(secret));
    }
}
