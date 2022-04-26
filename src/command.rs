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

use cubeos_error::{Error, Result};
use serde::{Serialize,Deserialize};
use std::mem::size_of;

// Struct that enables deserializing of incoming Vec<u8> msgs
// into data structures specified in the API or Service
#[derive(Serialize,Deserialize)]
pub struct Command<T> {
    // SpacePacket Command-ID retained for future use
    _id: u64,
    // Data from Vec<u8>,
    data: T,
}
impl<'a,T: Deserialize<'a>> Command<T>
    {
    pub fn new(&self, _id: u64, msg: &'a Vec<u8>) -> Self {
        Self {
            _id,
            data: Command::<T>::parse(&msg).unwrap(),
        }
    }
    // parser function, that deserializes msg if len equals
    // size of the data type T
    pub fn parse(msg: &'a Vec<u8>) -> Result<T> {       
        // if msg.len() != size_of::<T>() {
        //     #[cfg(feature = "debug")]
        //     println!("WrongNoArgs");
        //     Err(Error::WrongNoArgs)
        // } else {            
            Ok(bincode::deserialize(&msg).unwrap())
        // }
    }
}

// Empty data type used for functions (queries) that don't 
// require input data, e.g. ping()
#[derive(Serialize,Deserialize)]
pub struct Generic {
    pub gen: (),
}
