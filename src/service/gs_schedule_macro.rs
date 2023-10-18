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

#[macro_export]
macro_rules! service_macro {
    (
        use $error: ty;
        $krate: tt ::$strukt: tt {
            $(            
                query: $type_q: ident => fn $func_q: tt (&$(mut )?self $(, $msg_q: tt:$cmd_q: ty)*) -> $ign1_q: tt<$rep_q: ty> $(; out: $gql_q: ty)?;
            )*
            $(
                mutation: $type_m: ident => fn $func_m: tt (&$(mut )?self $(, $msg_m: tt:$cmd_m: ty)*) -> $ign1_m: tt<$rep_m: ty>;
            )*
        }
    ) => {    
        use std::str::FromStr;
        use failure::Fail;
        use std::net::UdpSocket;
        use cubeos_service::serde_json::to_string_pretty;
        use cubeos_service::udp_rs::Message;
        use cubeos_service::bincode;
        use cubeos_service::dialoguer::{MultiSelect,Select};
        use cubeos_service::command_id;
        use terminal_macro::terminal_macro;
        use strum::IntoEnumIterator;
        use std::convert::{From,Into};
        use std::io::Write;

        command_id!{
            // Ping,
            // LastCmd,
            // LastErr,
            $($type_q,)*
            $($type_m,)*
        }

        terminal_macro!(
            $($type_q$(, $msg_q, $cmd_q),*;)*
            $($type_m$(, $msg_m, $cmd_m),*;)*
        );
          
        fn handle(selection: usize) -> Result<String> {
            match CommandID::try_from(selection as u16) {
                Ok(id) => match id {
                    $(CommandID::$type_q => {
                        println!("{}",stringify!($type_q));
                        let input = get_input::<$type_q>();
                        let output = format!("{:?}",input);
                        Ok(output)
                    },)*
                    $(CommandID::$type_m => {
                        println!("{}",stringify!($type_m));
                        let input = get_input::<$type_m>();
                        let output = format!("{:?}",input);
                        Ok(output)
                    },)*
                },
                Err(e) => Err(e),
            }
        }

        pub fn terminal() {
            let app_name = std::env::current_exe()
                .unwrap()
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();
            loop {
                println!("");
                match MultiSelect::new()
                    $(.item(stringify!($type_q)))*
                    $(.item(stringify!($type_m)))*
                    .interact_opt() 
                {
                    Ok(Some(selection)) => {
                        let mut output = format!("{}: ",app_name);
                        for s in selection.iter() {
                            match &handle(*s+1) {
                                Ok(s) => output.push_str(s),
                                Err(e) => {
                                    println!("Command ID not found!");
                                    continue;
                                }
                            }
                        }
                        let mut file = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open("command.txt")
                            .unwrap();
                        writeln!(file, "{}", output).unwrap();
                    },
                    _ => continue,
                } 
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
    }
}