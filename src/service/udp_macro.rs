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

// Command-ID macro
#[macro_export]
macro_rules! service_macro {
    (
        $(
            $(query)?$(mutation)?: $type: ident => fn $func: tt (&self $(,$ign0: tt: $cmd: ty)*) -> $ign1: tt<$rep: ty> $(; in:)? $($conv_q: ty),* $(; out: $gql_q: ty)?;
        )*
    ) => {
        use command_id::*;
        use std::env::Args;
        use crate::subsystem::*;

        command_id!{$($type,)*}    

        // UDP handler function running on the service
        // takes incoming msg and parses it into CommandID and Command for msg handling
        pub fn udp_handler(sub: &Box<Subsystem>, msg: &mut Vec<u8>) -> CubeOSResult<Vec<u8>> {
            // Verify CommandID            
            match CommandID::try_from(u16::from_be_bytes([msg[0],msg[1]]))? {                
                $(CommandID::$type => {
                    // Parse Command
                    let command = Command::<CommandID,($($cmd),*)>::parse(msg)?;                    
                    // Serialize 
                    let data = command.data;
                    Command::<CommandID,$rep>::serialize(command.id,(run!(Subsystem::$func; sub, data $(,$cmd)*))?)                                        
                },)* 
            }
        }
    };
}

#[macro_export]
macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}

#[macro_export]
macro_rules! run {    
    // 0 input parameter
    ($f: expr; $sub: tt, $in: tt)
        => {$f($sub)};
    // 1 input parameter
    ($f: expr; $sub: tt, $in: tt, $_cmd: tt)
        => {$f($sub,$in)};
    // 2 input parameter        
    ($f: expr; $sub: tt, $in: tt, $_cmd: tt, $_cmd2: tt) 
        => {$f($sub,$in.0,$in.1)};
    // 3 input parameter
    ($f: expr; $sub: tt, $in: tt, $_cmd: tt, $_cmd2: tt, $_cmd3: tt) 
        => {$f($sub,$in.0,$in.1,$in.2)};
    // 4 input parameter
    ($f: expr; $sub: tt, $in: tt, $_cmd: tt, $_cmd2: tt, $_cmd3: tt, $_cmd4: tt) 
        => {$f($sub,$in.0,$in.1,$in.2,$in.3)};
    // 5 input parameter
    ($f: expr; $sub: tt, $in: tt, $_cmd: tt, $_cmd2: tt, $_cmd3: tt, $_cmd4: tt, $_cmd5: tt) 
        => {$f($sub,$in.0,$in.1,$in.2,$in.3,$in.4)};
}