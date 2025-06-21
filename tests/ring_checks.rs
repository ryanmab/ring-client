use std::sync::Arc;

use dotenvy_macro::dotenv;
use ring_client::location::Message;
use ring_client::{authentication::Credentials, Client, OperatingSystem};
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

    let received_events = Arc::new(Mutex::new(Vec::new()));

    {
        let received_events = Arc::clone(&received_events);

        let mut listener = location
            .get_listener()
            .await
            .expect("Should be able to listen for events");

        // Wait for a few seconds to receive events from Ring
        let outcome = timeout(
            std::time::Duration::from_secs(30),
            listener.listen(|event, _, _| async {
                let received_events = Arc::clone(&received_events);

                let mut received_events = received_events.lock().await;

                if !matches!(
                    event.message,
                    Message::SessionInfo(_) | Message::SubscriptionTopicsInfo(_)
                ) {
                    // We only expect two kinds of events when opening the listener - either a
                    // subscription topics info event or a session info event.
                    return Err("Received unexpected event type");
                }

                received_events.push(event);

                if received_events.len() >= 2 {
                    // If we have received enough events, close the connection
                    return Ok(false);
                }

                Ok(true)
            }),
        )
        .await;

        assert!(outcome.is_ok(), "Outcome of event listening should be Ok");
    }

    let events = received_events.lock().await;
    assert!(
        events.len() >= 2,
        "Expected at least two events to be received"
    );

    // Check that the events are of the expected type
    assert!(
        events
            .iter()
            .any(|e| matches!(e.message, Message::SessionInfo(_))),
        "Expected at least one SessionInfo event"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e.message, Message::SubscriptionTopicsInfo(_))),
        "Expected at least one SubscriptionTopicsInfo event"
    );
}
