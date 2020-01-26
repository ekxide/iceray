// Copyright 2020 Mathias Kraus - All rights reserved
//
// Licensed under the Apache License, Version 2.0 <LICENSE or
// http://www.apache.org/licenses/LICENSE-2.0>. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::types::Pages;

use structopt::StructOpt;

/// iceray - iceoryx introspection
#[derive(StructOpt, Debug)]
#[structopt(name = "iceray")]
pub struct Params {
    /// The update interal in  milliseonds
    #[structopt(short, long, default_value = "1000")]
    pub update_interval: u64,
    /// The initial page to show on startup
    #[structopt(short, long, default_value = "Memory")]
    pub initial_page: Pages,
}
