/// Support for Ring devices (such as doorbells and cameras)
pub mod device;
/// Support for Ring locations (such as homes and other properties)
pub mod location;
/// Support for Ring users (such as profile management)
pub mod session;
/// Support for Ring tickets (WebSocket connections)
pub mod ticket;

mod error;

use crate::helper::OperatingSystem;
pub use error::ApiError;

#[derive(Debug)]
pub struct RingApi {
    client: reqwest::Client,
    operating_system: OperatingSystem,
}

impl RingApi {
    pub fn new(operating_system: OperatingSystem) -> Self {
        Self {
            client: reqwest::Client::new(),
            operating_system,
        }
    }
}
