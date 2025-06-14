use crate::helper::url::Url;
use crate::location::Location;
use crate::{helper, ApiError};
use futures_util::stream::SplitStream;
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::future::Future;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Utf8Bytes;
use tokio_tungstenite::{connect_async, tungstenite, MaybeTlsStream, WebSocketStream};

/// A real-time event which occured in a Location.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Event {
    /// The content of the event.
    #[serde(rename = "msg")]
    pub message: Message,
}

impl Event {
    /// Create a new event with the given message.
    #[must_use]
    pub const fn new(message: Message) -> Self {
        Self { message }
    }
}

/// A message sent to or from Ring via WebSocket.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "msg")]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum Message {
    SubscriptionTopicsInfo(serde_json::Value),

    DeviceInfoSet(serde_json::Value),

    SessionInfo(serde_json::Value),

    DataUpdate(serde_json::Value),

    /// A message which is yet to be mapped by the crate.
    #[serde(other)]
    Unknown,
}

impl TryFrom<Event> for tungstenite::protocol::Message {
    type Error = serde_json::Error;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        Ok(Self::Text(Utf8Bytes::from(&serde_json::to_string(&event)?)))
    }
}

/// A live connection for exchanging messages with Ring.
///
/// For example, to enable an Alarm system.
#[derive(Debug)]
pub struct Connection {
    /// The read portion of the WebSocket stream.
    stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,

    /// The write portion of the WebSocket stream.
    sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>,
}

impl Connection {
    #[must_use]
    pub(crate) fn new(stream: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
        let (sink, stream) = stream.split();

        Self { stream, sink }
    }

    /// Reads the next message from the stream.
    #[must_use]
    pub async fn next(&mut self) -> Option<Result<Event, ApiError>> {
        while let Some(message) = self.stream.next().await {
            match message {
                Ok(message) => {
                    if let tungstenite::protocol::Message::Ping(_) = message {
                        // We can safetly ignore ping messages as Tungstenite will
                        // handle the Pong response for us.
                        //
                        // https://docs.rs/tungstenite/latest/tungstenite/protocol/struct.WebSocket.html#method.write
                        log::debug!("Recieved ping message from Ring");

                        continue;
                    }

                    let event = serde_json::from_str::<Event>(&message.to_string())
                        .map_err(ApiError::InvalidResponse);

                    if let Err(error) = event {
                        log::error!("Error deserializing message: {:?}", error);

                        return Some(Err(error));
                    }

                    log::debug!("Received event: {:?}", event);

                    return Some(event);
                }
                Err(error) => {
                    log::error!("Error receiving message: {:?}", error);

                    return Some(Err(ApiError::WebsocketError(error)));
                }
            }
        }

        None
    }

    /// Sends a message to Ring immediately (no buffering).
    ///
    /// # Errors
    ///
    /// Returns an error if the sink has already been closed.
    pub async fn send(&mut self, event: Event) -> Result<(), ApiError> {
        self.sink
            .send(event.try_into()?)
            .await
            .map_err(ApiError::WebsocketError)
    }

    /// Closes the connection to Ring gracefully.
    pub async fn close(self) {
        let stream = self.stream.reunite(self.sink);

        match stream {
            Ok(mut stream) => {
                let closed = stream.close(None).await;

                if let Err(error) = closed {
                    log::error!("Error closing stream: {:?}", error);
                    return;
                }

                log::info!("Shut down Websocket connection gracefully");
            }
            Err(_) => {
                log::info!("Unable to reunite write and read pair into stream");
            }
        }
    }
}

/// An event listener for a Location.
#[derive(Debug)]
pub struct Listener<'a> {
    location: &'a Location<'a>,
    connection: Connection,
}

