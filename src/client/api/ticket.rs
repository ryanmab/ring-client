use crate::client::api::error::ApiError;
use crate::client::api::RingApi;
use crate::client::authentication::Tokens;
use crate::helper::url::Url;
use crate::{helper, Client};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(missing_docs)]
pub enum AssetStatus {
    Online,
    Offline,
}

/// An Ring host.
///
/// This is effectively a Ring host which can be connected to for real-time streaming of Location events
/// ([`crate::location::Location`]).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    uuid: String,
    doorbot_id: i64,
    kind: String,
    status: AssetStatus,
    broker_host: String,
    on_battery: bool,
}

/// A ticket for a Ring host.
///
/// This is effectively a session which allows for connections to Ring WebSocket servers.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticket {
    /// The ID of the ticket.
    #[serde(rename = "ticket")]
    pub id: String,

    /// The URI of the host to connect to.
    pub host: String,

    subscription_topics: Vec<String>,
    assets: Vec<Asset>,
}

impl RingApi {
    pub async fn get_ticket(&self, location_id: &str, tokens: &Tokens) -> Result<Ticket, ApiError> {
        let response = self
            .client
            .get(helper::url::get_base_url(&Url::Ticket))
            .query(&[("locationID", location_id)])
            .header("User-Agent", self.operating_system.get_user_agent())
            .bearer_auth(&tokens.access_token)
            .send()
            .await?;

        Ok(response.json::<Ticket>().await?)
    }
}

impl Client {
    pub(crate) async fn get_ticket(&self, location_id: &str) -> Result<Ticket, ApiError> {
        self.api
            .get_ticket(
                location_id,
                &*self
                    .refresh_tokens_if_needed()
                    .await
                    .map_err(ApiError::AuthenticationRefreshFailed)?,
            )
            .await
    }
}
