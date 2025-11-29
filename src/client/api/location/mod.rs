mod event;

use crate::client::Client;
use crate::helper;
use crate::helper::url::Url;
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::client::api::error::ApiError;
use crate::client::{api::RingApi, authentication::Tokens};
pub use event::*;

/// A location in a Ring account.
#[derive(Debug)]
pub struct Location<'a> {
    session: &'a Client,

    /// Data about the location.
    pub data: LocationData,
}

impl Location<'_> {
    pub(crate) const fn new(session: &Client, data: LocationData) -> Location<'_> {
        Location { session, data }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Deserialize)]
pub struct LocationAddress {
    pub address1: String,
    pub address2: String,
    pub city: String,
    pub country: String,
    pub cross_street: String,
    pub state: String,
    pub timezone: String,
    pub zip_code: String,
}

#[allow(missing_docs)]
#[derive(Debug, Deserialize)]
pub struct GeoCoordinates {
    pub latitude: f64,
    pub longitude: f64,
}

#[allow(missing_docs)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GeoServiceVerified {
    Verified,
    Unverified,
    AddressCoordinates(String, String),
}

/// Data about a location in a Ring account.
#[derive(Debug, Deserialize)]
pub struct LocationData {
    #[serde(rename = "location_id")]
    pub(crate) id: String,

    /// The address of the location.
    pub address: LocationAddress,

    /// When the location was first created.
    pub created_at: DateTime<Utc>,

    /// Geographic coordinates of the location.
    pub geo_coordinates: GeoCoordinates,

    /// Whether the location's geographic coordinates are verified.
    pub geo_service_verified: GeoServiceVerified,

    /// Whether the location is a job site.
    pub is_jobsite: bool,

    /// Whether the location is being shared from a different account.
    pub is_owner: bool,

    /// The name of the location.
    pub name: String,

    /// The ID of the owner of the location.
    pub owner_id: i32,

    /// When the location was last updated.
    pub updated_at: DateTime<Utc>,

    /// Whether the user is verified.
    pub user_verified: bool,
}

#[derive(Debug, Deserialize)]
struct Response {
    user_locations: Vec<LocationData>,
}

impl RingApi {
    pub(crate) async fn get_location_data(
        &self,
        tokens: &Tokens,
    ) -> Result<Vec<LocationData>, ApiError> {
        Ok(self
            .client
            .get(helper::url::get_base_url(&Url::Locations))
            .header("User-Agent", self.operating_system.get_user_agent())
            .bearer_auth(&tokens.access_token)
            .send()
            .await?
            .json::<Response>()
            .await?
            .user_locations)
    }
}
