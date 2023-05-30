//
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
// Contributed by: Patrick Oppel (patrick.oppel94@gmail.com)
// 

#[macro_export]
macro_rules! service_macro {
    (
        use $error: ty;
        $krate: tt ::$strukt: tt {
            $(            
                query: $type_q: ident => fn $func_q: tt (&$(mut )?self $(, $msg_q: tt:$cmd_q: ty)*) -> $ign1_q: tt<$rep_q: ty> $(; out: $gql_q: ty)?;
            )*
            $(
                mutation: $type_m: ident => fn $func_m: tt (&$(mut )?self $(, $msg_m: tt:$cmd_m: ty)*) -> $ign1_m: tt<$rep_m: ty>;
            )*
        }
    ) => {    
        use std::str::FromStr;
        use failure::Fail;
        use std::net::{UdpSocket,SocketAddr};
        use cubeos_service::serde_json::to_string_pretty;
        use cubeos_service::udp_rs::Message;
        use cubeos_service::bincode;
        use cubeos_service::Result;
        use cubeos_service::dialoguer::{MultiSelect,Select};
        use cubeos_service::command_id;
        // use ground_handle::ground_handle;
        use strum_macros::{EnumIter, Display};
        use strum::IntoEnumIterator;
        use std::convert::{From,Into};
        use std::io::Write;
        use std::fs::Metadata;
        use print_json::print_json;
        use syn::*;
        use proc_macro2::*;
        use quote::*;
        use cargo_metadata::*;

        command_id!{
            // Ping,
            // LastCmd,
            // LastErr,
            $($type_q,)*
            $($type_m,)*
        }

        print_json!(
            $($type_q$(, $msg_q, $cmd_q),*;)*
            $($type_m$(, $msg_m, $cmd_m),*;)*
        );
        // $(print_json!($type_m,$($msg_m,$cmd_m),*);)*

        $(#[derive(Serialize,Deserialize,Debug)]
        pub struct $type_q {
            $(pub $msg_q: $cmd_q,)*
        }
        impl From<Command<CommandID,($($cmd_q),*)>> for $type_q {
            fn from(cmd: Command<CommandID,($($cmd_q),*)>) -> Self {
                let ($($msg_q),*) = cmd.data;
                Self {
                    $($msg_q),*
                }
            }
        }
        impl From<$type_q> for Command<CommandID,($($cmd_q),*)> {
            fn from(t: $type_q) -> Command<CommandID,($($cmd_q),*)> {
                let data = ($(t.$msg_q),*);
                Command {
                    id: CommandID::$type_q,
                    data,
                }
            }
        })*

        $(#[derive(Serialize,Deserialize,Debug)]
        pub struct $type_m {
            $(pub $msg_m: $cmd_m,)*
        }
        impl From<Command<CommandID,($($cmd_m),*)>> for $type_m {
            fn from(cmd: Command<CommandID,($($cmd_m),*)>) -> Self {
                let ($($msg_m),*) = cmd.data;
                Self {
                    $($msg_m),*
                }
            }
        }
        impl From<$type_m> for Command<CommandID,($($cmd_m),*)> {
            fn from(t: $type_m) -> Command<CommandID,($($cmd_m),*)> {
                let data = ($(t.$msg_m),*);
                Command {
                    id: CommandID::$type_m,
                    data,
                }
            }
        })*

        #[derive(Serialize,Deserialize,Debug)]
        enum Commands{
            $($type_q($type_q),)*
            $($type_m($type_m),)*
        }
        impl Commands {
            fn parse<'a>(msg: &'a Vec<u8>) -> Result<Self> {
                match CommandID::try_from(u16::from_be_bytes([msg[0],msg[1]])) {                    
                    $(Ok(CommandID::$type_q) => Ok(Commands::$type_q(bincode::deserialize::<$type_q>(&msg[2..])?)),)*
                    $(Ok(CommandID::$type_m) => Ok(Commands::$type_m(bincode::deserialize::<$type_m>(&msg[2..])?)),)*
                    Err(_) => Err(cubeos_service::Error::from(bincode::deserialize::<cubeos_service::Error>(&msg[2..])?)),
                }
            }

            fn serialize(&self) -> Result<Vec<u8>> {
                match self {
                    $(Commands::$type_q(q) => {
                        let mut buf: Vec<u8> = Vec::new();
                        buf.append(&mut u16::to_be_bytes(CommandID::$type_q as u16).to_vec());
                        buf.append(&mut bincode::serialize(q).unwrap());
                        Ok(buf)
                    },)*
                    $(Commands::$type_m(m) => {
                        let mut buf: Vec<u8> = Vec::new();
                        buf.append(&mut u16::to_be_bytes(CommandID::$type_m as u16).to_vec());
                        buf.append(&mut bincode::serialize(m).unwrap());
                        Ok(buf)
                    },)*
                }
            }            
        }

        // pub fn write_file() {
        //     let mut file = std::fs::OpenOptions::new()
        //         .create(true)
        //         .append(true)
        //         .open("commands.json")
        //         .expect("Failed to open file");

        //     $(writeln!(file,"{}",stringify!($type_q));
        //         $(print_json!($cmd_q);)*)*

        //     $(writeln!(file,"{}",stringify!($type_m));
        //         $(print_json!($cmd_m);)*)* 
        // }

        // function to connect to and send UDP messages to the satellite
        // binds socket and sends to target addresses specified in the config.toml file
        pub fn udp_passthrough(cmd: Vec<u8>, target: SocketAddr) -> CubeOSResult<Vec<u8>> {
            let mut buf = [0u8; 255];
            let socket = UdpSocket::bind("0.0.0.0:0").expect("couldn't bind to address");
            match socket.send_msg(&cmd,&target) {
                Ok(_) => {
                    #[cfg(feature = "debug")]
                    println!("Sending");
                    match socket.recv_msg() {
                        Ok((b,a)) => {
                            #[cfg(feature = "debug")]
                            println!("Received: {:?}", b);
                            Ok(b)
                        },
                        Err(_) => Err(CubeOSError::NoCmd),
                    }
                },
                Err(_) => Err(CubeOSError::NoCmd),
            }
        }

        fn handle_error(e: CubeOSError) -> String {
            match e {                
                CubeOSError::ServiceError(_) => <$error>::from(e).to_string(),
                _ => (&e).to_string(),
            }
        }
        
        pub fn json_handler(json: String, target: SocketAddr) -> String { 
            match serde_json::from_str::<Commands>(&json) {
                $(Ok(Commands::$type_q(q)) => {
                    let comand: Command<CommandID,($($cmd_q),*)>= q.into();
                    println!("{:?}",comand);
                    let cmd = match comand.ser() {
                        Ok(cmd) => cmd,
                        Err(e) => return serde_json::to_string_pretty(&e).unwrap(),
                    };
                    match udp_passthrough(cmd.into(),target) {
                        Ok(buf) => {
                            match Command::<CommandID,($($gql_q)?)>::parse(&buf) {
                                Ok(c) => match serde_json::to_string_pretty(&c.data) {
                                    Ok(s) => s,
                                    Err(e) => e.to_string(),
                                },
                                Err(err) => match serde_json::to_string_pretty(&handle_error(bincode::deserialize::<CubeOSError>(&buf[1..]).unwrap())) {
                                    Ok(s) => s,
                                    Err(e) => e.to_string(),
                                },
                            }
                        },
                        Err(err) => match serde_json::to_string_pretty(&handle_error(CubeOSError::from(err))) {
                            Ok(s) => s,
                            Err(e) => e.to_string(),
                        },
                    }
                }),*
                $(Ok(Commands::$type_m(m)) => {
                    let comand: Command<CommandID,($($cmd_m),*)>= m.into();
                    println!("{:?}",comand);
                    let cmd = match comand.ser() {
                        Ok(cmd) => cmd,
                        Err(e) => return serde_json::to_string_pretty(&e).unwrap(),
                    };
                    match udp_passthrough(cmd,target) {
                        Ok(buf) => {
                            match Command::<CommandID,()>::parse(&buf) {
                                Ok(c) => match serde_json::to_string_pretty(&c.data) {
                                    Ok(s) => s,
                                    Err(e) => e.to_string(),
                                },
                                Err(err) => match serde_json::to_string_pretty(&handle_error(bincode::deserialize::<CubeOSError>(&buf[1..]).unwrap())) {
                                    Ok(s) => s,
                                    Err(e) => e.to_string(),
                                },
                            }
                        },
                        Err(err) => match serde_json::to_string_pretty(&handle_error(CubeOSError::from(err))) {
                            Ok(s) => s,
                            Err(e) => e.to_string(),
                        },
                    }
                }),*
                Err(e) => {
                    println!("{:?}",e);
                    "Command ID not found".to_string()
                }
            }
            // match serde_json::from_str::<Command::<CommandID,serde_json::Value>>(&json) {
            //     Ok(cmd) => match cmd.id {
            //         $(CommandID::$type_q => {
            //             let cmd = match serde_json::from_str::<Command<CommandID,($($cmd_q),*)>>(&json) {
            //                 Ok(cmd) => cmd,
            //                 Err(e) => return "failed to parse command".to_string(),
            //             };
            //             let cmd = match Command::<CommandID, ($($cmd_q),*)>::serialize(cmd.id,cmd.data) {
            //                 Ok(cmd) => cmd,
            //                 Err(e) => return serde_json::to_string_pretty(&e).unwrap(),
            //             };
            //             match udp_passthrough(cmd,target) {
            //                 Ok(buf) => {
            //                     match Command::<CommandID,($($gql_q)?)>::parse(&buf) {
            //                         Ok(c) => match serde_json::to_string_pretty(&c.data) {
            //                             Ok(s) => s,
            //                             Err(e) => e.to_string(),
            //                         },
            //                         Err(err) => match serde_json::to_string_pretty(&handle_error(bincode::deserialize::<CubeOSError>(&buf[1..]).unwrap())) {
            //                             Ok(s) => s,
            //                             Err(e) => e.to_string(),
            //                         },
            //                     }
            //                 },
            //                 Err(err) => match serde_json::to_string_pretty(&handle_error(CubeOSError::from(err))) {
            //                     Ok(s) => s,
            //                     Err(e) => e.to_string(),
            //                 },
            //             }
            //         }),*
            //         $(CommandID::$type_m => {  
            //             let cmd = match serde_json::from_str::<Command<CommandID,($($cmd_m),*)>>(&json) {
            //                 Ok(cmd) => cmd,
            //                 Err(e) => return "failed to parse command".to_string(),
            //             };
            //             let cmd = match Command::<CommandID, ($($cmd_m),*)>::serialize(cmd.id,cmd.data) {
            //                 Ok(cmd) => cmd,
            //                 Err(e) => return serde_json::to_string_pretty(&e).unwrap(),
            //             };                      
            //             match udp_passthrough(cmd,target) {
            //                 Ok(buf) => {
            //                     match Command::<CommandID,()>::parse(&buf) {
            //                         Ok(c) => match serde_json::to_string_pretty(&c.data) {
            //                             Ok(s) => s,
            //                             Err(e) => e.to_string(),
            //                         },
            //                         Err(err) => match serde_json::to_string_pretty(&handle_error(bincode::deserialize::<CubeOSError>(&buf[1..]).unwrap())) {
            //                             Ok(s) => s,
            //                             Err(e) => e.to_string(),
            //                         },
            //                     }
            //                 },
            //                 Err(err) => match serde_json::to_string_pretty(&handle_error(CubeOSError::from(err))) {
            //                     Ok(s) => s,
            //                     Err(e) => e.to_string(),
            //                 },
            //             }
            //         },)*
            //     },
            //     Err(_) => "Command ID not found".to_string(),                
            // }            
        }
            // match CommandID::try_from(selection as u16) {
            //     Ok(id) => match id {
            //         $(CommandID::$type_q => {
            //             println!("{:?}:",stringify!($type_q));
            //             ground_handle!($($cmd_q,)* $($msg_q,)* $type_q );                                   
            //             match udp_passthrough(cmd,udp) {
            //                 Ok(buf) => {
            //                     match Command::<CommandID,($($gql_q)?)>::parse(&buf) {
            //                         Ok(c) => match serde_json::to_string_pretty(&c.data) {
            //                             Ok(s) => s,
            //                             Err(e) => e.to_string(),
            //                         },
            //                         Err(err) => match serde_json::to_string_pretty(&handle_error(bincode::deserialize::<CubeOSError>(&buf[1..]).unwrap())) {
            //                             Ok(s) => s,
            //                             Err(e) => e.to_string(),
            //                         },
            //                     }
            //                 },
            //                 Err(err) => match serde_json::to_string_pretty(&handle_error(CubeOSError::from(err))) {
            //                     Ok(s) => s,
            //                     Err(e) => e.to_string(),
            //                 },
            //             }
            //         },)*
            //         $(CommandID::$type_m => {
            //             println!("{:?}:",stringify!($type_m));
            //             ground_handle!($($cmd_m,)* $($msg_m,)* $type_m );
            //             match udp_passthrough(cmd,udp) {
            //                 Ok(buf) => {
            //                     match Command::<CommandID,()>::parse(&buf) {
            //                         Ok(c) => match serde_json::to_string_pretty(&c.data) {
            //                             Ok(s) => s,
            //                             Err(e) => e.to_string(),
            //                         },
            //                         Err(err) => match serde_json::to_string_pretty(&handle_error(bincode::deserialize::<CubeOSError>(&buf[1..]).unwrap())) {
            //                             Ok(s) => s,
            //                             Err(e) => e.to_string(),
            //                         },
            //                     }
            //                 },
            //                 Err(err) => match serde_json::to_string_pretty(&handle_error(CubeOSError::from(err))) {
            //                     Ok(s) => s,
            //                     Err(e) => e.to_string(),
            //                 },
            //             }
            //         },)*
            //     },
            //     Err(_) => "Command ID not found".to_string(),
            // }
        // }

        // pub fn terminal(udp: UdpPassthrough) {
        //     loop {
        //         println!("");
        //         match MultiSelect::new()
        //             $(.item(stringify!($type_q)))*
        //             $(.item(stringify!($type_m)))*
        //             .interact_opt() 
        //         {
        //             Ok(Some(selection)) => {
        //                 for s in selection.iter() {
        //                     println!("{}",handle(*s+1, &udp));
        //                 }
        //             },
        //             _ => continue,
        //         } 
        //     }         
        // }

        // pub fn file(udp: UdpPassthrough) {
        //     // Get the current process's executable path
        //     let exe_path = std::env::current_exe().expect("Failed to get current executable path");

        //     let file_path = std::path::Path::new(&format!("{}.json",exe_path.to_str().unwrap().to_owned()));
        //     // Return the file name component of the executable path as a &str
        //     let name = exe_path.file_name().unwrap().to_str().unwrap().to_owned() + ".json";

        //     let mut file = if !file_path.exists() {
        //         std::fs::File::create(&name).expect("Couldn't create file")
        //     } else {
        //         std::fs::File::open(file_path).expect("Couldn't open file")
        //     };

        //     loop {
        //         println!("");
        //         match MultiSelect::new()
        //             $(.item(stringify!($type_q)))*
        //             $(.item(stringify!($type_m)))*
        //             .interact_opt() 
        //         {
        //             Ok(Some(selection)) => {
        //                 for s in selection.iter() {
        //                     file.write_all(&handle(*s+1, &udp).as_bytes()).expect("Failed to write to file");
        //                 }
        //             },
        //             _ => continue,
        //         } 
        //     }         
        // }

        #[cfg(feature = "debug")]
        pub fn debug() {
            println!("{:?}", CommandID::VARIANT_COUNT);
            let mut cmd: usize = 0;
            while cmd <= CommandID::VARIANT_COUNT {
                println!("{:?}: {:?}", cmd, CommandID::try_from(cmd as u16));
                cmd = cmd + 1;
            }
        }  
    }
}
