//
// Copyright (C) 2018 Kubos Corporation
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
// 2022 rewritten for Cube-OS
// 
// Contributed by: Patrick Oppel (patrick.oppel94@gmail.com)
// 

use kubos_system::Config;
use log::info;
use std::collections::HashMap;
use std::net::{SocketAddr,UdpSocket};
use std::sync::{Arc, RwLock};
use std::str::FromStr;
use crate::error::*;
use udp_rs::Message;
use log::*;
use std::thread;

/// Type definition for a CLI tool
pub type OutputFn = dyn Fn(String, SocketAddr)->String + std::marker::Send + std::marker::Sync + 'static;

// Struct that enables passthrough of translated GraphQL inputs
// to UDP service on satellite
#[derive(Clone)]
pub struct UdpPassthrough {
    pub socket: SocketAddr,
    pub to: SocketAddr,
}
impl UdpPassthrough {
    pub fn new(bind: String, target: String) -> Self {
        let socket = SocketAddr::from_str(bind.as_str()).unwrap();
        let to = SocketAddr::from_str(target.as_str()).unwrap();

        Self {
            socket,
            to,
        }
    }
}

/// Context struct used by a service to provide Juniper context,
/// subsystem access and persistent storage.
#[derive(Clone)]
pub struct Context {
    ///
    pub storage: Arc<RwLock<HashMap<String, String>>>,
    ///
    pub udp_pass: UdpPassthrough,
}

// impl JuniperContext for Context {}

impl Context {
    // /// Returns a reference to the context's subsystem instance
    // pub fn subsystem(&self) -> &T {
    //     &self.subsystem
    // }

    /// Returns a reference to the context's UdpPassthrough instance
    pub fn udp(&self) -> &UdpPassthrough {
        &self.udp_pass
    }

    /// Attempts to get a value from the context's storage
    ///
    /// # Arguments
    ///
    /// `name` - Key to search for in storage
    pub fn get(&self, name: &str) -> String {
        let stor = self.storage.read().unwrap();
        match stor.get(&name.to_string()) {
            Some(s) => s.clone(),
            None => "".to_string(),
        }
    }

    /// Sets a value in the context's storage
    ///
    /// # Arguments
    ///
    /// `key` - Key to store value under
    /// `value` - Value to store
    pub fn set(&self, key: &str, value: &str) {
        let mut stor = self.storage.write().unwrap();
        stor.insert(key.to_string(), value.to_string());
    }

    /// Clears a single key/value from storage
    ///
    /// # Arguments
    ///
    /// `key` - Key to clear (along with corresponding value)
    pub fn clear(&self, name: &str) {
        let mut storage = self.storage.write().unwrap();
        storage.remove(name);
    }

    /// Clears all key/value pairs from storage
    pub fn clear_all(&self) {
        self.storage.write().unwrap().clear();
    }
}

/// This structure represents a hardware service.
///
/// Specifically the functionality provided by this struct
/// exists to provide a GraphQL interface over UDP, a means
/// of translate GraphQL commands to UDP commands and vice versa,
/// for debugging software with GraphQL running on the connected
/// computer rather than the satellite to improve performance.
///
/// ### Examples
///
/// # Creating and starting a service.
/// ```rust,ignore
/// use cubeos_service::Service;
///
/// Service::new(
///     "example-service",
///     schema::QueryRoot,
///     schema::MutationRoot,
///     socket,
///     target,
/// ).start();
/// ```
pub struct Service {
    config: Config,
    pub context: Context,
    json_handler: Option<Arc<OutputFn>>, 
}

impl Service {
    /// Creates a new service instance
    ///
    /// # Arguments
    ///
    /// `name` - The name of the service. This is used to find the appropriate config information
    /// `query` - The root query struct holding all other GraphQL queries.
    /// `mutation` - The root mutation struct holding all other GraphQL mutations.
    /// `socket` - UDP Socket to bind on the ground computer to enable UDP msgs
    /// `target` - Address of service running on the satellite
    pub fn new(
        config: Config,
        socket: String,
        target: String,
        json_handler: Option<Arc<OutputFn>>,
    ) -> Self
    {
            let context = Context {
            storage: Arc::new(RwLock::new(HashMap::new())),
            udp_pass: UdpPassthrough::new(socket,target),
        };

        Service { config, context, json_handler }
    }

    /// Starts the service's GraphQL/UDP server. This function runs
    /// without return.
    ///
    /// # Panics
    ///
    /// The UDP interface will panic if the ip address and port provided
    /// cannot be bound (like if they are already in use), or if for some reason the socket fails
    /// to receive a message.
    pub fn start(self) {
        let socket = UdpSocket::bind(self.context.udp_pass.socket).expect("couldn't bind to address");
        let target = self.context.udp_pass.to;
        // loop for JSON handling
        // listens for JSON messages on socket
        // uses json_handler function supplied by service to handle the cmd
        // returns answer to sender
        // #[cfg(feature = "debug")]
        println!("Start listener on: {:?}", socket);
        let mut buf = [0; 1024];
        loop{
            match socket.recv_from(&mut buf) {                
                Ok((b,a)) => {   
                    // #[cfg(feature = "debug")]
                    let msg = String::from_utf8(buf[..b].to_vec()).unwrap();
                    println!("Received message: {:?} from {:?}", msg, a);  
                    let sock = UdpSocket::bind("0.0.0.0:0").expect("couldn't bind to address");                
                    // let msg = String::from_utf8(msg).unwrap();                   
                    let handler = self.json_handler.as_ref().unwrap().clone();
                    let target = target.clone();
                    thread::spawn(move || {
                        let reply = handler(msg,target);
                        sock.send_to(reply.as_bytes(), a).unwrap();  
                    });
                    continue;
                }
                Err(_) => continue,
            };
        }       
    }
}