#[macro_export]
macro_rules! service_macro {
    (
        use $error: ty;
        $krate: tt ::$strukt: tt {
            $(
                $(query)?$(mutation)?: $type: ident => fn $func: tt (&$(mut )?self $(,$msg: tt: $cmd: ty)*) -> $ign1: tt<$rep: ty> $(; out: $gql_q: ty)?;
            )*
        }
    ) => {
        use webapp_macro::*;
        use yew::prelude::*;
        use yew_router::{history::History, prelude::RouterScopeExt};
        use wasm_bindgen::JsCast;
        use web_sys::{HtmlSelectElement, HtmlInputElement};
        use std::rc::Rc;
        use std::cell::RefCell;
        
        pub static INPUT: Lazy<Mutex<String>> = Lazy::new(Mutex::new("".to_string()));
        pub static OUTPUT: Lazy<Mutex<String>> = Lazy::new(Mutex::new("".to_string()));
        pub static SERVICEIP: String = "127.0.0.1:8082".to_string();

        // Add a proc_macro that builds the html inputs for different types

        #[derive(Clone, Routable, PartialEq)]
        pub enum Route {
            #[at("/")]
            Home,

            $(
                #[at("/$type_q")]
                $type_q,
            )*

            $(
                #[at("/$type_m")]
                $type_m,
            )*

            #[at("/Submit")]
            Submit,

            #[not_found]
            #[at("/404")]
            NotFound,
        }

        pub fn switch(routes: &Route) -> Html {
            match routes {
                Route::Home => html! {
                    <Home/>
                },
                $(
                    Route::$type_q => html! {
                        <$type_q/>
                    },
                )*
                $(
                    Route::$type_m => html! {
                        <$type_m/>
                    },
                )*
                Route::Submit => html! {
                    <Submit/>
                },
                Route::NotFound => html! {
                    <div class="container">
                        <p>{ "404" }</p>
                    </div>
                },
            }
        }

        pub struct Home {}

        impl Component for Home {
            type Message = ();
            type Properties = ();

            fn create(_ctx: &Context<Self>) -> Self {
                Self{}
            }
            
            fn view(&self, ctx: &Context<Self>) {
                let history = ctx.link().history().unwrap();
    
                html! {
                    <div class="home-container">
                    <div class="container">
                        <div class="background-image"></div>
                        <div class="image-container">
                        <img class="header-image" src="./images/cuava.png" 
                            alt="CUAVA logo"  
                            oncontextmenu={Callback::from(move |e: MouseEvent| {
                                e.prevent_default();
                            })}
                            ondragstart={Callback::from(move |e: DragEvent| {
                                e.prevent_default();
                            })}
                            />
                        </div>
                        <div class="button-container">
                        #(
                            <button onclick={ctx.link().callback(move |_| history.clone().push(#route))}>{#route_id} </button>
                        )*
                        </div>
                    </div>
                    </div>
                }
            }
    
            fn update(&self, _ctx: &Context<Self>, _msg: Message) -> bool {
                false
            }
        }

         // Submission branch
         pub enum Msg {
            StartFetch,
            FetchComplete(String),
            UpdateResult(String),
        }

        pub struct Submit {
            fetching: bool,
            show_loading: bool,
            result: Option<String>,
            fetch_completed: bool,
        }
        
        impl Component for Submit {
            type Message = Msg;
            type Properties = ();

            fn create(ctx: &Context<Self>) -> Self {
                Self {
                    fetching: false,
                    result: None,
                    show_loading: false,
                    fetch_completed: false,
                }
            }
        
            fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
                match msg {
                    Msg::StartFetch => {
                        self.fetching = true;
                        self.show_loading = true; 
                        self.fetch_completed = false; // set to false here
                        let link = ctx.link().clone();
                        spawn_local(async move {
                            let window = web_sys::window().expect("no global `window` exists");
                            let _ = window
                                .set_timeout_with_callback_and_timeout_and_arguments_0(
                                    &Closure::once_into_js(move || {
                                        link.send_message(Msg::FetchComplete("Waited long enough".to_string()))
                                    }).into(),
                                        3000)
                                .expect("unable to register timer");
                        });
                        true // return true after fetch has started
                    }
            
                    Msg::FetchComplete(result) => {
                        self.fetching = false; // This means the data has been fetched but not updated yet
                        let link = ctx.link().clone();
                        spawn_local(async move {
                            let window = web_sys::window().expect("no global `window` exists");
                            let _ = window
                                .set_timeout_with_callback_and_timeout_and_arguments_0(
                                    &Closure::once_into_js(move || {
                                        link.send_message(Msg::UpdateResult(result));
                                    }).into(),
                                        750)
                                    .expect("unable to register timer");
                        });
                        true
                    }
                    Msg::UpdateResult(result) => {
                        self.show_loading = false; //  This means the result is updated and the image can fade out
                        self.fetching = false;
                        self.fetch_completed = true;
                        self.result = Some(result);
                        true
                    }
                }
            }
        
            fn view(&self, ctx: &Context<Self>) -> Html {
                let input = INPUT.lock().unwrap();
                let output = OUTPUT.lock().unwrap();
                let history = ctx.link().history().unwrap();
                let history_clone = history.clone();
                let history_clone2 = history.clone();
                let onclick = Callback::once(move |_| {
                    // let input = INPUT.lock().unwrap();
                    history_clone.push(Route::Home).expect("AHHHH");
                });
                let onclick_home = Callback::once(move |_| {
                    *input = "".to_string();
                    history_clone2.push(Route::Home).expect("AHHHHH");
                });
                let ctx_clone = ctx.link().clone();
                let submit = Callback::once(move|_| { 
                    ctx_clone.send_message(Msg::StartFetch);
                    let socket: UdpSocket = UdpSocket.bind("0.0.0.0:0");
                    match socket.send_to(input.to_bytes(),SERVICE_IP) {
                        Ok(_) => {
                            let mut buf = [0u8; 1024];
                            match socket.recv(&mut buf) {
                                Ok(b) => {
                                    *output = String::from_utf8_lossy(&buf[..b]);
                                }
                                Err(e) => *output = format!("Error: {:?}", e),
                            }
                        }
                        Err(e) => *output = format!("Error: {:?}", e),
                    };
                    ctx_clone.send_message(Msg::FetchComplete);
                });
        
                // let app_state = APP_STATE.lock().unwrap();
        
                html! {
                    <div class="container">
                        <div class="background-image"></div>
                        <h1>{ "Display Results" }</h1>
                        <p>
                            { format!("Command: {}", INPUT)}
                            // {html_nested!{<br />}}
                            // {format!("input_text: {}." , app_state.input_text) }
                        </p>
                        <p>
                        {format!("Output: {}", OUTPUT)}
                        </p>
                        {
                            {
                                if self.show_loading {
                                    if self.fetching {
                                        html! { 
                                            <div class="image-container">
                                                <img class="loading-image" src="/images/loading.gif" />
                                            </div> 
                                        }
                                    } else {
                                        html! { 
                                            <div class="image-container">
                                                <img class="loading-image fade-out" src="/images/loading.gif" />
                                            </div> 
                                        }
                                    }
                                } else {
                                    if let Some(ref result) = self.result {
                                        html! { <p style="font-size:small;">{ result }</p> }
                                    } else {
                                        html! {}
                                    }
                                }
                            }
        
                        }
                        <div class="button-container">
                            <button class="button" onclick={submit}>{ "Submit" }</button>
                            <button class="button" disabled={!self.fetch_completed} onclick={ onclick.clone() } >{ "Log Data" }</button>
                            <button class="button" onclick={ onclick_home.clone() }>{ "Home" }</button>
                        </div>
                    </div>
                }
            }        

            $(
                #[derive(Debug, Clone, Serialize, Deserialize)]
                pub struct $type {
                    $(
                        $msg: $cmd,
                    )*
                }

                impl Component for $type {
                    type Message = HtmlMessage;
                    type Properties = ();

                    fn create(ctx: &Context<Self>) -> Self {
                        Self {
                            $(
                                $msg: $cmd::default(),
                            )*
                        }
                    }

                    fn view(&self, ctx: &Context<Self>) -> Html {
                        let history = ctx.link().history();
                        let onclick_next: Callback<MouseEvent> = Callback::from(move |_| {
                            history.clone().push(Route::Submit).expect("AAAAAAH");
                        });
                        let onclick_prev: Callback<MouseEvent> = Callback::from(move |_| {
                            history.clone().push(Route::Home).expect("AAAAAHAAAAA");
                        });

                        html!{
                            <div class="container">
                                <div class="background-image"></div>
                                <h1>{ stringify!($type) }</h1>
                                $(
                                    <div>
                                        <label>{stringify!($cmd)}</label>
                                        $cmd::html_input()
                                    </div>
                                )
                                <div class="button-container">
                                    <button class="button" onclick={onclick_prev}>{ "Back" }</button>
                                    <button class="button" disabled={#(#f_next)||*} onclick={onclick_next}>{ "Next" }</button>
                                </div>
                            </div>       
                        }
                    }

                    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
                        match msg {
                            $(
                                HtmlMessage::UpdateInput(msg) => {
                                    self = serde_json::from_str(&msg).unwrap();
                                    true
                                }
                            )*
                        }
                    }
                }
            )*
        }
    }
}