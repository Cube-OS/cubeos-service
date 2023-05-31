#[macro_export]
macro_rules! terminal_macro{
    (
        $service: tt: $struct: tt {
            $(            
                $(query)?$(mutation)?: $type_a: ident => fn $func: tt (&$(mut )?self $(,$msg: tt: $cmd: ty)*) -> $ign1: tt<$rep: ty> $(; out: $gql_q: ty)?;
            )*
        }
    ) => {    
        command_id!{
            $($type_a,)*
        }

        lazy_static! {
            static ref HOST_URL: String = {
                Config::new(stringify!($service))
                    .unwrap()
                    .get("ground_url")
                    .unwrap()     
                    .to_string()               
            };
        }

        #[derive(Default,Debug,Clone,Copy)]
        pub struct $struct {}
        impl $struct {
            pub fn terminal(&self) -> String {
                println!("{}",stringify!($service));
                loop {
                    println!("");
                    match MultiSelect::new()
                        $(.item(stringify!($type_a)))*
                        .interact_opt() 
                    {
                        Ok(Some(selection)) => {
                            for s in selection.iter() {
                                println!("{}",self.handle(*s+1));
                            }
                        },
                        _ => continue,
                    } 
                }
            }

            fn handle(&self, selection: usize) -> String {
                match CommandID::try_from(selection as u16) {
                    $(Ok(CommandID::$type_a) => {                           
                        ground_handle!($($cmd,)* $($msg,)* $type_a);
                        // println!("{}",stringify!($type_a));
                        // let mut input = String::new();
                        // std::io::stdin().read_line(&mut input).unwrap();
                        // let input_msg = serde_json::from_str::<($($cmd),*)>(&input).unwrap();
                        // let cmd = Command::<CommandID,($($cmd),*)>::new(CommandID::$type_a,input_msg);
                        let cmd = bincode::deserialize::<Command<CommandID,($($cmd),*)>>(&cmd).unwrap();
                        let terminal_socket = UdpSocket::bind("0.0.0.0:0").unwrap();
                        #[cfg(feature = "debug")]
                        println!("Sending to: {}",HOST_URL.as_str());
                        terminal_socket.send_to(&bincode::serialize(&serde_json::to_string::<Command<CommandID,($($cmd),*)>>(&cmd).unwrap()).unwrap(),HOST_URL.as_str()).unwrap();
                        let mut buf = [0; 1024];
                        let (amt, _) = terminal_socket.recv_from(&mut buf).unwrap();
                        let reply = bincode::deserialize::<$rep>(&buf[..amt]).unwrap();
                        serde_json::to_string::<$rep>(&reply).unwrap()                        
                    },)*
                    _ => {
                        String::from("Invalid selection")
                    }
                }
            }
        }
    }
}
