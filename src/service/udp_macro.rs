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
        use crate::$krate::$strukt as Subsystem;

        command_id!{
            // Ping,
            // LastCmd,
            // LastErr,
            $($type,)*
        }

        //impl Last for Subsystem {
        //    fn set_last_cmd(&self, input: Vec<u8>) {
        //        if let Ok(mut last_cmd) = self.last_cmd.write() {
        //            *last_cmd = input;
        //        }
        //    }
        //    fn get_last_cmd(&self) -> CubeOSResult<Vec<u8>> {
        //        Ok(self.last_cmd.read().unwrap().to_vec()) 
        //    }
        //   fn set_last_err(&self, err: CubeOSError) {
        //        println!("{:?}",err);
        //        if let Ok(mut last_err) = self.last_err.write() {
        //            *last_err = err;
        //        }
        //    }
        //    fn get_last_err(&self) -> CubeOSResult<CubeOSError> {
        //        Ok(self.last_err.read().unwrap().clone())
        //    }
        //}

        //impl Ping for Subsystem {
        //    fn ping(&self) -> CubeOSResult<()> {
        //       Ok(())
        //    }
        //}

        // UDP handler function running on the service
        // takes incoming msg and parses it into CommandID and Command for msg handling
        pub fn udp_handler(sub: &mut Box<Subsystem>, msg: &mut Vec<u8>) -> CubeOSResult<Vec<u8>> {
            #[cfg(feature = "debug")]
            println!("Message: {:?}",msg);

            // Verify CommandID            
            match CommandID::try_from(u16::from_be_bytes([msg[0],msg[1]]))? {
                // CommandID::Ping => {
                //     Command::<CommandID,()>::serialize(CommandID::Ping,sub.ping()?)
                // },
                // CommandID::LastCmd => {
                //     Command::<CommandID,Vec<u8>>::serialize(CommandID::LastCmd,sub.get_last_cmd()?)
                // },
                // CommandID::LastErr => {
                //     Command::<CommandID,CubeOSError>::serialize(CommandID::LastErr,sub.get_last_err()?)
                // }             
                $(CommandID::$type => {
                    // sub.set_last_cmd(msg.to_vec());
                    // Parse Command
                    let command = Command::<CommandID,($($cmd),*)>::parse(msg)?;                    
                    // Serialize 
                    let data = command.data;
                    match run!(Subsystem::$func; sub, data $(,$cmd)*) {
                        Ok(x) => {
                            let r = Command::serialize(command.id,x)?;
                            #[cfg(feature = "debug")]
                            println!("Reply: {:?}",r);
                            // Ok(Command::<CommandID,$rep>::serialize(command.id,x)?),
                            Ok(r)
                        }
                        Err(e) => {
                            // sub.set_last_err(CubeOSError::from(e.clone()));
                            Err(CubeOSError::from(e))
                        }
                    }
                },)* 
            }
        }

        #[cfg(feature = "debug")]
        pub fn debug() {
            println!("{:?}", CommandID::VARIANT_COUNT);
            let mut cmd: usize = 0;
            while cmd <= CommandID::VARIANT_COUNT {
                println!("{:?}: {:?}", cmd, CommandID::try_from(cmd as u16));
                cmd = cmd + 1;
            }
        }            
    };
}

#[macro_export]
macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}

// #[macro_export]
// macro_rules! run {
//     // Base case: 0 or more input parameters
//     ($f:expr; $sub:expr, $in:expr, $($params:expr),*) => {
//         $f($sub, $in, $($params),*);
//     };
//     // Recursive case: one or more arguments left
//     ($f:expr; $sub:expr, $in:expr, $next_arg:expr, $($rest:tt)*) => {
//         run!($f, $sub, $in, $next_arg, $($rest)*);
//     };
// }

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
