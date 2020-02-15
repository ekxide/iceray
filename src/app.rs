// Copyright 2020 Mathias Kraus - All rights reserved
//
// Licensed under the Apache License, Version 2.0 <LICENSE or
// http://www.apache.org/licenses/LICENSE-2.0>. This file may not be
// copied, modified, or distributed except according to those terms.

use iceoryx_rs::introspection::{MemPoolIntrospectionTopic, ProcessIntrospectionTopic};
use iceoryx_rs::sb::st::{Sample, SampleReceiver};

use termion::event::{Key, MouseEvent};

use std::collections::HashMap;
use std::collections::VecDeque;

pub struct TabsState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> TabsState<'a> {
    pub fn new(titles: Vec<&'a str>) -> TabsState {
        TabsState { titles, index: 0 }
    }
    pub fn next(&mut self) {
        let maybe_next_index = self.index + 1;
        if maybe_next_index < self.titles.len() {
            self.index = maybe_next_index;
        }
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }
}

pub const USED_CHUNKS_HISTORY_SIZE: usize = 120;

pub struct MemorySegments {
    sample_receiver: SampleReceiver<MemPoolIntrospectionTopic>,
    pub segments: VecDeque<Sample<MemPoolIntrospectionTopic>>,
    pub used_chunks_history: HashMap<(u32, usize), VecDeque<f64>>,
    pub selection: (u32, usize),
}

impl MemorySegments {
    pub fn new() -> Self {
        let topic = MemPoolIntrospectionTopic::new();
        const CACHE_SIZE: u32 = 101;
        let (subscriber, sample_receive_token) = topic.subscribe(CACHE_SIZE);

        Self {
            sample_receiver: subscriber.get_sample_receiver(sample_receive_token),
            segments: VecDeque::new(),
            used_chunks_history: HashMap::with_capacity(USED_CHUNKS_HISTORY_SIZE),
            selection: (0, 0),
        }
    }

    pub fn update(&mut self) {
        while let Some(sample) = self.sample_receiver.get_sample() {
            // update history
            sample
                .mempools()
                .into_iter()
                .enumerate()
                .for_each(|(index, mempool)| {
                    let history = self
                        .used_chunks_history
                        .entry((sample.segment_id(), index))
                        .or_insert(VecDeque::new());

                    if history.len() >= USED_CHUNKS_HISTORY_SIZE {
                        history.drain(0..1);
                    }

                    history.push_back(
                        mempool.used_chunks as f64 / mempool.total_number_of_chunks as f64 * 100f64,
                    );
                });

            // check if outdated segment is in the queue
            if let Some(front) = self.segments.front() {
                if front.segment_id() == sample.segment_id() {
                    self.segments.drain(0..1);
                }
            }
            self.segments.push_back(sample);
        }
    }

    fn selection_next(&mut self) {
        let mut next_segment = self.selection.0;
        let mut next_mempool = self.selection.1 as usize + 1;
        while let Some(segment) = self.segments.get(next_segment as usize) {
            let number_of_mempools = segment.mempools().into_iter().size_hint().0;
            if number_of_mempools > next_mempool {
                self.selection = (next_segment, next_mempool);
                return;
            }
            next_segment += 1;
            next_mempool = 0;
        }
    }

    fn selection_previous(&mut self) {
        if self.selection.1 > 0 {
            self.selection.1 = self.selection.1 - 1;
            return;
        }

        if self.selection.0 == 0 {
            return;
        }

        if let Some(segment) = self.segments.get(self.selection.0 as usize - 1) {
            let number_of_mempools = segment.mempools().into_iter().size_hint().0;
            if number_of_mempools > 0 {
                self.selection = (self.selection.0 - 1, number_of_mempools - 1);
            }
        }
    }
}

pub struct ProcessList {
    sample_receiver: SampleReceiver<ProcessIntrospectionTopic>,
    pub list: Option<Sample<ProcessIntrospectionTopic>>,
    pub selection: (usize, String),
}

impl ProcessList {
    pub fn new() -> Self {
        let topic = ProcessIntrospectionTopic::new();
        const CACHE_SIZE: u32 = 1;
        let (subscriber, sample_receive_token) = topic.subscribe(CACHE_SIZE);

        Self {
            sample_receiver: subscriber.get_sample_receiver(sample_receive_token),
            list: None,
            selection: (0, "".to_string()),
        }
    }

    pub fn update(&mut self) {
        if let Some(list) = self.sample_receiver.get_sample() {
            // check if selection index needs an update because the list changed
            let mut found = list.get_process(self.selection.0).map_or(false, |process| {
                process
                    .name()
                    .map_or(false, |name| name == self.selection.1)
            });

            // brute force if not found
            if !found {
                let process_count = list.process_count();
                for index in 0..process_count {
                    if let Some(process) = list.get_process(index) {
                        if process
                            .name()
                            .map_or(false, |name| name == self.selection.1)
                        {
                            self.selection.0 = index;
                            found = true;
                            break;
                        }
                    }
                }
            }

            // set the new list, since set_selection needs to access the new list if the process selection was outdated
            self.list = Some(list);
            // if still not found, select a new process
            if !found {
                self.set_selection(self.selection.0);
            }
        }
    }

    fn set_selection(&mut self, index: usize) {
        if let Some(list) = self.list.as_ref() {
            let process_count = list.process_count();
            self.selection.0 = index;
            // check if out of bounds
            if self.selection.0 >= process_count {
                self.selection.0 = if process_count > 0 {
                    process_count - 1
                } else {
                    0
                };
            }
            if let Some(name) = list
                .get_process(self.selection.0)
                .and_then(|process| process.name())
            {
                self.selection.1 = name;
            } else {
                // this should actually never happen
                self.selection.1 = "##error##".to_string();
            }
        }
    }

    fn selection_next(&mut self) {
        self.set_selection(self.selection.0 + 1);
    }

    fn selection_previous(&mut self) {
        if self.selection.0 > 0 {
            self.set_selection(self.selection.0 - 1);
        }
    }
}

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub mouse_hold_position: Option<(u16, u16)>,
    pub tabs: TabsState<'a>,

    pub memory: MemorySegments,
    pub processes: ProcessList,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str) -> Self {
        App {
            title,
            should_quit: false,
            mouse_hold_position: None,
            tabs: TabsState::new(vec!["Overview", "Memory", "Processes", "Ports"]),

            memory: MemorySegments::new(),
            processes: ProcessList::new(),
        }
    }

    pub fn on_key(&mut self, k: Key) {
        match k {
            Key::Char('q') => {
                self.should_quit = true;
            }
            Key::Right => {
                self.tabs.next();
            }
            Key::Left => {
                self.tabs.previous();
            }
            Key::Up => match self.tabs.index {
                1 => self.memory.selection_previous(),
                2 => self.processes.selection_previous(),
                _ => (),
            },
            Key::Down => match self.tabs.index {
                1 => self.memory.selection_next(),
                2 => self.processes.selection_next(),
                _ => (),
            },
            _ => {}
        }
    }

    pub fn on_mouse(&mut self, m: MouseEvent) {
        match m {
            MouseEvent::Press(_, x, y) => self.mouse_hold_position = Some((x, y)),
            MouseEvent::Hold(x, y) => self.mouse_hold_position = Some((x, y)),
            _ => self.mouse_hold_position = None,
        }
    }

    pub fn on_tick(&mut self) {
        self.memory.update();
        self.processes.update();
    }
}
