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
        $(
            // generic: $type_e: ident => {$func_e: tt, $cmd_e: tt, $conv_e: tt, $rep_e: tt},
            generic: $type_e: ident => fn $func_e: tt (&self, $ign0_e: tt:Generic) -> $ign1_e: tt<$rep_e: tt>; ($conv_e: tt, $gql_e: tt),
        )*
        $(
            // query: $type_q: ident => {$func_q: tt, $conv_q: tt, $cmd_q: tt, $rep_q: tt},
            query: $type_q: ident => fn $func_q: tt (&self, $ign0_q: tt:$cmd_q: tt) -> $ign1_q: tt<$rep_q: tt>; ($conv_q: tt, $gql_q: tt),
        )*
        $(
            // mutation: $type_m: ident => {$func_m: tt, $conv_m: tt, $cmd_m: tt, $rep_m: tt},
            mutation: $type_m: ident => fn $func_m: tt (&self, $ign0_m: tt:$cmd_m: tt) -> $ign1_m: tt<$rep_m: tt>; ($conv_m: tt, $gql_m: tt),
        )*
    ) => {    
        use std::convert::{TryFrom,TryInto};
        use variant_count::VariantCount;
        use cubeos_error::{Error as CubeOSError, Result as CubeOSResult};
        use juniper::{FieldResult,graphql_object};
        use std::net::UdpSocket;
        use serde_json::to_string;

        // construct CommandID Enum
        #[derive(Clone,Copy,Debug,PartialEq,VariantCount)]
        pub enum CommandID {
            $(
                $type_e,
            )*
            $(
                $type_q,
            )*
            $(
                $type_m,
            )*
        }
        // implement conversion of u16 to CommandID
        impl TryFrom<u16> for CommandID {
            type Error = CubeOSError;

            fn try_from(cmd: u16) -> Result<Self,Self::Error> {
                let mut i: usize = 0;
                let h_field: Vec<u16> = (0..CommandID::VARIANT_COUNT as u16).collect();
                match cmd {
                    $(x if x == h_field[increment(&mut i)] => Ok(CommandID::$type_e),)*
                    $(x if x == h_field[increment(&mut i)] => Ok(CommandID::$type_q),)*
                    $(x if x == h_field[increment(&mut i)] => Ok(CommandID::$type_m),)*
                    _ => Err(CubeOSError::NoCmd),
                }
            }
        }
        // implement conversion of CommandID to u16
        impl TryFrom<CommandID> for u16 {
            type Error = CubeOSError;

            fn try_from(c: CommandID) -> Result<u16,Self::Error> {
                let mut i: usize = 0;
                let h_field: Vec<u16> = (0..CommandID::VARIANT_COUNT as u16).collect();
                match c {
                    $(CommandID::$type_e => Ok(h_field[CommandID::$type_e as usize]),)*
                    $(CommandID::$type_q => Ok(h_field[CommandID::$type_q as usize]),)*
                    $(CommandID::$type_m => Ok(h_field[CommandID::$type_m as usize]),)*
                    _ => Err(CubeOSError::NoCmd),
                }
            }
        }        

        // helper function to convert CommandID to u16
        pub fn convert(c: CommandID) -> CubeOSResult<u16> {
            println!("{:?}",c);
            CommandID::try_into(c)
        }
        
        // function to connect to and send UDP messages to the satellite
        // binds socket and sends to target addresses specified in the config.toml file
        pub fn udp_passthrough(cmd: Vec<u8>, udp: &UdpPassthrough) -> CubeOSResult<Vec<u8>> {
            let mut buf = [0u8; 255];
            let s = match UdpSocket::bind(udp.socket) {
                Ok(s) => Ok(s),
                Err(_) => Err(CubeOSError::NoCmd),
            };
            let socket = s.unwrap();
            let to = udp.to;
            #[cfg(feature = "debug")]
            println!("{:?}",to);
            #[cfg(feature = "debug")]
            println!("Cmd: {:?}", cmd);
            socket.connect(to).expect("Could not connect to satellite");
            match socket.send(&cmd) {
                Ok(_) => {
                    #[cfg(feature = "debug")]
                    println!("Sending");
                    match socket.recv(&mut buf) {
                        Ok(x) => {
                            #[cfg(feature = "debug")]
                            println!("Received: {:?}", buf[..x].to_vec());
                            Ok(buf[..x].to_vec())
                        },
                        Err(_) => Err(CubeOSError::NoCmd),
                    }
                },
                Err(_) => Err(CubeOSError::NoCmd),
            }
        }

        // fn handle_reply(buf: &mut Vec<u8>, cmd: CommandID, des: T) -> serde_json::Result<String> {
        //     // Check bytes 0+1 for return type
        //     // [0,0] or [255,255] => Error
        //     // [CommandID as u16] => No Error
        //     match CommandID::try_from(((msg[0] as u16) << 8) | msg[1] as u16) {
        //         Ok(cmd) => deserialize_reply(&buf[2..].to_vec(), des)
        //         _ => deserialize_reply(&buf[2..].to_vec(), CubeOSError)
        //     }
        // }

        // fn deserialize_reply(buf: &Vec<u8>, des: T) -> serde_json::Result<String> {
        //     match bincode::deserialize::<des>(&buf[2..].to_vec()) {
        //         Ok(v) => Ok(serde_json::to_string(&$gql_q::from(v)).unwrap()),
        //         Err(e) => Ok(
        //             serde_json::to_string(&CubeOSError::from(e))
        //             .unwrap()
        //         ),
        //     }
        // }

        // GraphQL interface run on the debug computer
        // Translates GraphQL Inputs into UDP msg and sends these to the satellite
        // Translates Serialized responses from the satellite into GraphQL
        pub type Context = cubeos_service::Context;
        pub struct QueryRoot;    
        graphql_object!(QueryRoot: Context as "Query" |&self| {            
            $(                 
                field $func_q(&executor, msg: $conv_q) -> FieldResult<String> {      
                    match <$cmd_q>::try_from(msg) {
                        Ok(s) => {
                            match convert(CommandID::$type_q) {
                                Ok(b) => {
                                    let mut cmd: Vec<u8> = b.to_be_bytes().to_vec();
                                    println!("{:?}",cmd);
                                    cmd.append(&mut bincode::serialize(&s).unwrap());
                                    let test: u8 = 0;   
                                    match udp_passthrough(cmd,executor.context().udp()) {
                                        Ok(buf) => {
                                            #[cfg(feature = "debug")]
                                            println!("Handle reply");
                                            match CommandID::try_from(((buf[0] as u16) << 8) | buf[1] as u16) {
                                                Ok(CommandID::$type_q) => {
                                                    match bincode::deserialize::<$rep_q>(&buf[2..].to_vec()) {
                                                        Ok(v) => Ok(serde_json::to_string(&$gql_q::from(v)).unwrap()),
                                                        Err(e) => Ok(serde_json::to_string(&CubeOSError::from(e)).unwrap()),
                                                    }                                                    
                                                }
                                                _ => Ok(serde_json::to_string(
                                                        &bincode::deserialize::<CubeOSError>(&buf[2..].to_vec()).unwrap()
                                                    ).unwrap() 
                                                ),
                                            }                                            
                                        }
                                        Err(err) => Ok(
                                            serde_json::to_string(&CubeOSError::from(err))
                                            .unwrap()
                                        ),
                                    }
                                    // match bincode::deserialize::<$rep_q>(&udp_passthrough(cmd,executor.context().udp()).unwrap()) {
                                    //     Ok(v) => Ok(serde_json::to_string(&$gql_q::from(v)).unwrap()),
                                    //     Err(e) => Ok(
                                    //         serde_json::to_string(&CubeOSError::from(e))
                                    //         .unwrap()
                                    //     ),
                                    // }                                                                    
                                },
                                Err(e) => Ok(
                                    serde_json::to_string(&CubeOSError::from(e))
                                    .unwrap()
                                ),
                            }
                        },
                        Err(e) => Ok(
                            serde_json::to_string(&CubeOSError::from(e))
                            .unwrap()
                        ),
                    }
                }            
            )*
            $(
                field $func_e(&executor) -> FieldResult<String> {
                    match convert(CommandID::$type_e) {
                        Ok(b) => {
                            let mut cmd: Vec<u8> = b.to_be_bytes().to_vec();
                            println!("{:?}",cmd);
                            match udp_passthrough(cmd,executor.context().udp()) {
                                Ok(buf) => {
                                    match CommandID::try_from(((buf[0] as u16) << 8) | buf[1] as u16) {
                                        Ok(CommandID::$type_e) => {
                                            match bincode::deserialize::<$rep_e>(&buf[2..].to_vec()) {
                                                Ok(v) => Ok(serde_json::to_string(&$gql_e::from(v)).unwrap()),
                                                Err(e) => Ok(serde_json::to_string(&CubeOSError::from(e)).unwrap()),
                                            }                                                    
                                        }
                                        _ => Ok(serde_json::to_string(
                                            &bincode::deserialize::<CubeOSError>(&buf[2..].to_vec()).unwrap()
                                            ).unwrap()
                                        ),
                                    }                                            
                                }
                                Err(err) => Ok(
                                    serde_json::to_string(&CubeOSError::from(err))
                                    .unwrap()
                                ),
                            }
                            // match bincode::deserialize::<$rep_e>(&udp_passthrough(cmd,executor.context().udp()).unwrap()) {
                            //     Ok(v) => Ok(serde_json::to_string(&$gql_e::from(v)).unwrap()),
                            //     Err(e) => Ok(
                            //         serde_json::to_string(&CubeOSError::from(e))
                            //         .unwrap()
                            //     ),
                            // }
                        },
                        Err(e) => Ok(
                            serde_json::to_string(&CubeOSError::from(e))
                            .unwrap()
                        ),
                    }
                }
            )*
        });
        pub struct MutationRoot;  
        graphql_object!(MutationRoot: Context as "Mutation" |&self| {            
            $(                 
                field $func_m(&executor, msg: $conv_m) -> FieldResult<String> {
                    match <$cmd_m>::try_from(msg) {
                        Ok(s) => {
                            match convert(CommandID::$type_m) {
                                Ok(b) => {
                                    let mut cmd: Vec<u8> = b.to_be_bytes().to_vec();
                                    println!("{:?}",cmd);
                                    cmd.append(&mut bincode::serialize(&s).unwrap());
                                    match udp_passthrough(cmd,executor.context().udp()) {
                                        Ok(buf) => {
                                            match CommandID::try_from(((buf[0] as u16) << 8) | buf[1] as u16) {
                                                Ok(CommandID::$type_m) => {
                                                    match bincode::deserialize::<$rep_m>(&buf[2..].to_vec()) {
                                                        Ok(v) => Ok(serde_json::to_string(&$gql_m::from(v)).unwrap()),
                                                        Err(e) => Ok(serde_json::to_string(&CubeOSError::from(e)).unwrap()),
                                                    }                                                    
                                                }
                                                _ => Ok(serde_json::to_string(
                                                        &bincode::deserialize::<CubeOSError>(&buf[2..].to_vec()).unwrap()
                                                    ).unwrap()
                                                ),
                                            }                                            
                                        }
                                        Err(err) => Ok(
                                            serde_json::to_string(&CubeOSError::from(err))
                                            .unwrap()
                                        ),
                                    }
                                    // match bincode::deserialize::<$rep_m>(&udp_passthrough(cmd,executor.context().udp()).unwrap()) {
                                    //     Ok(v) => Ok(serde_json::to_string(&$gql_m::from(v)).unwrap()),
                                    //     Err(e) => Ok(
                                    //         serde_json::to_string(&CubeOSError::from(e))
                                    //         .unwrap()
                                    //     ),
                                    // }                                
                                },
                                Err(e) => Ok(
                                    serde_json::to_string(&CubeOSError::from(e))
                                    .unwrap()
                                ),
                            }
                        },
                        Err(e) => Ok(
                            serde_json::to_string(&CubeOSError::from(e))
                            .unwrap()
                        ),
                    }             
                }            
            )*
        });
    }
}