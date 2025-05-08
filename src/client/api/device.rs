use crate::client::api::error::ApiError;
use crate::client::api::RingApi;
use crate::client::authentication::Tokens;
use crate::helper;
use crate::helper::url::Url;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Formatter};

/// Data about a device in a Ring account.
#[derive(Deserialize, Debug)]
#[serde(tag = "kind")]
#[serde(rename_all = "snake_case")]
pub enum DeviceData {
    /// A Ring Security Camera.
    ///
    /// For example, a [Stick Up Camera](https://en-uk.ring.com/products/stick-up-security-camera-plugin).
    CocoaCamera {
        /// The ID of the device.
        id: usize,

        /// The ID of the location the device is in.
        ///
        /// See [`crate::location::Location`]
        location_id: String,

        /// The description of the device.
        description: String,

        #[serde(flatten)]
        #[allow(missing_docs)]
        extra: HashMap<String, Value>,
    },
    /// A Ring Doorbell.
    ///
    /// For example, a [Video Doorbell Pro 2](https://en-uk.ring.com/products/video-doorbell-pro-2).
    DoorbellGrahamCracker {
        /// The ID of the device.
        id: usize,

        /// The ID of the location the device is in.
        location_id: String,

        /// The description of the device.
        description: String,

        #[serde(flatten)]
        #[allow(missing_docs)]
        extra: HashMap<String, Value>,
    },
    /// A Ring Alarm Base Station.
    ///
    /// They are available as part of the [Alarm Pack](https://en-uk.ring.com/products/alarm-security-5-piece-kit-gen-2).
    BaseStationV1 {
        /// The ID of the device.
        id: usize,

        /// The ID of the location the device is in.
        location_id: String,

        /// The description of the device.
        description: String,

        #[serde(flatten)]
        #[allow(missing_docs)]
        extra: HashMap<String, Value>,
    },

    /// A device which is yet to be mapped by the crate.
    #[serde(other)]
    Other,
}

/// A Device which is enabled in a Ring account.
pub struct Device {
    #[allow(missing_docs)]
    pub data: DeviceData,
}

impl Debug for Device {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Device").field("data", &self.data).finish()
    }
}

impl Device {
    pub(crate) const fn new(data: DeviceData) -> Self {
        Self { data }
    }
}

#[derive(Deserialize)]
struct Response {
    doorbots: Vec<DeviceData>,
    authorized_doorbots: Vec<DeviceData>,
    chimes: Vec<DeviceData>,
    stickup_cams: Vec<DeviceData>,
    base_stations: Vec<DeviceData>,
    beams: Vec<DeviceData>,
    beams_bridges: Vec<DeviceData>,
    other: Vec<DeviceData>,
}

impl RingApi {
    pub async fn get_device_data(&self, tokens: &Tokens) -> Result<Vec<DeviceData>, ApiError> {
        let response = self
            .client
            .get(helper::url::get_base_url(&Url::Devices))
            .header("User-Agent", self.operating_system.get_user_agent())
            .bearer_auth(&tokens.access_token)
            .send()
            .await?
            .json::<Response>()
            .await?;

        Ok(Vec::new()
            .into_iter()
            .chain(response.doorbots)
            .chain(response.authorized_doorbots)
            .chain(response.chimes)
            .chain(response.stickup_cams)
            .chain(response.base_stations)
            .chain(response.beams)
            .chain(response.beams_bridges)
            .chain(response.other)
            .collect())
    }
}
