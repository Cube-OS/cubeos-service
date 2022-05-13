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
            query: $type_q: ident => fn $func_q: tt (&self $(, $msg_q: tt:$cmd_q: ty)*) -> $ign1_q: tt<$rep_q: ty> $(; in:)? $($conv_q: ty),* $(; out: $gql_q: ty)?;
        )*
        $(
            mutation: $type_m: ident => fn $func_m: tt (&self $(, $msg_m: tt:$cmd_m: ty)*) -> $ign1_m: tt<$rep_m: ty> $(; in:)? $($conv_m: ty),* $(; out: $gql_m: ty)?;
        )*
    ) => {   
        // use std::convert::{TryInto,Into};
        use juniper::{FieldResult,graphql_object};
        use serde_json::*;
        use crate::Subsystem;
        use command_id::*;
        
        command_id!{
            LastCmd,
            LastErr,
            $($type_q,)*
            $($type_m,)*
        }

        impl Last for Subsystem {
            fn set_last_cmd(&self, input: Vec<u8>) {
                if let Ok(mut last_cmd) = self.last_cmd.write() {
                    *last_cmd = input;
                }
            }
            fn get_last_cmd(&self) -> CubeOSResult<Vec<u8>> {
                Ok(self.last_cmd.read().unwrap().to_vec()) 
            }
            fn set_last_err(&self, err: CubeOSError) {
                if let Ok(mut last_err) = self.last_err.write() {
                    *last_err = err;
                }
            }
            fn get_last_err(&self) -> CubeOSResult<CubeOSError> {
                Ok(self.last_err.read().unwrap().clone())
            }
        }

        // GraphQl Query Implementation
        // (previously found in schema.rs)
        pub type Context = cubeos_service::Context<Box<Subsystem>>;
        pub struct QueryRoot;    
        graphql_object!(QueryRoot: Context as "Query" |&self| { 
            field get_last_cmd(&executor) -> FieldResult<String> {
                Ok(serde_json::to_string(&executor.context().subsystem().get_last_cmd().unwrap()).unwrap())
            }
            field get_last_err(&executor) -> FieldResult<String> {
                Ok(serde_json::to_string(&executor.context().subsystem().get_last_err().unwrap()).unwrap())
            }            
            $(                                 
                field $func_q(&executor $(, $msg_q: $conv_q)*) -> FieldResult<String> {
                    executor.context().subsystem().set_last_cmd(Command::<CommandID,($($cmd_q),*)>::serialize(CommandID::$type_q,($(<$cmd_q>::try_from($msg_q.clone())?),*)).unwrap());
                    match executor.context().subsystem().$func_q($($msg_q.try_into().unwrap()),*) {
                        Ok(x) => Ok(serde_json::to_string(&<($($gql_q)*)>::from(x)).unwrap()),
                        Err(e) => {
                            executor.context().subsystem().set_last_err(e.clone());
                            Ok(serde_json::to_string(&CubeOSError::from(e)).unwrap())
                        },
                    }
                    // Ok(serde_json::to_string(
                    //     &<($($gql_q)*)>::from(
                    //         executor
                    //             .context()
                    //             .subsystem()
                    //             .$func_q($($msg_q.try_into().unwrap()),*)
                    //             .unwrap()
                    //         )
                    //     )
                    //     .unwrap()                      
                    // )
                }            
            )*
        });
        
        // GraphQL Mutation implementation
        // (previously found in schema.rs)
        pub struct MutationRoot;  
        graphql_object!(MutationRoot: Context as "Mutation" |&self| {            
            $(                 
                // field $func_m(&executor, msg: $cmd_m) -> FieldResult<String> {
                field $func_m(&executor $(, $msg_m: $conv_m)*) -> FieldResult<String> {
                    executor.context().subsystem().set_last_cmd(Command::<CommandID,($($cmd_m),*)>::serialize(CommandID::$type_m,($(<$cmd_m>::try_from($msg_m.clone())?),*)).unwrap());
                    match executor.context().subsystem().$func_m($($msg_m.try_into().unwrap()),*) {
                        Ok(x) => Ok(serde_json::to_string(&<($($gql_m)*)>::from(x)).unwrap()),
                        Err(e) => {
                            executor.context().subsystem().set_last_err(e.clone());
                            Ok(serde_json::to_string(&CubeOSError::from(e)).unwrap())
                        },
                    }
                    // Ok(serde_json::to_string(
                    //     &<($($gql_m)*)>::from(
                    //         executor
                    //             .context()
                    //             .subsystem()
                    //             .$func_m($($msg_m.try_into().unwrap()),*)
                    //             .unwrap()
                    //         )
                    //     )
                    //     .unwrap()
                    // )
                }            
            )*
        });
    }
}