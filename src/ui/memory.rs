// Copyright 2020 Mathias Kraus - All rights reserved
//
// Licensed under the Apache License, Version 2.0 <LICENSE or
// http://www.apache.org/licenses/LICENSE-2.0>. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::app::{App, USED_CHUNKS_HISTORY_SIZE};

use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, Text};
use tui::Frame;

use tui::widgets::canvas::{Canvas, Line};

pub fn draw<B>(frame: &mut Frame<B>, area: Rect, app: &App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(80), Constraint::Min(0)].as_ref())
        .split(area);

    draw_mempool_segments(frame, chunks[0], app);
    draw_graph(frame, chunks[1], app);
}

pub fn draw_mempool_segments<B>(frame: &mut Frame<B>, area: Rect, app: &App)
where
    B: Backend,
{
    let mut text = Vec::<Text>::new();

    let sample = if let Some(sample) = app.memory.segments.as_ref() {
        sample
    } else {
        return;
    };

    (*sample).memory_segments().into_iter().for_each(|segment| {
        let segment_id = segment.segment_id();
        text.push(Text::raw("\n"));
        text.push(Text::styled(
            format!("Segment {}", segment_id),
            Style::default().modifier(Modifier::BOLD),
        ));
        text.push(Text::raw(format!(
            " [writer: {} - reader: {}]\n",
            segment.writer_group().unwrap_or("##Error##".to_string()),
            segment.reader_group().unwrap_or("##Error##".to_string())
        )));

        text.push(Text::raw("\n"));

        text.push(Text::styled(
            "  MemPool | Chunks In Use |    Total | Min Free | Chunk Size | Payload Size\n",
            Style::default().modifier(Modifier::BOLD),
        ));

        text.push(Text::raw(
            "  -------------------------------------------------------------------------\n",
        ));

        segment
            .mempools()
            .into_iter()
            .enumerate()
            .for_each(|(index, mempool)| {
                let style = if app.memory.selection == (segment_id, index) {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                };

                let used_chunks_style =
                    if mempool.used_chunks as f64 / mempool.total_number_of_chunks as f64 > 0.95 {
                        Style::default().fg(Color::Red)
                    } else {
                        style
                    };
                let min_free_chunks_style = if mempool.min_free_chunks as f64
                    / mempool.total_number_of_chunks as f64
                    > 0.05
                {
                    style
                } else {
                    Style::default().fg(Color::Red)
                };

                text.push(Text::styled(format!("  {:>7} | ", index,), style));
                text.push(Text::styled(
                    format!("{:>13}", mempool.used_chunks,),
                    used_chunks_style,
                ));
                text.push(Text::styled(
                    format!(" | {:>8} | ", mempool.total_number_of_chunks),
                    style,
                ));
                text.push(Text::styled(
                    format!("{:>8}", mempool.min_free_chunks),
                    min_free_chunks_style,
                ));
                text.push(Text::styled(
                    format!(
                        " | {:>10} | {:>12}\n",
                        mempool.chunk_size, mempool.payload_size
                    ),
                    style,
                ));
            });
    });

    let paragraph = Paragraph::new(text.iter())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Segment & MemPool Info")
                .title_style(Style::default()),
        )
        .wrap(false);

    frame.render_widget(paragraph, area);
}

pub fn draw_graph<B>(frame: &mut Frame<B>, area: Rect, app: &App)
where
    B: Backend,
{
    let (segment, mempool) = app.memory.selection;

    let chart_title = format!("Chunks In Use [Segment {} - MemPool {}]", segment, mempool);

    let bottom = -1.0;
    let left = 0.0;
    let top = 101.0;
    let right = USED_CHUNKS_HISTORY_SIZE as f64;

    let canvas = Canvas::default()
        .block(
            Block::default()
                .title(&chart_title)
                .title_style(Style::default())
                .borders(Borders::ALL),
        )
        .paint(|ctx| {
            if area.width < 4 || area.height < 4 {
                return;
            }
            if let Some(history) = app.memory.used_chunks_history.get(&(segment, mempool)) {
                let mut pos = USED_CHUNKS_HISTORY_SIZE - history.len();
                let mut last = None;
                history.iter().for_each(|value| {
                    if let Some(last) = last {
                        ctx.draw(&Line {
                            x1: pos as f64,
                            y1: last,
                            x2: pos as f64,
                            y2: *value,
                            color: Color::Yellow,
                        });
                    }
                    let pos_next = pos + 1;
                    ctx.draw(&Line {
                        x1: pos as f64,
                        y1: *value,
                        x2: pos_next as f64,
                        y2: *value,
                        color: Color::Yellow,
                    });
                    pos = pos_next;
                    last = Some(*value);
                });
            }
            ctx.print(left, top, "100%", Color::White);
            ctx.print(left, bottom + (top - bottom) / 2.0, "50%", Color::White);
            ctx.print(left, bottom, "0%", Color::White);
        })
        .x_bounds([left, right])
        .y_bounds([bottom, top]);

    frame.render_widget(canvas, area);
}
