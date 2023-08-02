// SPDX-License-Identifier: Apache-2.0

use crate::app::{App, USED_CHUNKS_HISTORY_SIZE};

use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use ratatui::widgets::canvas::{Canvas, Line as CanvasLine};

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
    let mut text = Vec::<Line>::new();

    let sample = if let Some(sample) = app.memory.segments.as_ref() {
        sample
    } else {
        return;
    };

    (*sample).memory_segments().into_iter().for_each(|segment| {
        let segment_id = segment.segment_id();
        text.push(Line::from(vec![Span::raw("")]));
        text.push(Line::from(vec![
            Span::styled(
                format!("Segment {}", segment_id),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(
                " [writer: {} - reader: {}]",
                segment.writer_group().unwrap_or("##Error##".to_string()),
                segment.reader_group().unwrap_or("##Error##".to_string())
            )),
        ]));

        text.push(Line::from(vec![Span::raw("")]));

        text.push(Line::from(vec![Span::styled(
            "  MemPool | Chunks In Use |    Total | Min Free | Chunk Size | Payload Size",
            Style::default().add_modifier(Modifier::BOLD),
        )]));

        text.push(Line::from(vec![Span::raw(
            "  -------------------------------------------------------------------------",
        )]));

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

                text.push(Line::from(vec![
                    Span::styled(format!("  {:>7} | ", index,), style),
                    Span::styled(format!("{:>13}", mempool.used_chunks,), used_chunks_style),
                    Span::styled(
                        format!(" | {:>8} | ", mempool.total_number_of_chunks),
                        style,
                    ),
                    Span::styled(
                        format!("{:>8}", mempool.min_free_chunks),
                        min_free_chunks_style,
                    ),
                    Span::styled(
                        format!(
                            " | {:>10} | {:>12}",
                            mempool.chunk_size, mempool.payload_size
                        ),
                        style,
                    ),
                ]));
            });
    });

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Segment & MemPool Info"),
        )
        .wrap(Wrap { trim: false });

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
                .borders(Borders::ALL)
                .title(&chart_title as &str),
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
                        ctx.draw(&CanvasLine {
                            x1: pos as f64,
                            y1: last,
                            x2: pos as f64,
                            y2: *value,
                            color: Color::Yellow,
                        });
                    }
                    let pos_next = pos + 1;
                    ctx.draw(&CanvasLine {
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
            ctx.print(
                left,
                top,
                Span::styled("100%", Style::default().fg(Color::White)),
            );
            ctx.print(
                left,
                bottom + (top - bottom) / 2.0,
                Span::styled("50%", Style::default().fg(Color::White)),
            );
            ctx.print(
                left,
                bottom,
                Span::styled("0%", Style::default().fg(Color::White)),
            );
        })
        .x_bounds([left, right])
        .y_bounds([bottom, top]);

    frame.render_widget(canvas, area);
}
