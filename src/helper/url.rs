const CLIENT_API_BASE_URL: &str = "https://api.ring.com/clients_api";
const DEVICE_API_BASE_URL: &str = "https://api.ring.com/devices/v1";
const OAUTH_BASE_URL: &str = "https://oauth.ring.com/oauth/token";
const APP_API_BASE_URL: &str = "https://prd-api-us.prd.rings.solutions/api/v1";

/// A supported route for the Ring API.
pub enum Url<'a> {
    Oauth,
    Session,
    Devices,
    Locations,
    Ticket,
    Websocket { host: &'a str, auth_code: &'a str },
}

/// Get a base URL for a given route.
pub fn get_base_url(url: &Url<'_>) -> String {
    match url {
        Url::Oauth => OAUTH_BASE_URL.into(),
        Url::Session => format!("{CLIENT_API_BASE_URL}/session"),
        Url::Devices => format!("{CLIENT_API_BASE_URL}/ring_devices"),
        Url::Locations => format!("{DEVICE_API_BASE_URL}/locations"),
        Url::Ticket => format!("{APP_API_BASE_URL}/clap/tickets"),
        Url::Websocket { host, auth_code } => {
            format!("wss://{host}/ws?authcode={auth_code}&ack=false&transport=websocket",)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_base_url() {
        assert_eq!(get_base_url(&Url::Oauth), OAUTH_BASE_URL);
        assert_eq!(
            get_base_url(&Url::Session),
            format!("https://api.ring.com/clients_api/session")
        );
        assert_eq!(
            get_base_url(&Url::Devices),
            format!("https://api.ring.com/clients_api/ring_devices")
        );
        assert_eq!(
            get_base_url(&Url::Locations),
            format!("https://api.ring.com/devices/v1/locations")
        );
        assert_eq!(
            get_base_url(&Url::Ticket),
            format!("https://prd-api-us.prd.rings.solutions/api/v1/clap/tickets")
        );
        assert_eq!(
            get_base_url(&Url::Websocket {
                host: "example.com",
                auth_code: "12345"
            }),
            "wss://example.com/ws?authcode=12345&ack=false&transport=websocket"
        );
    }
}
