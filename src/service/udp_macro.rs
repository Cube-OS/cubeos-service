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
            // $ignore: ident : $type: ident => {$func: tt, $ign: tt, $cmd: tt},
            $ign3: tt: $type: ident => fn $func: tt (&self, $ign0: tt:$cmd: tt) -> $ign1: tt<$rep: tt>; ($ign2: tt, $ign4: tt),
        )+
    ) => {
        use command_id::*;

        command_id!{$($type,)*}    
        
        // UDP handler function running on the service
        // takes incoming msg and parses it into CommandID and Command for msg handling
        pub fn udp_handler(sub: &Box<Subsystem>, msg: &mut Vec<u8>) -> CubeOSResult<Vec<u8>> {
            // Verify CommandID
            match CommandID::try_from(u16::from_be_bytes([msg[0],msg[1]]))? {
                $(CommandID::$type => {
                    // Parse Command
                    let cmd = Command::<CommandID,$cmd>::parse(msg)?;
                    // Serialize Result
                    Command::<CommandID,$rep>::serialize(cmd.id,sub.$func(cmd.data)?)
                },)+ 
            }
        }
    };
}