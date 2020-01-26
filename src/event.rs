// Copyright 2020 Mathias Kraus - All rights reserved
//
// Licensed under the Apache License, Version 2.0 <LICENSE or
// http://www.apache.org/licenses/LICENSE-2.0>. This file may not be
// copied, modified, or distributed except according to those terms.

// TODO add mandatory MIT license notice
// from https://github.com/fdehau/tui-rs/blob/master/examples/util/event.rs

use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use termion::event::Key;
use termion::event::MouseEvent;
use termion::input::TermRead;

pub enum Event<I, M> {
    Input(I),
    Mouse(M),
    Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<Event<Key, MouseEvent>>,
    _input_handle: thread::JoinHandle<()>,
    _tick_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: Key,
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exit_key: Key::Char('q'),
            tick_rate: Duration::from_millis(250),
        }
    }
}

impl Events {
    pub fn new(config: Config) -> Self {
        let (tx, rx) = mpsc::channel();
        let input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.events() {
                    match evt {
                        Ok(termion::event::Event::Key(key)) => {
                            if tx.send(Event::Input(key)).is_err() {
                                return;
                            }
                            if key == config.exit_key {
                                return;
                            }
                        }
                        Ok(termion::event::Event::Mouse(mouse)) => {
                            if tx.send(Event::Mouse(mouse)).is_err() {
                                return;
                            }
                        }
                        _ => {}
                    }
                }
            })
        };
        let tick_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let tx = tx.clone();
                loop {
                    tx.send(Event::Tick).unwrap();
                    thread::sleep(config.tick_rate);
                }
            })
        };
        Events {
            rx,
            _input_handle: input_handle,
            _tick_handle: tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<Key, MouseEvent>, mpsc::RecvError> {
        self.rx.recv()
    }
}
