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

use juniper::{GraphQLObject,GraphQLEnum};
use serde::{Serialize,Deserialize};
use serde_repr::*;

// Return for commands without output
// Defined as 0xFF to avoid ambiguity with cubeos-error::Error return values
#[derive(GraphQLEnum,Serialize_repr,Deserialize_repr,Debug)]
#[repr(u8)]
pub enum Resp {
    Success = 0xFF,
}

// Return for commands without output
#[derive(GraphQLObject,Serialize,Deserialize,Debug)]
pub struct GenericResponse
{
    pub success: Resp,
}
impl GenericResponse {
    pub fn new() -> Self {
        Self {
            success: Resp::Success,
        }
    }
}
// Alternative implementation
// Can be used to return Result<GenericResponse>:
// Ok(().into())
impl From<()> for GenericResponse {
    fn from(_r: ()) -> GenericResponse {
        GenericResponse {
            success: Resp::Success,
        }
    }
}