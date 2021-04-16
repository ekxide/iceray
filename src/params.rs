// SPDX-License-Identifier: Apache-2.0

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
