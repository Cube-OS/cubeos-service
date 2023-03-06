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
        use std::net::UdpSocket;
        use cubeos_service::serde_json::to_string_pretty;
        use cubeos_service::rust_udp::Message;
        use cubeos_service::bincode;
        use cubeos_service::dialoguer::{MultiSelect,Select};
        use cubeos_service::command_id::*;
        use ground::ground_handle;
        use strum::IntoEnumIterator;
        use std::convert::{From,Into};
        use std::io::Write;

        command_id!{
            // Ping,
            // LastCmd,
            // LastErr,
            $($type_q,)*
            $($type_m,)*
        }
        
        // function to connect to and send UDP messages to the satellite
        // binds socket and sends to target addresses specified in the config.toml file
        pub fn udp_passthrough(cmd: Vec<u8>, udp: &UdpPassthrough) -> CubeOSResult<Vec<u8>> {
            let mut buf = [0u8; 255];
            let socket = match UdpSocket::bind(udp.socket) {
                Ok(s) => s,
                Err(_) => return Err(CubeOSError::NoCmd),
            };
            // let socket = s.unwrap();
            #[cfg(feature = "debug")]
            println!("{:?}",udp.to);
            #[cfg(feature = "debug")]
            println!("Cmd: {:?}", cmd);
            match socket.send_msg(&cmd,&udp.to) {
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
        
        fn handle(selection: usize, udp: &UdpPassthrough) -> String {
            match CommandID::try_from(selection as u16) {
                Ok(id) => match id {
                    $(CommandID::$type_q => {
                        println!("{:?}:",stringify!($type_q));
                        ground_handle!($($cmd_q,)* $($msg_q,)* $type_q );                                   
                        match udp_passthrough(cmd,udp) {
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
                    },)*
                    $(CommandID::$type_m => {
                        println!("{:?}:",stringify!($type_m));
                        ground_handle!($($cmd_m,)* $($msg_m,)* $type_m );
                        match udp_passthrough(cmd,udp) {
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
                    },)*
                },
                Err(_) => "Command ID not found".to_string(),
            }
        }

        pub fn terminal(udp: UdpPassthrough) {
            loop {
                println!("");
                match MultiSelect::new()
                    $(.item(stringify!($type_q)))*
                    $(.item(stringify!($type_m)))*
                    .interact_opt() 
                {
                    Ok(Some(selection)) => {
                        for s in selection.iter() {
                            println!("{}",handle(*s+1, &udp));
                        }
                    },
                    _ => continue,
                } 
            }         
        }

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