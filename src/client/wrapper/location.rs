use crate::client::api;
use crate::client::location::Location;
use crate::Client;

impl Client {
    /// Retrieve a list of locations in the Ring account.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ring_client::Client;
    ///
    /// use ring_client::authentication::Credentials;
    /// use ring_client::OperatingSystem;
    ///
    /// # tokio_test::block_on(async {
    /// let client = Client::new("Home Automation", "mock-system-id", OperatingSystem::Ios);
    ///
    /// // For brevity, a Refresh Token is being used here. However, the client can also
    /// // be authenticated using a username and password.
    /// //
    /// // See `Client::login` for more information.
    /// let refresh_token = Credentials::RefreshToken("".to_string());
    ///
    /// client.login(refresh_token)
    ///      .await
    ///      .expect("Logging in with a valid refresh token should not fail");
    ///
    /// let locations = client.get_locations()
    ///      .await
    ///      .expect("Getting locations should not fail");
    ///
    /// let location = locations
    ///      .first()
    ///      .expect("There should be at least one location");
    /// # });
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails. This may occur either as the result of an API
    /// error, or if the authentication token needs to be refreshed and it is not successful.
    pub async fn get_locations(&self) -> Result<Vec<Location<'_>>, api::ApiError> {
        Ok(self
            .api
            .get_location_data(
                &*self
                    .refresh_tokens_if_needed()
                    .await
                    .map_err(api::ApiError::AuthenticationRefreshFailed)?,
            )
            .await?
            .into_iter()
            .map(|data| Location::new(self, data))
            .collect())
    }
}
