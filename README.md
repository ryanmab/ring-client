[![Coverage](https://api.coveragerobot.com/v1/graph/github/ryanmab/ring-client/badge.svg?token=5444eb416f4cfd4c632e25248d55ebab0e143552813434179b)](https://coveragerobot.com)
[![Crates.io Version](https://img.shields.io/crates/v/ring-client)](https://crates.io/crates/ring-client)
![Crates.io Total Downloads](https://img.shields.io/crates/d/ring-client)
[![docs.rs](https://img.shields.io/docsrs/ring-client)](https://docs.rs/ring-client)
[![Build](https://github.com/ryanmab/ring-client/actions/workflows/build.yml/badge.svg)](https://github.com/ryanmab/ring-client/actions/workflows/build.yml)
![GitHub License](https://img.shields.io/github/license/ryanmab/ring-client)

<!-- cargo-rdme start -->

# Ring Client

The Ring Client crate provides a client for interfacing with [Ring](https://www.ring.com/)
home security devices.

## Usage
```toml
[dependencies]
ring-client = "0.1.0"
```

## Capabilities

1. Authenticate with Ring - either via Username and Password, or Refresh Tokens.
2. Interact with Ring locations - including listening for events (such as motion detectors) in
   real-time, as well as changing the states of devices (such as enabling or disabling an Alarm system).
3. Retrieve profile information.

## Examples

More in-depth examples can be found in documentation comments on the Client methods.

### Listening for Events

Perhaps one of the most useful features of the crate is the ability to listen and respond to
events which occur in a location in real-time.

This is done using the [`Listener::listen`] method.

```rust
use ring_client::Client;

use ring_client::authentication::Credentials;
use ring_client::OperatingSystem;

let client = Client::new("Home Automation", "mock-system-id", OperatingSystem::Ios);

// For brevity, a Refresh Token is being used here. However, the client can also
// be authenticated using a username and password.
//
// See `Client::login` for more information.
let refresh_token = Credentials::RefreshToken("".to_string());

client.login(refresh_token)
     .await
     .expect("Logging in with a valid refresh token should not fail");

let locations = client.get_locations()
     .await
     .expect("Getting locations should not fail");

let location = locations
     .first()
     .expect("There should be at least one location");

let mut listener = location.get_listener()
     .await
     .expect("Creating a listener should not fail");

// Listen for events in the location and react to them using the provided closure.
listener.listen(|event, _, _| async move {
    // Connection can be used to send commands to the Ring API.
    println!("New event: {:#?}", event);

    // The connection argument can be used to send events back to Ring in
    // response to the event.

    // Return true or false to indicate whether the listener should continue listening, or
    // whether the promise should be resolved.
    true
})
.await;

```

### Sending Events

The [`Listener`] can also be used to send events to the Ring API, such as arming or disarming an alarm
system.

```rust
use serde_json::json;
use ring_client::Client;

use ring_client::authentication::Credentials;
use ring_client::location::{Event, Message};
use ring_client::OperatingSystem;

let client = Client::new("Home Automation", "mock-system-id", OperatingSystem::Ios);

// For brevity, a Refresh Token is being used here. However, the client can also
// be authenticated using a username and password.
//
// See `Client::login` for more information.
let refresh_token = Credentials::RefreshToken("".to_string());

client.login(refresh_token)
     .await
     .expect("Logging in with a valid refresh token should not fail");

let locations = client.get_locations()
     .await
     .expect("Getting locations should not fail");

let location = locations
     .first()
     .expect("There should be at least one location");

location.get_listener()
    .await
    .expect("Creating a listener should not fail")
    .send(
        Event::new(
            Message::DataUpdate(json!({}))
        )
    )
    .await
    .expect("Sending an event should not fail");
```

### Listing Devices

```rust
use ring_client::Client;

use ring_client::authentication::Credentials;
use ring_client::OperatingSystem;

let client = Client::new("Home Automation", "mock-system-id", OperatingSystem::Ios);

// For brevity, a Refresh Token is being used here. However, the client can also
// be authenticated using a username and password.
//
// See `Client::login` for more information.
let refresh_token = Credentials::RefreshToken("".to_string());

client.login(refresh_token)
     .await
     .expect("Logging in with a valid refresh token should not fail");

let devices = client.get_devices()
     .await
     .expect("Getting devices not fail");

println!("{:#?}", devices);
```

## Contributing

There are _tons_ of features which could be added to the crate. If you'd like to contribute, please
feel free to open an issue or a pull request.

Examples of features which could be added:
1. Better parity between the Ring API and the structs.
2. Support for streaming video from Ring cameras and doorbells.

### Testing

Many of the tests require a valid Ring account before they can be run, which can be provided
via a Refresh Token being set in the `.env` file.

The `.env` file can be created by using `.env.example` as a template:
```sh
cp .env.example .env
```
#### Running tests
The tests can be run with:
```sh
cargo test
```

<!-- cargo-rdme end -->
