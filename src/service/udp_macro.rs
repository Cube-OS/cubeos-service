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

        command_id!{
            LastCmd,
            LastErr,
            $($type,)*
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
                println!("{:?}",err);
                if let Ok(mut last_err) = self.last_err.write() {
                    *last_err = err;
                }
            }
            fn get_last_err(&self) -> CubeOSResult<CubeOSError> {
                Ok(self.last_err.read().unwrap().clone())
            }
        }

        // UDP handler function running on the service
        // takes incoming msg and parses it into CommandID and Command for msg handling
        pub fn udp_handler(sub: &Box<Subsystem>, msg: &mut Vec<u8>) -> CubeOSResult<Vec<u8>> {
            // Verify CommandID            
            match CommandID::try_from(u16::from_be_bytes([msg[0],msg[1]]))? {
                CommandID::LastCmd => {
                    Command::<CommandID,Vec<u8>>::serialize(CommandID::LastCmd,sub.get_last_cmd()?)
                },
                CommandID::LastErr => {
                    Command::<CommandID,CubeOSError>::serialize(CommandID::LastErr,sub.get_last_err()?)
                }             
                $(CommandID::$type => {
                    sub.set_last_cmd(msg.to_vec());
                    // Parse Command
                    let command = Command::<CommandID,($($cmd),*)>::parse(msg)?;                    
                    // Serialize 
                    let data = command.data;
                    match run!(Subsystem::$func; sub, data $(,$cmd)*) {
                        Ok(x) => Ok(Command::<CommandID,$rep>::serialize(command.id,x)?),
                        Err(e) => {
                            sub.set_last_err(CubeOSError::from(e.clone()));
                            Err(CubeOSError::from(e))
                        }
                    }
                    // match Command::<CommandID,$rep>::serialize(command.id,(run!(Subsystem::$func; sub, data $(,$cmd)*))?) {
                    //     Ok(x) => Ok(x),
                    //     Err(e) => {
                    //         sub.set_last_err(CubeOSError::from(e.clone()));
                    //         Err(CubeOSError::from(e))
                    //     }
                    // }
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