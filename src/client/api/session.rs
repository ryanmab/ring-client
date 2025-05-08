use crate::client::api::error::ApiError;
use crate::client::api::RingApi;
use crate::client::authentication::Tokens;
use crate::helper::url::Url;
use crate::{constant, helper};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

/// The profile data for the logged in user.
#[derive(Deserialize, Debug)]
pub struct Profile {
    /// The ID of the user.
    pub id: usize,

    /// The email address of the user.
    pub email: String,

    /// The first name of the user.
    pub first_name: String,

    /// The last name of the user.
    pub last_name: String,

    #[serde(flatten)]
    #[allow(missing_docs)]
    pub extra: HashMap<String, Value>,
}

/// An active session
#[derive(Deserialize, Debug)]
#[allow(missing_docs)]
pub struct Session {
    pub profile: Profile,
}

impl RingApi {
    pub async fn set_session(
        &self,
        display_name: &str,
        system_id: &str,
        tokens: &Tokens,
    ) -> Result<Session, ApiError> {
        Ok(self
            .client
            .post(helper::url::get_base_url(&Url::Session))
            .header("User-Agent", self.operating_system.get_user_agent())
            .bearer_auth(&tokens.access_token)
            .json(&json!({
                "device": {
                    "hardware_id": helper::hardware::generate_hardware_id(system_id),
                    "os": self.operating_system.to_string(),
                    "metadata": {
                        "api_version": constant::API_VERSION,
                        "device_model": display_name
                    }
                }
            }))
            .send()
            .await?
            .json::<Session>()
            .await?)
    }
}
