use thiserror::Error;

/// Errors which can occur when trying to authenticate with the Ring API.
#[derive(Error, Debug)]
#[error(transparent)]
#[non_exhaustive]
pub enum AuthenticationError {
    /// The credentials provided were invalid.
    #[error("The credentials provided were invalid")]
    InvalidCredentials,

    /// Ring presented a MFA (Two Factor Authentication) challenge which require
    /// an SMS code to be sent to the user, and provided to the API.
    ///
    /// This typically occurs when logging in with a username and password
    /// ([`crate::authentication::Credentials::User`]).
    ///
    /// You can use [`respond_to_challenge`](crate::client::Client::respond_to_challenge) to
    /// continue the authentication process once the SMS code has been captured.
    #[error("An MFA code is required to complete the authentication process")]
    MfaCodeRequired,

    /// An error occured with the Ring OAuth endpoint.
    #[error("An error occurred while trying to communicate with the Ring OAuth API")]
    OAuthError(#[from] reqwest::Error),

    /// The response from the Ring OAuth API could not be decoded.
    #[error("An error occurred while decoding the response from the Ring OAuth API: {0}")]
    InvalidResponse(#[from] serde_json::Error),

    /// Ring presented a challenge which was not expected, and is currently not supported
    /// by the crate.
    #[error("The presented challenge is not supported. Challenge was: {0}")]
    UnsupportedChallenge(String),

    /// Setting the session details with Ring failed.
    #[error("An error occurred while trying to set the session details with Ring")]
    SessionFailed,
}
