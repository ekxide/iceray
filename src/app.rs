// SPDX-License-Identifier: Apache-2.0

use iceoryx_rs::introspection::{
    MemPoolIntrospection, MemPoolIntrospectionTopic, PortIntrospection, PortIntrospectionTopic,
    ProcessIntrospection, ProcessIntrospectionTopic, ServiceDescription,
};
use iceoryx_rs::st::{Sample, SampleReceiver};

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
    pub segments: Option<Sample<MemPoolIntrospectionTopic>>,
    pub used_chunks_history: HashMap<(u32, usize), VecDeque<f64>>,
    pub selection: (u32, usize),
}

impl MemorySegments {
    pub fn new() -> Self {
        let inactive_sub = MemPoolIntrospection::new().expect("Mempool introspection subscriber");
        let (subscriber, sample_receive_token) = inactive_sub.subscribe();

        Self {
            sample_receiver: subscriber.get_sample_receiver(sample_receive_token),
            segments: None,
            used_chunks_history: HashMap::with_capacity(USED_CHUNKS_HISTORY_SIZE),
            selection: (0, 0),
        }
    }

    pub fn update(&mut self) {
        if let Some(sample) = self.sample_receiver.take() {
            // update history
            sample
                .memory_segments()
                .into_iter()
                .for_each(|memory_segment| {
                    memory_segment.mempools().into_iter().enumerate().for_each(
                        |(index, mempool)| {
                            let history = self
                                .used_chunks_history
                                .entry((memory_segment.segment_id(), index))
                                .or_insert(VecDeque::new());

                            if history.len() >= USED_CHUNKS_HISTORY_SIZE {
                                history.drain(0..1);
                            }

                            history.push_back(
                                mempool.used_chunks as f64 / mempool.total_number_of_chunks as f64
                                    * 100f64,
                            );
                        },
                    )
                });

            self.segments = Some(sample);
        }
    }

    fn selection_next(&mut self) {
        let sample = if let Some(sample) = self.segments.as_ref() {
            sample
        } else {
            return;
        };

        let mut next_segment = self.selection.0;
        let mut next_mempool = self.selection.1 as usize + 1;
        while let Some(segment) = sample
            .memory_segments()
            .into_iter()
            .nth(next_segment as usize)
        {
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

        let sample = if let Some(sample) = self.segments.as_ref() {
            sample
        } else {
            return;
        };

        if let Some(segment) = sample
            .memory_segments()
            .into_iter()
            .nth(self.selection.0 as usize - 1)
        {
            let number_of_mempools = segment.mempools().into_iter().size_hint().0;
            if number_of_mempools > 0 {
                self.selection = (self.selection.0 - 1, number_of_mempools - 1);
            }
        }
    }
}

pub struct ProcessDetails {
    pub pid: i32,
    pub publisher_ports: Vec<ServiceDescription>,
    pub subscriber_ports: Vec<ServiceDescription>,
    pub nodes: Vec<String>,
}

pub struct ProcessList {
    sample_receiver: SampleReceiver<ProcessIntrospectionTopic>,
    pub list: Option<Sample<ProcessIntrospectionTopic>>, //TODO use HashMap
    pub map: BTreeMap<String, ProcessDetails>,
    pub selection: (usize, String),
}

impl ProcessList {
    pub fn new() -> Self {
        let inactive_sub = ProcessIntrospection::new().expect("Process introspection subscriber");
        let (subscriber, sample_receive_token) = inactive_sub.subscribe();

        Self {
            sample_receiver: subscriber.get_sample_receiver(sample_receive_token),
            list: None,
            map: BTreeMap::new(),
            selection: (0, "".to_string()),
        }
    }

    pub fn update(&mut self) {
        if let Some(list) = self.sample_receiver.take() {
            self.map.clear();

            list.processes().into_iter().for_each(|process| {
                if let Some(process_name) = process.name() {
                    let _details = self.map.entry(process_name).or_insert(ProcessDetails {
                        pid: process.pid(),
                        publisher_ports: Vec::new(),
                        subscriber_ports: Vec::new(),
                        nodes: Vec::new(),
                    });
                    // details.nodes.push(node);
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
    pub publisher_processes: Vec<String>,
    pub subscriber_processes: Vec<String>,
}

pub struct ServiceList {
    sample_receiver: SampleReceiver<PortIntrospectionTopic>,
    pub map: BTreeMap<ServiceDescription, ServiceDetails>,
    pub selection: (usize, ServiceDescription),
}

impl ServiceList {
    pub fn new() -> Self {
        let inactive_sub = PortIntrospection::new().expect("Port introspection subscriber");
        let (subscriber, sample_receive_token) = inactive_sub.subscribe();

        Self {
            sample_receiver: subscriber.get_sample_receiver(sample_receive_token),
            map: BTreeMap::new(),
            selection: (0, ServiceDescription::default()),
        }
    }

    pub fn update(&mut self, processes: &mut ProcessList) {
        if let Some(ports) = self.sample_receiver.take() {
            self.map.clear();
            for (_, process_details) in processes.map.iter_mut() {
                process_details.publisher_ports.clear();
                process_details.subscriber_ports.clear();
                process_details.nodes.clear();
            }

            ports.publisher_ports().into_iter().for_each(|publisher| {
                if let Some(service_description) = publisher.service_description() {
                    let details =
                        self.map
                            .entry(service_description.clone())
                            .or_insert(ServiceDetails {
                                publisher_processes: Vec::new(),
                                subscriber_processes: Vec::new(),
                            });
                    if let Some(process_name) = publisher.process_name() {
                        if let Some(process_details) = processes.map.get_mut(&process_name).as_mut()
                        {
                            process_details.publisher_ports.push(service_description);
                        }
                        details.publisher_processes.push(process_name);
                    }
                }
            });

            ports.subscriber_ports().into_iter().for_each(|subscriber| {
                if let Some(service_description) = subscriber.service_description() {
                    let details =
                        self.map
                            .entry(service_description.clone())
                            .or_insert(ServiceDetails {
                                publisher_processes: Vec::new(),
                                subscriber_processes: Vec::new(),
                            });
                    if let Some(process_name) = subscriber.process_name() {
                        if let Some(process_details) = processes.map.get_mut(&process_name).as_mut()
                        {
                            process_details.subscriber_ports.push(service_description);
                        }
                        details.subscriber_processes.push(process_name);
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
