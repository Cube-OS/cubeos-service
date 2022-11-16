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
        $krate: tt ::$strukt: tt {
            $(            
                query: $type_q: ident => fn $func_q: tt (&$(mut )?self $(, $msg_q: tt:$cmd_q: ty)*) -> $ign1_q: tt<$rep_q: ty> $(; in:)? $($conv_q: ty),* $(; out: $gql_q: ty)?;
            )*
            $(
                mutation: $type_m: ident => fn $func_m: tt (&$(mut )?self $(, $msg_m: tt:$cmd_m: ty)*) -> $ign1_m: tt<$rep_m: ty> $(; in:)? $($conv_m: ty),*;
            )*
        }
    ) => {    
        // use std::convert::{TryFrom,TryInto};
        // use cubeos_error::{Error as CubeOSError, Result as CubeOSResult};
        use cubeos_service::juniper::{FieldResult,graphql_object, GraphQLObject, GraphQLType, GraphQLEnum};
        use std::net::UdpSocket;
        use cubeos_service::serde_json::to_string;
        use cubeos_service::rust_udp::Message;
        use cubeos_service::bincode;
        use std::convert::Into;
        use cubeos_service::command_id::*;

        command_id!{
            Ping,
            LastCmd,
            LastErr,
            $($type_q,)*
            $($type_m,)*
        }

        $(#[derive(GraphQLObject)]
        pub struct $type_q {
            response: Option<$($gql_q)?>,
            err: String,
        })*

        #[derive(GraphQLObject)]
        pub struct MutResponse {
            success: bool,
            err: String,
        }
        // $(
        //     m_struct!($type_m, $($gql_m)?);
        // )*
        
        // function to connect to and send UDP messages to the satellite
        // binds socket and sends to target addresses specified in the config.toml file
        pub fn udp_passthrough(cmd: Vec<u8>, udp: &UdpPassthrough) -> CubeOSResult<Vec<u8>> {
            let mut buf = [0u8; 255];
            let s = match UdpSocket::bind(udp.socket) {
                Ok(s) => Ok(s),
                Err(_) => Err(CubeOSError::NoCmd),
            };
            let socket = s.unwrap();
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

        fn parse_last_command(msg: &mut Vec<u8>) -> String {
            match CommandID::try_from(u16::from_be_bytes([msg[0],msg[1]])) {
                $(Ok(CommandID::$type_q) => {
                    match Command::<CommandID,($($cmd_q),*)>::parse(msg) {
                        Ok(x) => serde_json::to_string(&x).unwrap(),
                        Err(e) => {
                            let mut s: String = serde_json::to_string(&CommandID::$type_q).unwrap();
                            msg.remove(0);
                            msg.remove(0);
                            $(
                                s.push_str("\n");
                                s.push_str(std::any::type_name::<$cmd_q>());
                                s.push_str(":");
                                let l: Vec<u8> = msg.drain(..std::mem::size_of::<$cmd_q>()).collect();
                                match bincode::deserialize::<$cmd_q>(&l) {
                                    Ok(st) => s.push_str(&serde_json::to_string(&st).unwrap()),
                                    Err(_) => s.push_str(&serde_json::to_string(&l).unwrap()),
                                }
                            )*
                            s
                        }
                    }
                },)*
                $(Ok(CommandID::$type_m) => {
                    match Command::<CommandID,($($cmd_m),*)>::parse(msg) {
                        Ok(x) => serde_json::to_string(&x).unwrap(),
                        Err(e) => {
                            let mut s: String = serde_json::to_string(&CommandID::$type_m).unwrap();
                            msg.remove(0);
                            msg.remove(0);
                            $(
                                s.push_str("\n");
                                s.push_str(std::any::type_name::<$cmd_m>());
                                s.push_str(":");
                                let l: Vec<u8> = msg.drain(..std::mem::size_of::<$cmd_m>()).collect();                                
                                match bincode::deserialize::<$cmd_m>(&l) {
                                    Ok(st) => s.push_str(&serde_json::to_string(&st).unwrap()),
                                    Err(_) => s.push_str(&serde_json::to_string(&l).unwrap()),
                                }
                            )*
                            s
                        }
                    }
                },)*
                _ => {
                    let mut s = serde_json::to_string(&CubeOSError::NoCmd).unwrap();
                    s.push_str(&serde_json::to_string(&msg).unwrap());
                    s
                }
            }
        }

        // GraphQL interface run on the debug computer
        // Translates GraphQL Inputs into UDP msg and sends these to the satellite
        // Translates Serialized responses from the satellite into GraphQL
        pub type Context = cubeos_service::Context;
        pub struct QueryRoot;    
        graphql_object!(QueryRoot: Context as "Query" |&self| {  
            field ping(&executor) -> FieldResult<String> {
                let mut cmd = Command::<CommandID,()>::serialize(CommandID::Ping,()).unwrap();
                match udp_passthrough(cmd,executor.context().udp()) {
                    Ok(buf) => {
                        match Command::<CommandID,()>::parse(&buf) {
                            Ok(c) => Ok(serde_json::to_string(&<()>::from(c.data)).unwrap()),
                            Err(err) => Ok(serde_json::to_string(&CubeOSError::from(err)).unwrap()), 
                        }
                    },
                    Err(err) => Ok(serde_json::to_string(&CubeOSError::from(err)).unwrap()),
                }
            }
            field get_last_cmd(&executor) -> FieldResult<String> {
                let mut cmd = Command::<CommandID,()>::serialize(CommandID::LastCmd,()).unwrap();
                match udp_passthrough(cmd,executor.context().udp()) {
                    Ok(buf) => {
                        match Command::<CommandID,Vec<u8>>::parse(&buf) {
                            // Ok(c) => Ok(serde_json::to_string(&c.data).unwrap()),
                            Ok(mut c) => Ok(parse_last_command(&mut c.data)),
                            Err(e) => Ok(serde_json::to_string(&CubeOSError::from(e)).unwrap()),
                        }
                    }
                    Err(err) => Ok(serde_json::to_string(&CubeOSError::from(err)).unwrap()),
                }
            }
            field get_last_err(&executor) -> FieldResult<String> {
                let mut cmd = Command::<CommandID,()>::serialize(CommandID::LastErr,()).unwrap();
                match udp_passthrough(cmd,executor.context().udp()) {
                    Ok(buf) => {
                        Ok(serde_json::to_string(&CubeOSError::from(bincode::deserialize::<CubeOSError>(&buf[2..]).unwrap())).unwrap())
                        // match Command::<CommandID,CubeOSError>::parse(&buf) {
                        //     Ok(c) => Ok(serde_json::to_string(&c.data).unwrap()),
                        //     Err(e) => Ok(serde_json::to_string(&CubeOSError::from(e)).unwrap()),
                        // }
                    }
                    Err(err) => Ok(serde_json::to_string(&CubeOSError::from(err)).unwrap()),
                }
            }
            $(                 
                field $func_q(&executor $(, $msg_q: $conv_q)*) -> FieldResult<$type_q> {
                    let mut cmd = Command::<CommandID,($($cmd_q),*)>::serialize(CommandID::$type_q,($(<$cmd_q>::try_from($msg_q)?),*)).unwrap();
                    match udp_passthrough(cmd,executor.context().udp()) {
                        Ok(buf) => {
                            match Command::<CommandID,$rep_q>::parse(&buf) {
                                Ok(c) => Ok($type_q{response: Some(serde_json::from_str::<($($gql_q)?)>(&serde_json::to_string(&c.data).unwrap()).unwrap()),err: "".to_string()}),
                                Err(CubeOSError::NoCmd) => Ok($type_q{response: None, err: serde_json::to_string(&CubeOSError::from(bincode::deserialize::<CubeOSError>(&buf[2..].to_vec())?)).unwrap()}),
                                Err(err) => Ok($type_q{response: None, err: serde_json::to_string(&CubeOSError::from(err)).unwrap()}),
                            }
                        }
                        Err(err) => Ok($type_q{response: None, err: serde_json::to_string(&CubeOSError::from(err)).unwrap()}),
                    }
                }            
            )*
        });
        pub struct MutationRoot;  
        graphql_object!(MutationRoot: Context as "Mutation" |&self| {            
            $(     
                field $func_m(&executor $(, $msg_m: $conv_m)*) -> FieldResult<MutResponse> {
                    let mut cmd = Command::<CommandID,($($cmd_m),*)>::serialize(CommandID::$type_m,($(<$cmd_m>::try_from($msg_m)?),*)).unwrap();
                    match udp_passthrough(cmd,executor.context().udp()) {
                        Ok(buf) => {
                            match Command::<CommandID,$rep_m>::parse(&buf) {
                                Ok(c) => Ok(MutResponse{success: true,err: "".to_string()}),
                                Err(CubeOSError::NoCmd) => Ok(MutResponse{success: false, err: serde_json::to_string(&CubeOSError::from(bincode::deserialize::<CubeOSError>(&buf[2..].to_vec())?)).unwrap()}),
                                Err(err) => Ok(MutResponse{success: false, err: serde_json::to_string(&CubeOSError::from(err)).unwrap()}),
                            }
                        }
                        Err(err) => Ok(MutResponse{success: false, err: serde_json::to_string(&CubeOSError::from(err)).unwrap()}),                  
                    }
                },
            )*
        });
    }
}

