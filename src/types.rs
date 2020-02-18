// Copyright 2020 Mathias Kraus - All rights reserved
//
// Licensed under the Apache License, Version 2.0 <LICENSE or
// http://www.apache.org/licenses/LICENSE-2.0>. This file may not be
// copied, modified, or distributed except according to those terms.

use std::str::FromStr;

#[derive(Debug)]
pub enum Pages {
    Overview,
    Memory,
    Processes,
    Services,
}

impl FromStr for Pages {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "Overview" => Ok(Pages::Overview),
            "Memory" => Ok(Pages::Memory),
            "Processes" => Ok(Pages::Processes),
            "Services" => Ok(Pages::Services),
            _ => Err("Could not parse page type!".to_string()),
        }
    }
}
