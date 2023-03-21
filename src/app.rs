#[macro_export]
macro_rules! app_macro{
    (
        $service: tt: $struct: tt {
            $(            
                $(query)?$(mutation)?: $type: ident => fn $func: tt (&$(mut )?self $(,$msg: tt: $cmd: ty)*) -> $ign1: tt<$rep: ty> $(; out: $gql_q: ty)?;
            )*
        }
    ) => {
        use crate::rust_udp::Connection;
        use cubeos_service::command_id;

        command_id!{
            $($type,)*
        }        

        lazy_static! {
            static ref HOST_URL: String = {
                Config::new(stringify!($service))
                    .unwrap()
                    .hosturl()
                    .unwrap()                    
            };
        }

        pub struct $struct {}
        impl $struct {
            $(
                pub fn $func($($msg:$cmd),*) -> Result<$rep> {
                    let app_url = "0.0.0.0:0".to_string();
                    let connection = Connection::from_path(app_url,HOST_URL.to_string());
                    match Command::<CommandID,$rep>::parse(&connection.transfer(Command::serialize(CommandID::$type,($($msg),*))?)?) {
                        Ok(c) => Ok(c.data),
                        Err(e) => Err(e),
                    }                
                }
            )*
        }       
    }
}