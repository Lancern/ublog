use std::time::Duration;

use rand::Rng;
use reqwest::{Client, IntoUrl, Method, Request, RequestBuilder, Response, StatusCode};
use serde::{Deserialize, Serialize};

use crate::api::{NotionApiError, NotionError};

#[derive(Debug)]
pub(super) struct NotionRequestExecutor {
    token: String,
    http_client: Client,
}

impl NotionRequestExecutor {
    pub(super) fn new<T>(token: T) -> Self
    where
        T: Into<String>,
    {
        let token = token.into();
        Self {
            token,
            http_client: Client::new(),
        }
    }

    pub(super) fn build_notion_request<T>(&self, method: Method, url: T) -> RequestBuilder
    where
        T: IntoUrl,
    {
        const NOTION_VERSION_HEADER_NAME: &str = "Notion-Version";
        const NOTION_VERSION_HEADER_VALUE: &str = "2022-06-28";

        self.http_client
            .request(method, url)
            .bearer_auth(&self.token)
            .header(NOTION_VERSION_HEADER_NAME, NOTION_VERSION_HEADER_VALUE)
    }

    pub(super) async fn execute(&self, mut req: Request) -> Result<Response, NotionApiError> {
        loop {
            let request_backup = req.try_clone().unwrap();

            let response = self.http_client.execute(req).await?;
            let response_status = response.status();

            if response_status.is_success() {
                return Ok(response);
            }

            match response_status {
                StatusCode::TOO_MANY_REQUESTS => {
                    self.handle_notion_rate_limit(response).await;
                }
                _ => {
                    let err = self.extract_notion_error(response).await;
                    return Err(err);
                }
            }

            req = request_backup;
        }
    }

    async fn handle_notion_rate_limit(&self, response: Response) {
        // The Retry-After header value should gives the amount of time in seconds to be waited before sending
        // requests again.
        const DEFAULT_WAIT_MILLISECONDS: u64 = 3000;
        const MAX_RANDOM_DELAY_MILLISECONDS: u64 = 200;

        let mut retry_after_ms = response
            .headers()
            .get("Retry-After")
            .and_then(|hv| hv.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .map(|sec| sec * 1000)
            .unwrap_or(DEFAULT_WAIT_MILLISECONDS);

        // Add a random delay to the wait time.
        retry_after_ms += rand::thread_rng().gen_range(0..=MAX_RANDOM_DELAY_MILLISECONDS);

        let retry_after = Duration::from_millis(retry_after_ms);

        tokio::time::sleep(retry_after).await;
    }

    async fn extract_notion_error(&self, response: Response) -> NotionApiError {
        let error_model: NotionErrorModel = match response.json().await {
            Ok(model) => model,
            Err(err) => {
                return NotionApiError::Network(err);
            }
        };

        NotionApiError::Notion(error_model.into())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct NotionErrorModel {
    code: String,
    message: String,
}

#[allow(clippy::from_over_into)]
impl Into<NotionError> for NotionErrorModel {
    fn into(self) -> NotionError {
        NotionError {
            code: self.code,
            message: self.message,
        }
    }
}
