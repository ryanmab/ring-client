use std::sync::Arc;

use chrono::DateTime;

use crate::authentication::{AuthenticationError, Credentials};
use crate::client::authentication::Tokens;
use crate::Client;

impl Client {
    /// Login to Ring using a set of credentials.
    ///
    /// These credentials can either be:
    /// * A username and password ([`Credentials::User`])
    /// * A refresh token ([`Credentials::RefreshToken`])
    ///
    /// # Example
    ///
    /// ## Login with a Username and Password
    /// ```no_run
    /// use ring_client::Client;
    ///
    /// use ring_client::authentication::Credentials;
    /// use ring_client::AuthenticationError;
    /// use ring_client::OperatingSystem;
    ///
    /// # tokio_test::block_on(async {
    ///   let client = Client::new("Home Automation", "mock-system-id", OperatingSystem::Ios);
    ///
    ///   let credentials = Credentials::User {
    ///     username: "username".to_string(),
    ///     password: "password".to_string(),
    ///   };
    ///
    ///   let attempt = client.login(credentials).await;
    ///
    ///   if let Err(AuthenticationError::MfaCodeRequired) = attempt {
    ///     // The user needs to enter a 2FA code.
    ///     client.respond_to_challenge("123456").await.expect("Providing a valid 2FA code should not fail");
    ///   }
    ///   else {
    ///     // The login was successful!
    ///   }
    /// # })
    /// ```
    ///
    /// ## Login with a Refresh Token
    ///
    /// If the user has previosuly logged in, Ring will have issued a refresh token. This token
    /// can be used on subsequent login attempts to avoid having to complete the full login flow
    /// (2FA, etc).
    ///
    /// Refresh tokens can be retrieved using [`Client::get_refresh_token`] after a successful
    /// login.
    ///
    /// ```no_run
    /// use ring_client::Client;
    ///
    /// use ring_client::authentication::Credentials;
    /// use ring_client::OperatingSystem;
    ///
    /// # tokio_test::block_on(async {
    ///    let client = Client::new("Home Automation", "mock-system-id", OperatingSystem::Ios);
    ///
    ///    let refresh_token = Credentials::RefreshToken("".to_string());
    ///
    ///    client.login(refresh_token).await.expect("Logging in with a valid refresh token should not fail");
    /// # })
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error logging in was unsuccessful and a Two Factor Authentication (2FA)
    /// challenge was not issued.
    pub async fn login(&self, credentials: Credentials) -> Result<(), AuthenticationError> {
        let mut lock = self.user.write().await;
        let user = lock.insert(credentials);

        match user {
            Credentials::User { username, password } => {
                self.tokens.write().await.replace(Arc::new(
                    self.auth.login(username, password, &self.system_id).await?,
                ));
            }
            Credentials::RefreshToken(ref refresh_token) => {
                self.tokens.write().await.replace(Arc::new(
                    self.auth
                        .refresh_tokens(Arc::new(Tokens::new(
                            String::new(),
                            DateTime::default(),
                            refresh_token.to_string(),
                        )))
                        .await?,
                ));
            }
        };

        self.api
            .set_session(
                &self.display_name,
                &self.system_id,
                &*self
                    .refresh_tokens_if_needed()
                    .await
                    .map_err(|_| AuthenticationError::SessionFailed)?,
            )
            .await
            .map_err(|_| AuthenticationError::SessionFailed)?;

        Ok(())
    }

    /// Respond to a challenge issued by Ring during the authentication process.
    ///
    /// This is typically used to handle Two Factor Authentication (2FA) challenges
    ///
    /// # Errors
    ///
    /// Returns an error if the challenge could not be completed.
    pub async fn respond_to_challenge(&self, code: &str) -> Result<(), AuthenticationError> {
        if let Some(Credentials::User { username, password }) = self.user.read().await.as_ref() {
            self.tokens.write().await.replace(Arc::new(
                self.auth
                    .respond_to_challenge(username, password, &self.system_id, code)
                    .await?,
            ));

            self.api
                .set_session(
                    &self.display_name,
                    &self.system_id,
                    &*self
                        .refresh_tokens_if_needed()
                        .await
                        .map_err(|_| AuthenticationError::SessionFailed)?,
                )
                .await
                .map_err(|_| AuthenticationError::SessionFailed)?;
        }

        Ok(())
    }

    /// Get the refresh token issued by Ring for the current session.
    ///
    /// If [`Credentials::RefreshToken`] was used to login initially, this will return the
    /// same token.
    pub async fn get_refresh_token(&self) -> Option<String> {
        if let Some(refresh_token) = self.tokens.read().await.as_ref() {
            return Some(refresh_token.refresh_token.to_string());
        }

        None
    }
}
