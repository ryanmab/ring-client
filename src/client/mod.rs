use crate::client::api::RingApi;
use crate::client::authentication::{Credentials, RingAuth, Tokens};
use std::sync::Arc;
use tokio::sync::RwLock;

mod api;
mod wrapper;

/// Support for the Ring Authentication flow.
pub mod authentication;

pub use api::device;
pub use api::location;
pub use api::session;
pub use api::ticket;

pub use api::ApiError;
pub use authentication::AuthenticationError;

/// Client used to authenticate and interact with Ring.
#[derive(Debug)]
pub struct Client {
    user: RwLock<Option<Credentials>>,
    tokens: RwLock<Option<Arc<Tokens>>>,
    auth: RingAuth,
    api: RingApi,
    display_name: String,
    system_id: String,
}

impl Client {
    /// Create a new client.
    ///
    /// The system ID is used by Ring to identify the client on subsequent logins, and should be
    /// predictable and consistent per device.
    #[must_use]
    pub fn new(
        display_name: &str,
        system_id: &str,
        operating_system: crate::helper::OperatingSystem,
    ) -> Self {
        Self {
            user: RwLock::new(None),
            tokens: RwLock::new(None),
            auth: RingAuth::new(operating_system),
            api: RingApi::new(operating_system),
            display_name: display_name.to_string(),
            system_id: system_id.to_string(),
        }
    }
}
