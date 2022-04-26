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
        use std::convert::{TryFrom,TryInto};
        use variant_count::VariantCount;
        use cubeos_error::{Error as CubeOSError, Result as CubeOSResult};

        // Construct CommandID Enum
        #[derive(Clone,Copy,Debug,PartialEq,VariantCount)]
        pub enum CommandID {
            $(
                $type,
            )+
        }
        // implementation of conversion of u16 to CommandID
        impl TryFrom<u16> for CommandID {
            type Error = CubeOSError;

            fn try_from(cmd: u16) -> Result<Self,Self::Error> {
                let mut i: usize = 0;
                let h_field: Vec<u16> = (0..CommandID::VARIANT_COUNT).collect();
                match cmd {
                    $(x if x == h_field[increment(&mut i)] => Ok(CommandID::$type),)+
                    _ => Err(CubeOSError::NoCmd),
                }
            }
        }  
        
        // implement conversion of CommandID to u16
        impl TryFrom<CommandID> for u16 {
            type Error = CubeOSError;

            fn try_from(c: CommandID) -> Result<u16,Self::Error> {
                let mut i: usize = 0;
                let h_field: Vec<u16> = (0..CommandID::VARIANT_COUNT).collect();
                match c {
                    $(CommandID::$type => Ok(h_field[CommandID::$type as usize]),)*
                    _ => Err(CubeOSError::NoCmd),
                }
            }
        }

        #[cfg(feature = "debug")]
        pub fn debug() {
            $(
                println!("{:?}: {:?}", CommandID::$type, u16::try_from(CommandID::$type));
            )+
        }      
        
        // fn stream(msg: &mut Vec<u8>) -> CubeOSResult<Vec<u8>> {
        //     bincode::serialize(msg)
        // }

        fn handle_err(buf: &mut Vec<u8>, err: &CubeOSError) {
            #[cfg(feature = "debug")]
            println!("Handle Error");
            buf.clear();            
            match bincode::serialize(err) {
                Ok(mut k) => {
                    buf.append(&mut [0,0].to_vec());
                    buf.append(&mut k);
                }
                Err(b) => {
                    buf.append(&mut [255,255].to_vec());
                    buf.push(from_bincode_error(b));
                }
            }
        }

        fn from_bincode_error(b: bincode::Error) -> u8 {
            match *b {
                bincode::ErrorKind::Io(_) => 0,
                bincode::ErrorKind::InvalidUtf8Encoding(_) => 1,
                bincode::ErrorKind::InvalidBoolEncoding(_) => 2,
                bincode::ErrorKind::InvalidCharEncoding => 3,
                bincode::ErrorKind::InvalidTagEncoding(_) => 4,
                bincode::ErrorKind::DeserializeAnyNotSupported => 5,
                bincode::ErrorKind::SizeLimit => 6,
                bincode::ErrorKind::SequenceMustHaveLength => 7,
                bincode::ErrorKind::Custom(_) => 8,            
            }
        }
        
        // UDP handler function running on the service
        // takes incoming msg and parses it into CommandID and Command for msg handling
        pub fn udp_handler(sub: &Box<Subsystem>, msg: &mut Vec<u8>) -> CubeOSResult<Vec<u8>> {
            // generate CommandID from incoming msg
            #[cfg(feature = "debug")]
            println!("{:?}",msg);
            match CommandID::try_from(((msg[0] as u16) << 8) | msg[1] as u16) {
                Ok(cmdid) => {
                    // save first 2 bytes to send back in reply
                    let mut buf: Vec<u8> = Vec::new();
                    buf.push(msg.remove(0));
                    buf.push(msg.remove(0));
                    // msg handling
                    // Returns serialized result from fn
                    match cmdid {
                        $(CommandID::$type => {
                            match &sub.$func(Command::<$cmd>::parse(&msg.to_vec())?) {
                                Ok(res) => {
                                    #[cfg(feature = "debug")]
                                    println!("Function Result: {:?}", res);
                                    match bincode::serialize(res) {
                                        Ok(mut out) => buf.append(&mut out),
                                        Err(err) => handle_err(&mut buf, &err.into()),
                                    }                                    
                                }
                                Err(err) => handle_err(&mut buf, err),                                                                     
                            }  
                            #[cfg(feature = "debug")]                          
                            println!("Send: {:?}", buf.to_vec());
                            Ok(buf)
                        },)+
                    }
                }
                Err(e) => Err(e),
            }
        }
    };
}