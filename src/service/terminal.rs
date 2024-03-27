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
use log::{debug,info};
use std::collections::HashMap;
use std::net::{SocketAddr};
use std::sync::{Arc, RwLock};
use std::str::FromStr;
use std::io::Write;
use crate::error::*;

/// Type definition for a CLI tool
pub type InputFn = dyn Fn() -> Result<String> + std::marker::Send + std::marker::Sync + 'static;
pub type OutputFn = dyn Fn(String,UdpPassthrough) -> String + std::marker::Send + std::marker::Sync + 'static;

pub struct Functions {
    pub input: Arc<InputFn>,
    pub output: Arc<OutputFn>,
}

pub enum Mode {
    Input,
    Run,
    Terminal,    
}

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
    command: Option<String>,
    functions: Functions,
    mode: Mode,
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
        command: Vec<String>,
        input: Arc<InputFn>,
        output: Arc<OutputFn>,        
    ) -> Self
    {
        let context = Context {
            storage: Arc::new(RwLock::new(HashMap::new())),
            udp_pass: UdpPassthrough::new(socket,target),
        };
        let functions = Functions {
            output,
            input,
        };
        let (mode, command) = if command.len() < 4 {
            (Mode::Terminal, None)
        } else if command.contains(&"-s".to_string()) {
            (Mode::Terminal, Some(command[3..].to_vec().concat()))
        } else if command.contains(&"-p".to_string()) {
            (Mode::Input, None)
        } else {
            (Mode::Run, Some(command[3..].to_vec().concat()))
        };

        Service { config, context, command, functions, mode }
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
        let app_name = std::env::current_exe()
            .unwrap()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();        
        
        match self.mode {
            Mode::Input => {
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("command.txt")
                    .unwrap();
                let input = self.functions.input.clone();
                loop {
                    match input() {
                        Ok(result) => {
                            let data = format!("{}: {}",app_name, result);
                            writeln!(file, "{}", data).unwrap();
                            println!("Write to file: {}",data);
                        },
                        Err(e) => {
                            println!("{}", e);
                        },
                    }
                }
            },
            Mode::Run => {
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("output.txt")
                    .unwrap();
                let output = self.functions.output.clone();
                let result = output(self.command.unwrap(), self.context.udp_pass.clone());
                    
                let data = format!("{}: {:?}", app_name, result);
                writeln!(file, "{}", data).unwrap();
                println!("{}",data);            
            }
            Mode::Terminal => {
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("output.txt")
                    .unwrap();
                let input = self.functions.input.clone();
                let output = self.functions.output.clone();                
                loop {
                    match input() {
                        Ok(parts) => {                           
                            let result = output(parts, self.context.udp_pass.clone());
                            println!("{}",result);
                            if self.command.is_some() {                                        
                                let data = format!("{}: {:?}", app_name, result);
                                writeln!(file, "{}", data).unwrap();
                            }
                        },
                        Err(e) => {
                            println!("{}", e);
                        },
                    }
                }                            
            },
        }
    }
}