// #[macro_export]
// macro_rules! m_struct {
//     (
//         $type_m: ident, $gql_m: ty  
//     ) => {
//         #[derive(GraphQLObject)]
//         pub struct $type_m {
//             response: Option<$gql_m>,
//             err: String,
//         }
//     };
//     (
//         $type_m: ident,
//     ) => {
//         #[derive(GraphQLObject)]
//         pub struct $type_m {
//             response: Option<bool>,
//             err: String,
//         }
//     };
// }

// #[macro_export]
// macro_rules! m_func {
//     (
//         $type_m: ident, $func_m: tt, $($msg_m: tt:$cmd_m: ty,)* $rep_m: ty, $($conv_m: ty,)* $gql_m: ty
//     ) => {
//         field $func_m(&executor $(, $msg_m: $conv_m)*) -> FieldResult<$type_m> {
//             let mut cmd = Command::<CommandID,($($cmd_m),*)>::serialize(CommandID::$type_m,($(<$cmd_m>::try_from($msg_m)?),*)).unwrap();
//             match udp_passthrough(cmd,executor.context().udp()) {
//                 Ok(buf) => {
//                     match Command::<CommandID,$rep_m>::parse(&buf) {
//                         Ok(c) => Ok($type_m{response: Some(serde_json::from_str::<($($gql_m)?)>(&serde_json::to_string(&c.data).unwrap()).unwrap()),err: "".to_string()}),
//                         Err(CubeOSError::NoCmd) => Ok($type_m{response: None, err: serde_json::to_string(&CubeOSError::from(bincode::deserialize::<CubeOSError>(&buf[2..].to_vec())?)).unwrap()}),
//                         Err(err) => Ok($type_m{response: None, err: serde_json::to_string(&CubeOSError::from(err)).unwrap()}),
//                     }
//                 }
//                 Err(err) => Ok($type_m{response: None, err: serde_json::to_string(&CubeOSError::from(err)).unwrap()}),                   
//             }
//         }
//     };
//     (
//         $type_m: ident, $func_m: tt, $($msg_m: tt:$cmd_m: ty,)* $rep_m: ty, $($conv_m: ty,)*
//     ) => {
//         field $func_m(&executor $(, $msg_m: $conv_m)*) -> FieldResult<$type_m> {
//             let mut cmd = Command::<CommandID,($($cmd_m),*)>::serialize(CommandID::$type_m,($(<$cmd_m>::try_from($msg_m)?),*)).unwrap();
//             match udp_passthrough(cmd,executor.context().udp()) {
//                 Ok(buf) => {
//                     match Command::<CommandID,$rep_m>::parse(&buf) {
//                         Ok(c) => Ok($type_m{success: true,err: "".to_string()}),
//                         Err(CubeOSError::NoCmd) => Ok($type_m{success: false, err: serde_json::to_string(&CubeOSError::from(bincode::deserialize::<CubeOSError>(&buf[2..].to_vec())?)).unwrap()}),
//                         Err(err) => Ok($type_m{success: false, err: serde_json::to_string(&CubeOSError::from(err)).unwrap()}),
//                     }
//                 }
//                 Err(err) => Ok($type_m{success: false, err: serde_json::to_string(&CubeOSError::from(err)).unwrap()}),                    
//             }
//         }
//     };
// }