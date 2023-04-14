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
use crate::error::*;
use udp_rs::Message;
use std::thread;
use std::sync::Mutex;

/// Type definition for a "UDP" server pointer
pub type UdpFn<T, Vec> = dyn Fn(&mut T, &mut Vec) -> Result<Vec<>> + std::marker::Send + std::marker::Sync + 'static;

/// Context struct used by a service to provide,
/// subsystem access and persistent storage.
#[derive(Clone)]
pub struct Context<T: Clone + std::marker::Send> {
    ///
    pub subsystem: Arc<RwLock<T>>,
    ///
    pub storage: Arc<RwLock<HashMap<String, String>>>,
}

/// This structure represents a hardware service.
///
/// Specifically the functionality provided by this struct
/// exists to provide a UDP interface.
///
/// ### Examples
///
/// # Creating and starting a service.
/// ```rust,ignore
/// use kubos_service::Service;
///
/// let sub = model::Subsystem::new();
/// Service::new(
///     "example-service",
///     sub,
/// ).start();
/// ```
#[derive(Clone)]
pub struct Service<T:Clone + std::marker::Send + 'static>{
// pub struct Service<T> {
    config: Config,
    context: Context<T>,   
    // control: ServiceControlBlock, 
    /// Function pointer to a function that defines how to handle UDP requests
    udp_handler: Option<Arc<UdpFn<T, Vec<u8>>>>,  
}

impl <T: Clone + std::marker::Send + std::marker::Sync + 'static> Service<T> {
    /// Creates a new service instance
    ///
    /// # Arguments
    ///
    /// `name` - The name of the service. This is used to find the appropriate config information
    /// `subsystem` - An instance of the subsystem struct. This one instance will be used by all queries.
    pub fn new(   
        config: Config,   
        subsystem: T,
        udp_handler: Option<Arc<UdpFn<T, Vec<u8>>>>,
    ) -> Self
    // where
    //     T: Send + Sync + Clone + 'static,
    {  
        let context = Context {
            subsystem: Arc::new(RwLock::new(subsystem)),
            storage: Arc::new(RwLock::new(HashMap::new())),
        }; 
        
        Service { config, context, udp_handler }
    }

    /// Starts the service's UDP server. This function runs
    /// without return.
    ///
    /// # Panics
    ///
    /// The UDP interface will panic if the ip address and port provided
    /// cannot be bound (like if they are already in use), or if for some reason the socket fails
    /// to receive a message.
    pub fn start(self) {
        let hosturl = self
            .config
            .hosturl()
            .ok_or_else(|| {
                log::error!("Failed to load service URL");
                "Failed to load service URL"
            })
            .unwrap();
        let addr = hosturl
            .parse::<SocketAddr>()
            .map_err(|err| {
                log::error!("Failed to parse SocketAddr: {:?}", err);
                err
            })
            .unwrap();
        info!("Listening on: {}", addr);

        let udp_handler = self.udp_handler.unwrap();

        let socket = UdpSocket::bind(addr).expect("couldn't bind to address");

        // loop for UDP handling
        // listens for UDP messages on socket
        // uses udp_handler function supplied by service to handle the cmd
        // returns answer to sender
        #[cfg(feature = "debug")]
        println!("Start listener on: {:?}", socket);
        loop{
            match socket.recv_msg() {
                Ok((mut b,a)) => {
                    let sock = UdpSocket::bind("0.0.0.0:0").expect("couldn't bind to address");
                    println!("{:?}", sock);
                    // let handler = udp_handler.lock().unwrap().clone();
                    let handler = udp_handler.clone();
                    let s = self.context.subsystem.clone();
                    thread::spawn(move || {
                        match handler(&mut s.try_write().unwrap(),&mut b) {
                            Ok(x) => {
                                #[cfg(feature = "debug")]
                                println!("Send: {:?} to {:?}",&x,&a);
                                sock.send_msg(&x,&a).expect("Couldn't send")
                            }
                            Err(e) => {
                                sock.send_to(&handle_err(&e),&a).expect("couldn't send");
                            }
                        }
                    });
                    continue;
                }
                Err(_) => continue,
            };
        }
    }
}

// Helper function to handle Errors
// 
// Returns [0,0] instead of CommandID, 
// or [255,255] if another error occured within this function
fn handle_err(err: &Error) -> Vec<u8>{
    #[cfg(feature = "debug")]
    println!("Handle Error");
    let mut buf: Vec<u8> = Vec::new();
    match bincode::serialize(err) {
        Ok(mut k) => {
            buf.append(&mut [0,0].to_vec());
            buf.append(&mut k);
        }
        Err(b) => {
            buf.append(&mut [255,255].to_vec());
            buf.push(from_bincode_error(b));
        }
    }
    buf
}

fn from_bincode_error(b: bincode::Error) -> u8 {
    match *b {
        bincode::ErrorKind::Io(_) => 0,
        bincode::ErrorKind::InvalidUtf8Encoding(_) => 1,
        bincode::ErrorKind::InvalidBoolEncoding(_) => 2,
        bincode::ErrorKind::InvalidCharEncoding => 3,
        bincode::ErrorKind::InvalidTagEncoding(_) => 4,
        bincode::ErrorKind::DeserializeAnyNotSupported => 5,
        bincode::ErrorKind::SizeLimit => 6,
        bincode::ErrorKind::SequenceMustHaveLength => 7,
        bincode::ErrorKind::Custom(_) => 8,            
    }
}
