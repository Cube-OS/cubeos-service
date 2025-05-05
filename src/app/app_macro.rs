#[macro_export]
macro_rules! app_macro{
    (
        // $app: tt: $timeout: tt;
        $service: tt: $struct: tt {
            $(            
                $(query)?$(mutation)?: $type: ident => fn $func: tt (&$(mut )?self $(,$msg: tt: $cmd: ty)*) -> $ign1: tt<$rep: ty> $(; out: $gql_q: ty)?;
            )*
        }
    ) => {
        use cubeos_service::udp_rs::Connection;
        use cubeos_service::command_id;
        use std::str::FromStr;
        use log::debug;

        command_id!{
            $($type,)*
        }        

        lazy_static! {
            static ref HOST_URL: String = {
                Config::new(&stringify!($service).replace("_","-"))
                    .unwrap()
                    .hosturl()
                    .unwrap()                  
            };
            // static ref APP_URL: String = {
            //     Config::new(&stringify!($app).replace("_","-"))
            //         .unwrap()
            //         .hosturl()
            //         .unwrap()                  
            // };
        }

        pub struct $struct {}
        impl $struct {
            $(
                pub fn $func($($msg:$cmd),*) -> Result<$rep> {
                    let app_url = "0.0.0.0:0".to_string();
                    // let app_url = APP_URL.to_string();
                    let connection = Connection::from_path(app_url,HOST_URL.to_string());
                    let mut command = Command::serialize(CommandID::$type,($($msg),*))?;
                    // command.insert(0,0);
                    debug!("Command: {:?}", command);
                    // match connection.transfer_timeout(command,std::time::Duration::from_secs($timeout)) {
                    //     Ok(response) => {
                    //         debug!("Response: {:?}", response);
                    //         match Command::<CommandID,$rep>::parse(&response) {
                    //             Ok(c) => Ok(c.data),
                    //             Err(e) => Err(e.into()),
                    //         }
                    //     },
                    //     Err(e) => Err(e.into()),
                    // }
                    match Command::<CommandID,$rep>::parse(&connection.transfer_timeout(command,std::time::Duration::from_secs(1))?) {
                        Ok(c) => Ok(c.data),
                        Err(e) => Err(e),
                    }                
                }
            )*
        }       
    }
}
