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
        use $error: ty;
        $krate: tt ::$strukt: tt {
            $(
                $(query)?$(mutation)?: $type: ident => fn $func: tt (&$(mut )?self $(,$ign0: tt: $cmd: ty)*) -> $ign1: tt<$rep: ty> $(; out: $gql_q: ty)?;
            )*
        }
    ) => {
        use cubeos_service::command_id;
        use std::env::Args;
        use std::str::FromStr;
        use log::debug;
        use crate::$krate::$strukt as Subsystem;

        command_id!{
            // Ping,
            // LastCmd,
            // LastErr,
            $($type,)*
        }

        // UDP handler function running on the service
        // takes incoming msg and parses it into CommandID and Command for msg handling
        pub fn udp_handler(sub: &mut Box<Subsystem>, msg: &mut Vec<u8>) -> CubeOSResult<Vec<u8>> {
            debug!("Message: {:?}",msg);

            // Verify CommandID            
            match CommandID::try_from(u16::from_be_bytes([msg[0],msg[1]]))? {          
                $(CommandID::$type => {
                    let data = bincode::deserialize::<($($cmd),*)>(&msg[2..])?;
                    match run!(Subsystem::$func; sub, data $(,$cmd)*) {
                        Ok(x) => {                            
                            let mut r = <u16>::try_from(CommandID::$type)?.to_be_bytes().to_vec();
                            r.append(&mut bincode::serialize(&x)?);                            
                            debug!("Reply: {:?}",r);
                            Ok(r)
                        }
                        Err(e) => {
                            Err(CubeOSError::from(e))
                        }
                    }
                },)* 
            }
        }

        // #[cfg(feature = "debug")]
        // pub fn debug() {
        //     println!("{:?}", CommandID::VARIANT_COUNT);
        //     let mut cmd: usize = 0;
        //     while cmd <= CommandID::VARIANT_COUNT {
        //         println!("{:?}: {:?}", cmd, CommandID::try_from(cmd as u16));
        //         cmd = cmd + 1;
        //     }
        // }            
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
    // 6 input parameter
    ($f: expr; $sub: tt, $in: tt, $_cmd: tt, $_cmd2: tt, $_cmd3: tt, $_cmd4: tt, $_cmd5: tt, $_cmd6: tt) 
        => {$f($sub,$in.0,$in.1,$in.2,$in.3,$in.4,$in.5)};
    // 7 input parameter
    ($f: expr; $sub: tt, $in: tt, $_cmd: tt, $_cmd2: tt, $_cmd3: tt, $_cmd4: tt, $_cmd5: tt, $_cmd6: tt, $_cmd7: tt) 
        => {$f($sub,$in.0,$in.1,$in.2,$in.3,$in.4,$in.5,$in.6)};
    // 8 input parameter
    ($f: expr; $sub: tt, $in: tt, $_cmd: tt, $_cmd2: tt, $_cmd3: tt, $_cmd4: tt, $_cmd5: tt, $_cmd6: tt, $_cmd7: tt, $_cmd8: tt) 
        => {$f($sub,$in.0,$in.1,$in.2,$in.3,$in.4,$in.5,$in.6,$in.7)};
    // 9 input parameter
    ($f: expr; $sub: tt, $in: tt, $_cmd: tt, $_cmd2: tt, $_cmd3: tt, $_cmd4: tt, $_cmd5: tt, $_cmd6: tt, $_cmd7: tt, $_cmd8: tt, $_cmd9: tt) 
        => {$f($sub,$in.0,$in.1,$in.2,$in.3,$in.4,$in.5,$in.6,$in.7,$in.8)};
    // 10 input parameter
    ($f: expr; $sub: tt, $in: tt, $_cmd: tt, $_cmd2: tt, $_cmd3: tt, $_cmd4: tt, $_cmd5: tt, $_cmd6: tt, $_cmd7: tt, $_cmd8: tt, $_cmd9: tt, $_cmd10: tt) 
        => {$f($sub,$in.0,$in.1,$in.2,$in.3,$in.4,$in.5,$in.6,$in.7,$in.8,$in.9)};
}
