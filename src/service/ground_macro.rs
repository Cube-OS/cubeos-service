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
        // $(
        //     // generic: $type_e: ident => {$func_e: tt, $cmd_e: tt, $conv_e: tt, $rep_e: tt},
        //     generic: $type_e: ident => fn $func_e: tt (&self, $ign0_e: tt:Generic) -> $ign1_e: tt<$rep_e: tt>; ($conv_e: tt, $gql_e: tt),
        // )*
        // query: $type_q: ident => {$func_q: tt, $conv_q: tt, $cmd_q: tt, $rep_q: tt},
        // mutation: $type_m: ident => {$func_m: tt, $conv_m: tt, $cmd_m: tt, $rep_m: tt},
        $(            
            query: $type_q: ident => fn $func_q: tt (&self $(, $msg_q: tt:$cmd_q: ty)*) -> $ign1_q: tt<$rep_q: ty> $(; in:)? $($conv_q: ty),* $(; out: $gql_q: ty)?;
        )*
        $(
            mutation: $type_m: ident => fn $func_m: tt (&self $(, $msg_m: tt:$cmd_m: ty)*) -> $ign1_m: tt<$rep_m: ty> $(; in:)? $($conv_m: ty),* $(; out: $gql_m: ty)?;
        )*
    ) => {    
        // use std::convert::{TryFrom,TryInto};
        // use variant_count::VariantCount;
        // use cubeos_error::{Error as CubeOSError, Result as CubeOSResult};
        use juniper::{FieldResult,graphql_object};
        use std::net::UdpSocket;
        use serde_json::to_string;

        use command_id::*;

        command_id!{$($type_q,)*$($type_m,)*}
        
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

        // GraphQL interface run on the debug computer
        // Translates GraphQL Inputs into UDP msg and sends these to the satellite
        // Translates Serialized responses from the satellite into GraphQL
        pub type Context = cubeos_service::Context;
        pub struct QueryRoot;    
        graphql_object!(QueryRoot: Context as "Query" |&self| {            
            $(                 
                field $func_q(&executor $(, $msg_q: $conv_q)*) -> FieldResult<String> {
                    let mut cmd = Command::<CommandID,($($cmd_q),*)>::serialize(CommandID::$type_q,($(<$cmd_q>::try_from($msg_q)?),*)).unwrap();
                    match udp_passthrough(cmd,executor.context().udp()) {
                        Ok(buf) => {
                            match Command::<CommandID,$rep_q>::parse(&buf) {
                                Ok(c) => Ok(serde_json::to_string(&<($($gql_q)*)>::from(c.data)).unwrap()),
                                Err(CubeOSError::NoCmd) => Ok(serde_json::to_string(&CubeOSError::from(bincode::deserialize::<CubeOSError>(&buf[2..].to_vec())?)).unwrap()),
                                Err(err) => Ok(serde_json::to_string(&CubeOSError::from(err)).unwrap()),
                            }
                        }
                        Err(err) => Ok(serde_json::to_string(&CubeOSError::from(err)).unwrap()),
                    }
                }            
            )*
        });
        pub struct MutationRoot;  
        graphql_object!(MutationRoot: Context as "Mutation" |&self| {            
            $(                 
                field $func_m(&executor $(, $msg_m: $conv_m)*) -> FieldResult<String> {
                    let mut cmd = Command::<CommandID,($($cmd_m),*)>::serialize(CommandID::$type_m,($(<$cmd_m>::try_from($msg_m)?),*)).unwrap();
                    match udp_passthrough(cmd,executor.context().udp()) {
                        Ok(buf) => {
                            match Command::<CommandID,$rep_m>::parse(&buf) {
                                Ok(c) => Ok(serde_json::to_string(&<($($gql_m)*)>::from(c.data)).unwrap()),
                                Err(CubeOSError::NoCmd) => Ok(serde_json::to_string(&CubeOSError::from(bincode::deserialize::<CubeOSError>(&buf[2..].to_vec())?)).unwrap()),
                                Err(err) => Ok(serde_json::to_string(&CubeOSError::from(err)).unwrap()),
                            }
                        }
                        Err(err) => Ok(serde_json::to_string(&CubeOSError::from(err)).unwrap()),
                    }
                },
            )*
        });
    }
}