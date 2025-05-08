use thiserror::Error;

/// Errors which can occur when trying to communicate with the Ring API.
///
/// This may either be the REST API or the WebSocket API.
#[derive(Error, Debug)]
#[error(transparent)]
#[non_exhaustive]
pub enum ApiError {
    /// When communicating with Ring over a REST API an error occurred.
    #[error("An error occurred while trying to communicate with the Ring OAuth API")]
    RequestError(#[from] reqwest::Error),

    /// Decoding the response from a Ring endpoint failed.
    #[error("An error occurred while decoding the response from the Ring API: {0}")]
    InvalidResponse(#[from] serde_json::Error),

    /// When communicating with Ring over a WebSocket connection an error occurred.
    #[error("An error occurred while trying to communicate with the Ring API: {0}")]
    WebsocketError(#[from] tokio_tungstenite::tungstenite::Error),

    /// When refreshing the authentication tokens an error occurred.
    #[error("An error occurred while trying to refresh the authentication tokens")]
    AuthenticationRefreshFailed(crate::client::authentication::AuthenticationError),

    /// An attempt to write to a closed WebSocket sink was made.
    #[error("An error occurred while sending a message")]
    SinkAlreadyClosed,
}
