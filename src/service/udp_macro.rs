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
            $ign3: tt: $type: ident => fn $func: tt (&self $(,$ign0: tt: $cmd: tt)*) -> $ign1: tt<$rep: ty>; ($ign2: ty, $ign4: ty),
                // ign0: tt:$cmd: tt) -> $ign1: tt<$rep: tt>; ($ign2: tt, $ign4: tt),
        )+
        // $(
        //     $ign3n: tt: $typen: ident => fn $funcn: tt (&self, $($ign0n: tt: $cmdn: tt),+) -> $ign1n: tt<$repn: tt>; ($ign2n: tt, $ign4n: tt),
        // )*
        // $(
        //     $ign3: tt: $type3: ident => fn $func: tt (&self, $($ign0: tt: $cmd: tt),+) -> $ign1: tt<$rep: tt>; ($ign2: tt, $ign4: tt),
        // )*
        // $(
        //     $ign3: tt: $type4: ident => fn $func: tt (&self, $($ign0: tt: $cmd: tt),+) -> $ign1: tt<$rep: tt>; ($ign2: tt, $ign4: tt),
        // )*
    ) => {
        use command_id::*;
        use std::env::Args;
        use crate::subsystem::*;

        command_id!{$($type,)*}    
        // command!{$(id: $type,$(cmd: $cmd),*)+}
        
        // pub trait Service {
        //     $(fn $func(&self, $($ign0:$cmd),+) -> CubeOSResult<$rep>;)+            
        // }
        // pub trait UdpHandler {
        //     fn udp_handler(&self, msg: &mut Vec<u8>) -> CubeOSResult<Vec<u8>>;
        // }
        // impl UdpHandler for Service {
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
                    // let f: dyn Fn($($cmd),*) -> CubeOSResult<$rep> = sub.$func;
                    // let count = count!($($cmd)*);
                    // let strings: [String; 3] = init_array![String::from("hi!"); 3];
                    Command::<CommandID,$rep>::serialize(command.id,(run!(Subsystem::$func; sub, data $(,$cmd)*))?)                                        
                },)* 
                // $(CommandID::$type2 => {
                //     // Parse Command
                //     let command = Command::<CommandID,($($cmd),+)>::parse(msg)?;                    
                //     // Serialize Result
                //     Command::<CommandID,$rep>::serialize(command.id,sub.$func(command.data.0,command.data.1,command.data.2)?)                                        
                // },)* 
                // $(CommandID::$type2 => {
                //     // Parse Command
                //     let command = Command::<CommandID,($($cmd),+)>::parse(msg)?;                    
                //     // Serialize Result
                //     Command::<CommandID,$rep>::serialize(command.id,sub.$func(command.data.0,command.data.1,command.data.2,command.data.3)?)                                        
                // },)* 
            }
        }
        // }
    };
}

#[macro_export]
macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}

// #[macro_export]
// macro_rules! init_array {
//     (@accum (0, $_e:expr) -> ($($body:tt)*))
//         => {init_array!(@as_expr [$($body)*])};
//     (@accum (1, $e:expr) -> ($($body:tt)*))
//         => {init_array!(@accum (0, $e) -> ($($body)* $e,))};
//     (@accum (2, $e:expr) -> ($($body:tt)*))
//         => {init_array!(@accum (1, $e) -> ($($body)* $e,))};
//     (@accum (3, $e:expr) -> ($($body:tt)*))
//         => {init_array!(@accum (2, $e) -> ($($body)* $e,))};
//     (@as_expr $e:expr) => {$e};
//     [$e:expr; $n:tt] => {
//         {
//             let e = $e;
//             init_array!(@accum ($n, e.clone()) -> ())
//         }
//     };
// }

#[macro_export]
macro_rules! run {    
    ($f: expr; $sub: tt, $in: tt)=> {$f($sub)};
    ($f: expr; $sub: tt, $in: tt, $_cmd: tt)=> {$f($sub,$in)};
    ($f: expr; $sub: tt, $in: tt, $_cmd: tt, $_cmd2: tt) => {$f($sub,$in.0,$in.1)};
    // ($f: expr; $in: tt, $_cmd: tt, $_cmd2: tt) => {$f($in.0, $in.1)};
    // ($f: expr; $in: tt, $_cmd: tt) => {$f($in)};
}