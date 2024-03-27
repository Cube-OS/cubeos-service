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
        use log::{debug,info,error};
        use std::net::UdpSocket;
        use cubeos_service::serde_json::to_string_pretty;
        use cubeos_service::udp_rs::Message;
        use cubeos_service::bincode;
        use cubeos_service::command_id;
        use cubeos_service::dialoguer::{MultiSelect,Select};
        use terminal_macro::terminal_macro;
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

        terminal_macro!(
            $($type_q$(, $msg_q, $cmd_q),*;)*
            $($type_m$(, $msg_m, $cmd_m),*;)*
        );
        
        // function to connect to and send UDP messages to the satellite
        // binds socket and sends to target addresses specified in the config.toml file
        pub fn udp_passthrough(cmd: Vec<u8>, udp: &UdpPassthrough) -> CubeOSResult<Vec<u8>> {
            // let mut buf = [0u8; 255];
            let socket = match UdpSocket::bind(udp.socket) {
                Ok(s) => s,
                Err(_) => return Err(CubeOSError::NoCmd),
            };
            // let socket = s.unwrap();
            debug!("{:?}",udp.to);
            debug!("Cmd: {:?}", cmd);
            match socket.send_msg(&cmd,&udp.to) {
                Ok(_) => {
                    debug!("Sending");
                    match socket.recv_msg() {
                        Ok((b,a)) => {                            
                            debug!("Received: {:?}", b);
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

        fn handle_id(mut cmd: Vec<u8>) -> Vec<u8> {
            let mut first = cmd.remove(0);

            let first_u16 = u16::from(first) + 1;
            let mut buf = Vec::new();
            buf.append(&mut first_u16.to_be_bytes().to_vec());
            buf.append(&mut cmd);
            buf
        }
        
        pub fn output(mut command: String, udp: UdpPassthrough) -> String {
            let cmd_enum = serde_json::from_str::<Command>(&command).unwrap();

            let cmd_ser = match bincode::serialize(&cmd_enum) {
                Ok(c) => c,
                Err(e) => return handle_error(CubeOSError::from(e)),
            };
            let cmd_fin = handle_id(cmd_ser);
            match udp_passthrough(cmd_fin,&udp) {
                Ok(buf) => {
                    match u16::from_be_bytes([buf[0],buf[1]]) {
                        0 => return handle_error(bincode::deserialize::<CubeOSError>(&buf[2..]).unwrap()).to_string(),
                        65535 => return handle_error(bincode::deserialize::<CubeOSError>(&buf[2..]).unwrap()).to_string(),
                        _ => {
                            match cmd_enum {
                                $(Command::$type_q(_) => {
                                    match bincode::deserialize::<$rep_q>(&buf[2..]) {
                                        Ok(c) => match serde_json::to_string_pretty(&<$($gql_q)?>::from(c)) {
                                            Ok(s) => s,
                                            Err(e) => e.to_string(),
                                        },
                                        Err(e) => e.to_string(),
                                    }                                    
                                },)*
                                $(Command::$type_m(_) => "Success".to_string(),)*
                                _ => format!("Invalid command: {}", command),                            
                            }
                        }
                    }
                },
                Err(err) => match serde_json::to_string_pretty(&handle_error(CubeOSError::from(err))) {
                    Ok(s) => s,
                    Err(e) => e.to_string(),
                },
            }
        }

        fn handle_input(selection: usize) -> Result<String> {
            match CommandID::try_from(selection as u16) {
                Ok(id) => match id {
                    $(CommandID::$type_q => {
                        println!("{}",stringify!($type_q));
                        let input = get_input::<$type_q>();
                        let cmd = Command::$type_q(input);
                        Ok(serde_json::to_string_pretty(&cmd).unwrap())
                    },)*
                    $(CommandID::$type_m => {
                        println!("{}",stringify!($type_m));
                        let input = get_input::<$type_m>();
                        let cmd = Command::$type_m(input);
                        Ok(serde_json::to_string_pretty(&cmd).unwrap())
                    },)*
                },
                Err(e) => Err(e),
            }
        }

        pub fn input() -> Result<String> {
            println!("");
            match Select::new()
                $(.item(stringify!($type_q)))*
                $(.item(stringify!($type_m)))*
                .interact() 
            {
                Ok(selection) => {
                    Ok(handle_input(selection+1)?)
                },
                Err(e) => Err(e.into()),
            }         
        }

        // #[cfg(feature = "debug")]
        // pub fn debug() {
        //     println!("{:?}", CommandID::VARIANT_COUNT);
        //     let mut cmd: usize = 0;
        //     while cmd <= CommandID::VARIANT_COUNT {
        //         println!("{:?}: {:?}", cmd, CommandID::try_from(cmd as u16));
        //         cmd = cmd + 1;
        //     }
        // }  
    }
}