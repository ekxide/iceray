// Copyright 2020 Mathias Kraus - All rights reserved
//
// Licensed under the Apache License, Version 2.0 <LICENSE or
// http://www.apache.org/licenses/LICENSE-2.0>. This file may not be
// copied, modified, or distributed except according to those terms.

use iceoryx_rs::introspection::MemPoolIntrospectionTopic;
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
    pub fn new(sample_receiver: SampleReceiver<MemPoolIntrospectionTopic>) -> Self {
        Self {
            sample_receiver,
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

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub mouse_hold_position: Option<(u16, u16)>,
    pub tabs: TabsState<'a>,

    pub memory: MemorySegments,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str) -> Self {
        let topic = MemPoolIntrospectionTopic::new();
        const CACHE_SIZE: u32 = 101;
        let (subscriber, sample_receive_token) = topic.subscribe(CACHE_SIZE);
        App {
            title,
            should_quit: false,
            mouse_hold_position: None,
            tabs: TabsState::new(vec!["Overview", "Memory", "Processes", "Ports"]),

            memory: MemorySegments::new(subscriber.get_sample_receiver(sample_receive_token)),
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
            Key::Up => {
                self.memory.selection_previous();
            }
            Key::Down => {
                self.memory.selection_next();
            }
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
    }
}
