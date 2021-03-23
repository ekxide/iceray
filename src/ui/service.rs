// Copyright 2020 Mathias Kraus - All rights reserved
//
// Licensed under the Apache License, Version 2.0 <LICENSE or
// http://www.apache.org/licenses/LICENSE-2.0>. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::app::App;

use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Paragraph, Wrap};
use tui::Frame;

pub fn draw<B>(frame: &mut Frame<B>, area: Rect, app: &App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
        .split(area);

    draw_service_list(frame, chunks[0], app);
    draw_service_details(frame, chunks[1], app);
}

pub fn draw_service_list<B>(frame: &mut Frame<B>, area: Rect, app: &App)
where
    B: Backend,
{
    let mut text = Vec::<Spans>::new();

    text.push(Spans::from(vec![Span::raw("")]));

    for (index, (service, _)) in app.services.map.iter().enumerate() {
        let style = if app.services.selection.0 == index {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        text.push(Spans::from(vec![Span::styled(
            format!(
                "{} • {} • {}",
                (*service).service_id,
                (*service).instance_id,
                (*service).event_id
            ),
            style,
        )]));
    }

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Service List"))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

pub fn draw_service_details<B>(frame: &mut Frame<B>, area: Rect, app: &App)
where
    B: Backend,
{
    let mut text = Vec::<Spans>::new();

    text.push(Spans::from(vec![Span::raw("")]));

    if let Some(details) = app.services.map.get(&app.services.selection.1) {
        text.push(Spans::from(vec![
            Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!(
                "{} • {} • {}",
                app.services.selection.1.service_id,
                app.services.selection.1.instance_id,
                app.services.selection.1.event_id,
            )),
        ]));

        text.push(Spans::from(vec![
            Span::styled(
                format!("Processes with corresponding Publisher Ports: "),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{}", details.publisher_processes.len())),
        ]));

        for process in details.publisher_processes.iter() {
            text.push(Spans::from(vec![Span::raw(format!(" • {}", process))]));
        }

        text.push(Spans::from(vec![
            Span::styled(
                format!("Processes with corresponding Subscriber Ports: ",),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{}", details.subscriber_processes.len())),
        ]));

        for process in details.subscriber_processes.iter() {
            text.push(Spans::from(vec![Span::raw(format!(" • {}", process))]));
        }
    }

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Service Details"),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}
