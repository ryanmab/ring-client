use crate::helper::url::Url;
use crate::location::Location;
use crate::{helper, ApiError};
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::future::Future;
use std::sync::{Arc, Weak};
use tokio::net::TcpStream;
use tokio::sync::oneshot::Sender;
use tokio::{
    sync::{oneshot, Mutex},
    task::JoinHandle,
};
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

/// A sink for sending events to Ring.
///
/// For example, to enable an Alarm system.
#[derive(Debug)]
pub struct Sink {
    sink: Weak<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>>>,
}

impl Sink {
    #[must_use]
    pub(crate) const fn new(
        sink: Weak<
            Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>>,
        >,
    ) -> Self {
        Self { sink }
    }

    /// Sends a message to Ring immediately (no buffering).
    ///
    /// # Errors
    ///
    /// Returns an error if the sink has already been closed.
    pub async fn send(&self, message: Message) -> Result<(), ApiError> {
        match self.sink.upgrade() {
            Some(sink) => {
                let mut sink = sink.lock().await;
                sink.send(Event { message }.try_into()?).await?;

                Ok(())
            }
            None => Err(ApiError::SinkAlreadyClosed),
        }
    }
}

/// An event listener for a Location.
#[derive(Debug)]
pub struct Listener<'a> {
    location: &'a Location<'a>,

    handle: Option<(JoinHandle<()>, Sender<()>)>,
    sink: Option<Sink>,
}

impl Listener<'_> {
    /// Create a brand new event listener for a location.
    ///
    /// This generally accepts a callback defined by the caller, which is triggered whenever an
    /// event is triggered by Ring.
    #[must_use]
    pub fn new<'a, EventHandler, EventHandlerFut>(
        location: &'a Location<'_>,
        stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        on_event: EventHandler,
    ) -> Listener<'a>
    where
        EventHandler: Fn(Event, Arc<Sink>) -> EventHandlerFut + Send + Sync + 'static,
        EventHandlerFut: Future<Output = ()> + Send + 'static,
    {
        let mut listener = Listener {
            location,
            handle: None,
            sink: None,
        };

        listener.spawn(stream, on_event);

        listener
    }

    fn spawn<EventHandler, EventHandlerFut>(
        &mut self,
        stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        on_event: EventHandler,
    ) where
        EventHandler: Fn(Event, Arc<Sink>) -> EventHandlerFut + Send + Sync + 'static,
        EventHandlerFut: Future<Output = ()> + Send + 'static,
    {
        let (tx, rx) = oneshot::channel::<()>();

        let (write, mut read) = stream.split();
        let write = Arc::new(Mutex::new(write));

        self.sink = Some(Sink::new(Arc::downgrade(&write)));

        let handle = tokio::spawn(async move {
            let sink = Arc::new(Sink::new(Arc::downgrade(&write)));

            tokio::select! {
                () = async {
                    while let Some(message) = read.next().await {
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

                                let Ok(event) = serde_json::from_str::<Event>(&message.to_string()) else {
                                    log::error!("Error parsing event: {:?}", &message);
                                    continue;
                                };

                                if event.message == Message::Unknown {
                                    log::warn!("Unknown message received: {:?}", message);
                                    continue;
                                }

                                on_event(event, Arc::clone(&sink)).await;
                            }
                            Err(error) => {
                                log::error!("Error receiving message: {:?}", error);
                            }
                        }
                    }
                } => {
                    log::info!("Event listener finished.");
                }
                _ = rx => {
                    let stream = Arc::try_unwrap(write).unwrap().into_inner().reunite(read);

                    match stream {
                        Ok(mut stream) => {
                            let closed = stream.close(None).await;

                            if let Err(error) = closed {
                                log::error!("Error closing stream: {:?}", error);
                                return;
                            }

                            log::info!("Shut down event listener");
                        },
                        Err(_) => {
                            log::info!("Unable to reunite write and read pair into stream");
                        }
                    }
                }
            }
        });

        self.handle = Some((handle, tx));
    }

    /// Block until the stream to be closed.
    pub async fn join(self) {
        if let Some((handle, _)) = self.handle {
            let _ = handle.await.map_err(|_| {
                log::error!(
                    "Error joining event listener for location: {:?}",
                    self.location.data.id
                );
            });
        }
    }

    /// Terminate the event listener.
    pub fn terminate(mut self) {
        log::info!(
            "Sending terminate signal to event listener for location: {:?}",
            self.location.data.id
        );

        if let Some((_, tx)) = self.handle.take() {
            log::info!(
                "Sending shutdown signal to event listener for location: {:?}",
                self.location.data.id
            );

            let _ = tx.send(());
        } else {
            log::info!(
                "No task ongoing listening for events for location: {:?}",
                self.location.data.id
            );
        }
    }
}

impl Location<'_> {
    /// Listen for events in a location.
    ///
    /// # Errors
    ///
    /// Will return an error if a connection cannot be established with Ring.
    pub async fn listen_for_events<EventHandler, EventHandlerFut>(
        &self,
        on_event: EventHandler,
    ) -> Result<Listener<'_>, ApiError>
    where
        EventHandler: Fn(Event, Arc<Sink>) -> EventHandlerFut + Send + Sync + 'static,
        EventHandlerFut: Future<Output = ()> + Send + 'static,
    {
        let (stream, _) = self.connect().await?;

        Ok(Listener::new(self, stream, on_event))
    }

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
