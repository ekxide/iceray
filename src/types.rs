// SPDX-License-Identifier: Apache-2.0

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
