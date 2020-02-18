// Copyright 2020 Mathias Kraus - All rights reserved
//
// Licensed under the Apache License, Version 2.0 <LICENSE or
// http://www.apache.org/licenses/LICENSE-2.0>. This file may not be
// copied, modified, or distributed except according to those terms.

use iceoryx_rs::introspection::{
    MemPoolIntrospectionTopic, PortIntrospectionTopic, ProcessIntrospectionTopic,
    ServiceDescription,
};
use iceoryx_rs::sb::st::{Sample, SampleReceiver};

use termion::event::{Key, MouseEvent};

use std::collections::BTreeMap;
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

pub struct ProcessDetails {
    pub pid: i32,
    pub sender_ports: Vec<ServiceDescription>,
    pub receiver_ports: Vec<ServiceDescription>,
    pub runnables: Vec<String>,
}

pub struct ProcessList {
    sample_receiver: SampleReceiver<ProcessIntrospectionTopic>,
    pub list: Option<Sample<ProcessIntrospectionTopic>>, //TODO use HashMap
    pub map: BTreeMap<String, ProcessDetails>,
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
            map: BTreeMap::new(),
            selection: (0, "".to_string()),
        }
    }

    pub fn update(&mut self) {
        if let Some(list) = self.sample_receiver.get_sample() {
            self.map.clear();

            list.processes().into_iter().for_each(|process| {
                if let Some(process_name) = process.name() {
                    let details = self.map.entry(process_name).or_insert(ProcessDetails {
                        pid: process.pid(),
                        sender_ports: Vec::new(),
                        receiver_ports: Vec::new(),
                        runnables: Vec::new(),
                    });
                    // details.runnables.push(runnable);
                }
            });

            // check if selection is still at the right position
            let found = self
                .map
                .keys()
                .nth(self.selection.0)
                .map_or(false, |key| *key == self.selection.1);

            if !found {
                if let Some((index, key)) = self
                    .map
                    .keys()
                    .enumerate()
                    .find(|(_, key)| **key == self.selection.1)
                {
                    self.selection.0 = index;
                    self.selection.1 = key.clone();
                } else {
                    self.set_selection(self.selection.0);
                }
            }
        }
    }

    fn set_selection(&mut self, index: usize) {
        // check if out of bounds
        let mut index = index;
        let service_count = self.map.len();
        if index >= service_count {
            index = if service_count > 0 {
                service_count - 1
            } else {
                0
            };
        }

        if let Some(key) = self.map.keys().nth(index) {
            self.selection.0 = index;
            self.selection.1 = key.clone();
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

pub struct ServiceDetails {
    pub sender_processes: Vec<String>,
    pub receiver_processes: Vec<String>,
}

pub struct ServiceList {
    sample_receiver: SampleReceiver<PortIntrospectionTopic>,
    pub map: BTreeMap<ServiceDescription, ServiceDetails>,
    pub selection: (usize, ServiceDescription),
}

impl ServiceList {
    pub fn new() -> Self {
        let topic = PortIntrospectionTopic::new();
        const CACHE_SIZE: u32 = 1;
        let (subscriber, sample_receive_token) = topic.subscribe(CACHE_SIZE);

        Self {
            sample_receiver: subscriber.get_sample_receiver(sample_receive_token),
            map: BTreeMap::new(),
            selection: (0, ServiceDescription::default()),
        }
    }

    pub fn update(&mut self, processes: &mut ProcessList) {
        if let Some(ports) = self.sample_receiver.get_sample() {
            self.map.clear();
            for (_, process_details) in processes.map.iter_mut() {
                process_details.sender_ports.clear();
                process_details.receiver_ports.clear();
                process_details.runnables.clear();
            }

            ports.sender_ports().into_iter().for_each(|sender| {
                if let Some(service_description) = sender.service_description() {
                    let details = self
                        .map
                        .entry(service_description.clone())
                        .or_insert(ServiceDetails {
                            sender_processes: Vec::new(),
                            receiver_processes: Vec::new(),
                        });
                    if let Some(process_name) = sender.process_name() {
                        if let Some(process_details) = processes.map.get_mut(&process_name).as_mut() {
                            process_details.sender_ports.push(service_description);
                        }
                        details.sender_processes.push(process_name);
                    }
                }
            });

            ports.receiver_ports().into_iter().for_each(|receiver| {
                if let Some(service_description) = receiver.service_description() {
                    let details = self
                        .map
                        .entry(service_description.clone())
                        .or_insert(ServiceDetails {
                            sender_processes: Vec::new(),
                            receiver_processes: Vec::new(),
                        });
                    if let Some(process_name) = receiver.process_name() {
                        if let Some(process_details) = processes.map.get_mut(&process_name).as_mut() {
                            process_details.receiver_ports.push(service_description);
                        }
                        details.receiver_processes.push(process_name);
                    }
                }
            });

            // check if selection is still at the right position
            let found = self
                .map
                .keys()
                .nth(self.selection.0)
                .map_or(false, |key| *key == self.selection.1);

            if !found {
                if let Some((index, key)) = self
                    .map
                    .keys()
                    .enumerate()
                    .find(|(_, key)| **key == self.selection.1)
                {
                    self.selection.0 = index;
                    self.selection.1 = key.clone();
                } else {
                    self.set_selection(self.selection.0);
                }
            }
        }
    }

    fn set_selection(&mut self, index: usize) {
        // check if out of bounds
        let mut index = index;
        let service_count = self.map.len();
        if index >= service_count {
            index = if service_count > 0 {
                service_count - 1
            } else {
                0
            };
        }

        if let Some(key) = self.map.keys().nth(index) {
            self.selection.0 = index;
            self.selection.1 = key.clone();
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
    pub services: ServiceList,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str) -> Self {
        App {
            title,
            should_quit: false,
            mouse_hold_position: None,
            tabs: TabsState::new(vec!["Overview", "Memory", "Processes", "Services"]),

            memory: MemorySegments::new(),
            processes: ProcessList::new(),
            services: ServiceList::new(),
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
                3 => self.services.selection_previous(),
                _ => (),
            },
            Key::Down => match self.tabs.index {
                1 => self.memory.selection_next(),
                2 => self.processes.selection_next(),
                3 => self.services.selection_next(),
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
        self.services.update(&mut self.processes);
    }
}