impl<'a> Listener<'a> {
    /// Create a brand new event listener for a location.
    ///
    /// This generally accepts a callback defined by the caller, which is triggered whenever an
    /// event is triggered by Ring.
    #[must_use]
    pub fn new<'b>(
        location: &'b Location<'_>,
        stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    ) -> Listener<'b> {
        Listener {
            location,
            connection: Connection::new(stream),
        }
    }

    /// Listen for events in a particular location.
    ///
    /// # Example
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
    /// // For brevity, a Refresh Token is being used here. However, the client can also
    /// // be authenticated using a username and password.
    /// //
    /// // See `Client::login` for more information.
    /// let refresh_token = Credentials::RefreshToken("".to_string());
    ///
    /// client.login(refresh_token)
    ///      .await
    ///      .expect("Logging in with a valid refresh token should not fail");
    ///
    /// let locations = client.get_locations()
    ///      .await
    ///      .expect("Getting locations should not fail");
    ///
    /// let location = locations
    ///      .first()
    ///      .expect("There should be at least one location");
    ///
    /// let mut listener = location.get_listener()
    ///      .await
    ///      .expect("Creating a listener should not fail");
    ///
    /// // Listen for events in the location and react to them using the provided closure.
    /// listener.listen(|event, location, mut connection| async move {
    ///     // Connection can be used to send commands to the Ring API.
    ///     println!("New event: {:#?}", event);
    ///
    ///     // The connection argument can be used to send events back to Ring in
    ///     // response to the event.
    ///
    ///     // Return true or false to indicate whether the listener should continue listening
    ///     true
    /// })
    /// .await;
    ///
    /// # });
    ///```
    pub async fn listen<EventHandler, EventHandlerFut>(&'a mut self, on_event: EventHandler)
    where
        EventHandler:
            Fn(Event, &'a Location<'a>, Arc<Mutex<&'a mut Connection>>) -> EventHandlerFut,
        EventHandlerFut: Future<Output = bool>,
    {
        let connection = Arc::new(Mutex::new(&mut self.connection));

        loop {
            // Wait for the next event from the connection and then drop the lock
            // to allow any on_event calls to use the connection without blocking
            let event = { connection.lock().await.next().await };

            match event {
                Some(Ok(event)) => {
                    if event.message == Message::Unknown {
                        log::warn!("Unknown message received: {:?}", event.message);
                        continue;
                    }

                    log::debug!("Received event: {:?}", event);

                    let outcome = on_event(event, self.location, Arc::clone(&connection)).await;

                    if !outcome {
                        log::debug!("Event handler returned false, stopping listener");
                        break;
                    }
                }
                Some(Err(error)) => {
                    log::error!("Error receiving event: {:?}", error);
                    continue;
                }
                None => {
                    log::info!("Websocket stream closed, stopping listener");
                    break;
                }
            };
        }
    }

    /// Send an event to Ring.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection is closed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use serde_json::json;
    /// use ring_client::Client;
    ///
    /// use ring_client::authentication::Credentials;
    /// use ring_client::location::{Event, Message};
    /// use ring_client::OperatingSystem;
    ///
    /// # tokio_test::block_on(async {
    /// let client = Client::new("Home Automation", "mock-system-id", OperatingSystem::Ios);
    ///
    /// // For brevity, a Refresh Token is being used here. However, the client can also
    /// // be authenticated using a username and password.
    /// //
    /// // See `Client::login` for more information.
    /// let refresh_token = Credentials::RefreshToken("".to_string());
    ///
    /// client.login(refresh_token)
    ///      .await
    ///      .expect("Logging in with a valid refresh token should not fail");
    ///
    /// let locations = client.get_locations()
    ///      .await
    ///      .expect("Getting locations should not fail");
    ///
    /// let location = locations
    ///      .first()
    ///      .expect("There should be at least one location");
    ///
    /// let listener = location.get_listener().await;
    ///
    /// location.get_listener()
    ///     .await
    ///     .expect("Creating a listener should not fail")
    ///     .send(
    ///         Event::new(
    ///             Message::DataUpdate(json!({}))
    ///         )
    ///     )
    ///     .await
    ///     .expect("Sending an event should not fail");
    /// # });
    ///```
    pub async fn send(&mut self, event: Event) -> Result<(), ApiError> {
        self.connection.send(event).await
    }

    /// Close the underlying connection to Ring.
    pub async fn close(self) {
        self.connection.close().await;
    }
}

impl<'a> Location<'a> {
    /// Get a listener for events in a location.
    ///
    /// # Errors
    ///
    /// Will return an error if a connection cannot be established with Ring.
    pub async fn get_listener(&'a self) -> Result<Listener<'a>, ApiError> {
        let (stream, _) = self.connect().await?;

        Ok(Listener::new(self, stream))
    }

    /// Generate a ticket (credentials and URI for a Ring Websocket server) and connect to it.
    async fn connect(
        &self,
    ) -> Result<
        (
            WebSocketStream<MaybeTlsStream<TcpStream>>,
            tungstenite::handshake::client::Response,
        ),
        ApiError,
    > {
        let ticket = self.session.get_ticket(&self.data.id).await?;

        let request = helper::url::get_base_url(&Url::Websocket {
            host: &ticket.host,
            auth_code: &ticket.id,
        })
        .into_client_request()?;

        Ok(connect_async(request).await?)
    }
}
