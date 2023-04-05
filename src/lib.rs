//
// Copyright (C) 2017 Kubos Corporation
// Copyright (C) 2022 CUAVA
//
// Licensed under the Apache License, Version 2.0 (the "License")
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// 
// 
// 2022 rewritten for CubeOS
// 
// Contributed by: Patrick Oppel (patrick.oppel94@gmail.com)
// 

// #![deny(missing_docs)]
// #![deny(warnings)]

//! A collection of structures and functions used to create hardware apis, services and apps
//! in the Cube-OS Linux ecosystem.
//!
//! # Use
//!
//! The basic use of the cubeos_service crate is through the Service structure.
//! This structure provides an interface for creating a new service instance. 
//! The service instance is chosen with the --features flag at build. 
//! The default service enables the UDP handling of the service on the satellite.
//! The ground feature creates a CLI to communicate with the satellite from a ground-station.
//! The app feature is used by Apps to enable UDP communication between the App and a service.
//! 
//! The service instance also provides a starting entry point and basic configuration
//! file parsing.
//!
//! ## In Services
//!
//! Services should only link to the `cubeos_service` crate if they have a
//! hardware device they want to expose over the UDP service interface.
//!
//! ## Configuration
//!
//! Services which use this crate have the option of using a local configuration file
//! or falling back on default config values. The service will search for the configuration
//! file at this location `/etc/cubeos-config.toml` unless otherwise specified with
//! the `-c` flag at run time.
//!
//! The service configuration file uses the Toml format and is expected to use the
//! following layout:
//!
//! ```toml,ignore
//! [service-name]
//! config-key = "value"
//! config-key2 = 123
//!
//! # This section and values are needed for all services
//! [service-name.addr]
//! ip = "127.0.0.1"
//! port = 8082
//! ```
//!
//! The `[service-name.addr]` section is required for all services and is used to set
//! the ip/port on which the service will listen for messages. Any service specific
//! configuration values can be specified directly under the `[service-name]` section.
//! Note - the `service-name` used in the sections must match the name used when creating
//! the `Config` instance inside your service.
//!
//! ### Examples
//!
//! # Creating and starting a simple service.
//!
//! ```rust,ignore
//! use cubeos_service::{Config, Service};
//! use model::Subsystem;
//!
//! Service::new(
//!     Config::new("service-name").unwrap(),
//!     Subsystem::new(),
//!     Some(Arc::new(udp_handler)),
//! ).start();
//! ```
//!
//! # Using the service config info to configure the subsystem.
//!
//! ```rust,ignore
//! use cubeos_service::{Config, Service};
//! use model::Subsystem;
//!
//! let config = Config::new("example-service").unwrap();
//! let subsystem = Subsystem { bus = config["bus"] ) };
//! Service::new(
//!     config,
//!     subsystem,
//!     Some(Arc::new(udp_handler)),
//! ).start();
//! ```
//!
//! # Running a service with the default config file (`/etc/cubeos-config.toml`).
//!
//! ```bash
//! $ ./example-service
//! ```
//!
//! # Running a service with a custom config file.
//!
//! ```bash
//! $ ./example-service -c config.toml
//! ```
//! 
//! 
pub use ::bincode;
pub use ::serde_json;
pub use ::rust_udp;
pub use ::serde;
pub use ::std::convert;
pub use ::variant_count;

#[cfg(feature = "ground")]
pub use ::dialoguer;
#[cfg(feature = "ground")]
pub use ::ground_handle;
#[cfg(feature = "ground")]
pub use ::strum;
#[cfg(feature = "ground")]
pub use ::strum_macros;

#[cfg(feature = "app")]
pub use ::lazy_static::lazy_static;

#[cfg(feature = "app")]
mod app;
#[cfg(feature = "default")]
mod service;

mod command;
mod last;
mod ping;
mod error;

pub use crate::error::{Error,Result};
pub use crate::ping::*;
pub use crate::last::*;
#[cfg(feature = "default")]
pub use crate::service::*;
#[cfg(feature = "app")]
pub use crate::app::*;
pub use crate::command::Command;
pub use kubos_system::logger as Logger;
pub use kubos_system::Config;
