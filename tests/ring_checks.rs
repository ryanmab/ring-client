use std::sync::Arc;

use dotenvy_macro::dotenv;
use ring_client::{authentication::Credentials, location, Client, OperatingSystem};
use tokio::{sync::Mutex, time::timeout};

#[tokio::test]
async fn test_listing_devices() {
    let client = Client::new("Home Automation", "mock-system-id", OperatingSystem::Ios);

    let refresh_token = Credentials::RefreshToken(dotenv!("RING_REFRESH_TOKEN").to_string());

    client
        .login(refresh_token)
        .await
        .expect("Refresh token should always be valid");

    let devices = client
        .get_devices()
        .await
        .expect("Expected to get locations");

    assert!(!devices.is_empty(), "No devices found");
}

#[tokio::test]
async fn test_listening_for_events_in_location() {
    let client = Client::new("Home Automation", "mock-system-id", OperatingSystem::Ios);

    let refresh_token = Credentials::RefreshToken(dotenv!("RING_REFRESH_TOKEN").to_string());

    client
        .login(refresh_token)
        .await
        .expect("Refresh token should always be valid");

    let locations = client
        .get_locations()
        .await
        .expect("Should be able to get locations");

    assert!(!locations.is_empty(), "No locations found");

    let location = locations.first().expect("Expected at least one location");

    let recieved_events = Arc::new(Mutex::new(Vec::new()));

    {
        let recieved_events = Arc::clone(&recieved_events);

        let listener = location
            .listen_for_events(move |event, _| {
                let recieved_events = Arc::clone(&recieved_events);

                async move {
                    recieved_events.lock().await.push(event);
                }
            })
            .await
            .expect("Should be able to listen for events");

        // Wait for a few seconds to receive events from Ring
        let _ = timeout(std::time::Duration::from_secs(10), listener.join()).await;
    }

    let events = recieved_events.lock().await;
    assert!(
        events.len() >= 2,
        "Expected at least two events to be received"
    );

    // Check that the events are of the expected type
    assert!(
        events
            .iter()
            .any(|e| matches!(e.message, crate::location::Message::SessionInfo(_))),
        "Expected at least one SessionInfo event"
    );
    assert!(
        events.iter().any(|e| matches!(
            e.message,
            crate::location::Message::SubscriptionTopicsInfo(_)
        )),
        "Expected at least one SubscriptionTopicsInfo event"
    );
}
