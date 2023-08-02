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

// Define which implementation of the Service and service-macro!
// are used depending on the selected feature

#[cfg(not(any(feature = "ground",feature = "terminal",feature = "app")))]
mod udp;
#[cfg(not(any(feature = "ground",feature = "terminal",feature = "app")))]
mod udp_macro;
#[cfg(not(any(feature = "ground",feature = "terminal",feature = "app")))]
pub use udp::{Context,Service};

#[cfg(feature = "ground")]
mod ground;
#[cfg(feature = "ground")]
mod ground_macro;
#[cfg(feature = "ground")]
pub use ground::{Context,Service,UdpPassthrough};

#[cfg(feature = "terminal")]
mod terminal;
#[cfg(feature = "terminal")]
mod terminal_macro;
#[cfg(feature = "terminal")]
pub use terminal::{Context,Service,UdpPassthrough};