#![deny(missing_docs)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(missing_debug_implementations, rust_2018_idioms, rustdoc::all)]
#![allow(rustdoc::private_doc_tests)]
#![forbid(unsafe_code)]
#![deny(clippy::clone_on_ref_ptr)]

//! # Ring Client
//!
//! The Ring Client crate provides a client for interfacing with [Ring](https://www.ring.com/)
//! home security devices.
//!
//! ## Usage
//! ```toml
//! [dependencies]
//! ring-client = "0.0.2"
//! ```
//!
//! ## Capabilities
//!
//! 1. Authenticate with Ring - either via Username and Password, or Refresh Tokens.
//! 2. Interact with Ring locations - including listening for events (such as motion detectors) in
//!    real-time, as well as changing the states of devices (such as enabling or disabling an Alarm system).
//! 3. Retrieve profile information.
//!
//! ## Examples
//!
//! More in-depth examples can be found in documentation comments on the Client methods.
//!
//! ## Listening for Events
//!
//! Perhaps one of the most useful features of the crate is the ability to listen and respond to
//! events which occur in a location in real-time.
//!
//! This is done using the [`crate::location::Location::listen_for_events`] method.
//!
//! ```no_run
//! use ring_client::Client;
//!
//! use ring_client::authentication::Credentials;
//! use ring_client::OperatingSystem;
//!
//! # tokio_test::block_on(async {
//! let client = Client::new("Home Automation", "mock-system-id", OperatingSystem::Ios);
//!
//! // For berevity, a Refresh Token is being used here. However, the client can also
//! // be authenticated using a username and password.
//! //
//! // See `Client::login` for more information.
//! let refresh_token = Credentials::RefreshToken("".to_string());
//!
//! client.login(refresh_token)
//!      .await
//!      .expect("Logging in with a valid refresh token should not fail");
//!
//! let locations = client.get_locations()
//!      .await
//!      .expect("Getting locations should not fail");
//!
//! let location = locations
//!      .first()
//!      .expect("There should be at least one location");
//!
//! let listener = location.listen_for_events(|event, sink| async move {
//!     // The sink can be used to send events to Ring.
//!     println!("New event: {:#?}", event);
//! })
//! .await
//! .expect("Creating a listener should not fail");
//!
//! // Wait for the listener to finish.
//! listener
//!     .join()
//!     .await
//! # });
//!```
//!
//! ## Listing Devices
//!
//! ```no_run
//! use ring_client::Client;
//!
//! use ring_client::authentication::Credentials;
//! use ring_client::OperatingSystem;
//!
//! # tokio_test::block_on(async {
//! let client = Client::new("Home Automation", "mock-system-id", OperatingSystem::Ios);
//!
//! // For berevity, a Refresh Token is being used here. However, the client can also
//! // be authenticated using a username and password.
//! //
//! // See `Client::login` for more information.
//! let refresh_token = Credentials::RefreshToken("".to_string());
//!
//! client.login(refresh_token)
//!      .await
//!      .expect("Logging in with a valid refresh token should not fail");
//!
//! let devices = client.get_devices()
//!      .await
//!      .expect("Getting devices not fail");
//!
//! println!("{:#?}", devices);
//! # });
//!```
//!
//! ## Contributing
//!
//! There are _tons_ of features which could be added to the crate. If you'd like to contribute, please
//! feel free to open an issue or a pull request.
//!
//! Examples of features which could be added:
//! 1. Better parity between the Ring API and the structs.
//! 2. Support for streaming video from Ring cameras and doorbells.
//!
//! ### Testing
//!
//! Many of the tests require a valid Ring account before they can be run, which can be provided
//! via a Refresh Token being set in the `.env` file.
//!
//! The `.env` file can be created by using `.env.example` as a template:
//! ```sh
//! cp .env.example .env
//! ```

//! #### Running tests

//! The tests can be run with:
//! ```sh
//! cargo test
//! ```

mod client;
mod constant;
mod helper;

pub use client::*;

#[doc(hidden)]
pub use helper::OperatingSystem;
