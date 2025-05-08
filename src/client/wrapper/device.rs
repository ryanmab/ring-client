use crate::client::api::device::Device;
use crate::client::api::ApiError;
use crate::Client;

impl Client {
    /// Retrieve a list of devices in the Ring account.
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
    /// // For berevity, a Refresh Token is being used here. However, the client can also
    /// // be authenticated using a username and password.
    /// //
    /// // See `Client::login` for more information.
    /// let refresh_token = Credentials::RefreshToken("".to_string());
    ///
    /// client.login(refresh_token)
    ///      .await
    ///      .expect("Logging in with a valid refresh token should not fail");
    ///
    /// let devices = client.get_devices()
    ///      .await
    ///      .expect("Getting devices not fail");
    ///
    /// println!("{:#?}", devices);
    /// # });
    ///```
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails. This may occur either as the result of an API
    /// error, or if the authentication token needs to be refreshed and it is not successful.
    pub async fn get_devices(&self) -> Result<Vec<Device>, ApiError> {
        self.api
            .get_device_data(
                &*self
                    .refresh_tokens_if_needed()
                    .await
                    .map_err(ApiError::AuthenticationRefreshFailed)?,
            )
            .await
            .map(|data| data.into_iter().map(Device::new).collect())
    }
}
