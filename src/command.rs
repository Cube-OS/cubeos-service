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

use crate::error::*;
use serde::{Serialize,Deserialize};
use std::convert::TryFrom;

#[macro_export]
macro_rules! command_id{
    (
        $($type: ident,)+
    ) => {
        use std::convert::{TryFrom,TryInto};
        use cubeos_service::variant_count::VariantCount;
        use cubeos_service::{Error as CubeOSError, Result as CubeOSResult};
        use std::ops::AddAssign;
        use std::fmt;
        use serde::{Serialize,Deserialize};

        // helper functions to implement the TryFrom<u16> for udp and ground macros
        // increments a usize and outputs the value
        // needed to increment a counter inside the macro expression $()+ 
        pub fn increment(i: &mut usize) -> usize {
            i.add_assign(1);
            *i-1
        }

        // Construct CommandID Enum
        #[derive(Clone,Copy,Debug,PartialEq,VariantCount,Serialize,Deserialize)]
        pub enum CommandID {
            $(
                $type,
            )+
        }

        impl FromStr for CommandID {
            type Err = CubeOSError;
            fn from_str(s: &str) -> CubeOSResult<Self> {
                match s {
                    $(
                        stringify!($type) => Ok(CommandID::$type),
                    )+
                    _ => Err(CubeOSError::NoCmd)
                }
            }
        }
        // implementation of conversion of u16 to CommandID
        impl TryFrom<u16> for CommandID {
            type Error = CubeOSError;

            fn try_from(cmd: u16) -> CubeOSResult<Self> {
                let mut i: usize = 0;
                let h_field: Vec<u16> = (1..1+CommandID::VARIANT_COUNT as u16).collect();
                match cmd {
                    $(x if x == h_field[increment(&mut i)] => Ok(CommandID::$type),)+
                    _ => Err(CubeOSError::NoCmd),
                }
            }
        }  
        
        // implement conversion of CommandID to u16
        impl TryFrom<CommandID> for u16 {
            type Error = CubeOSError;

            fn try_from(c: CommandID) -> CubeOSResult<u16> {
                let mut i: usize = 0;
                let h_field: Vec<u16> = (1..1+CommandID::VARIANT_COUNT as u16).collect();
                match c {
                    $(CommandID::$type => Ok(h_field[CommandID::$type as usize]),)*
                    _ => Err(CubeOSError::NoCmd),
                }
            }
        }

        #[cfg(feature = "terminal")]
        #[derive(Debug,Clone,Serialize,Deserialize)]
        pub enum Command{
            $(
                $type($type),
            )+
        }
    }
}