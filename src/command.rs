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

use cubeos_error::*;
use serde::{Serialize,Deserialize};
use std::convert::TryFrom;

// Struct that enables deserializing of incoming Vec<u8> msgs
// into data structures specified in the API or Service
#[derive(Serialize,Deserialize, Debug)]
pub struct Command<C,T> {
    // SpacePacket Command-ID retained for future use
    pub id: C,
    // Data from Vec<u8>,
    pub data: T,
}
impl<'a,C: TryFrom<u16>, T: Serialize + Deserialize<'a>> Command<C,T>
    where 
        u16: TryFrom<C>,
        cubeos_error::Error: From<<C as TryFrom<u16>>::Error>,
        cubeos_error::Error: From<<u16 as TryFrom<C>>::Error>,
    {
    pub fn new(&self, id: C, msg: &'a Vec<u8>) -> Self {
        Self {
            id,
            data: bincode::deserialize(&msg).unwrap(),
        }
    }

    // parser function
    pub fn parse(msg: &'a Vec<u8>) -> Result<Self> {       
        
        match u16::from_be_bytes([msg[0],msg[1]]) {
            0 => Err(Error::from(bincode::deserialize::<Error>(&msg[2..])?)),
            id => Ok(Command{id: C::try_from(id)?,data: bincode::deserialize(&msg[2..])?}),
            65535 => Err(Error::from(bincode::deserialize::<Error>(&msg[2..])?)),
        }
    }

    // serializer function
    pub fn serialize(id: C, msg: T) -> Result<Vec<u8>> {
        let mut buf: Vec<u8> = Vec::new();

        buf.append(&mut u16::try_from(id)?.to_be_bytes().to_vec());
        buf.append(&mut bincode::serialize(&msg)?);
        Ok(buf)
    }
}