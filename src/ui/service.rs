// Copyright 2020 Mathias Kraus - All rights reserved
//
// Licensed under the Apache License, Version 2.0 <LICENSE or
// http://www.apache.org/licenses/LICENSE-2.0>. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::app::App;

use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, Text};
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
    let mut text = Vec::<Text>::new();

    text.push(Text::raw("\n"));

    for (index, (service, _)) in app.services.map.iter().enumerate() {
        let style = if app.services.selection.0 == index {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        text.push(Text::styled(
            format!(
                "{} • {} • {}\n",
                (*service).service_id,
                (*service).instance_id,
                (*service).event_id
            ),
            style,
        ));
    }

    let paragraph = Paragraph::new(text.iter())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Service List")
                .title_style(Style::default()),
        )
        .wrap(false);

    frame.render_widget(paragraph, area);
}

pub fn draw_service_details<B>(frame: &mut Frame<B>, area: Rect, app: &App)
where
    B: Backend,
{
    let mut text = Vec::<Text>::new();

    text.push(Text::raw("\n"));

    if let Some(details) = app.services.map.get(&app.services.selection.1) {
        text.push(Text::styled(
            "Name: ",
            Style::default().modifier(Modifier::BOLD),
        ));
        text.push(Text::raw(format!(
            "{} • {} • {}\n",
            app.services.selection.1.service_id,
            app.services.selection.1.instance_id,
            app.services.selection.1.event_id,
        )));

        text.push(Text::styled(
            format!("Processes with corresponding Sender Ports: "),
            Style::default().modifier(Modifier::BOLD),
        ));

        text.push(Text::raw(format!("{}\n", details.sender_processes.len())));

        for process in details.sender_processes.iter() {
            text.push(Text::raw(format!(" • {}\n", process)));
        }

        text.push(Text::styled(
            format!("Processes with corresponding Receiver Ports: ",),
            Style::default().modifier(Modifier::BOLD),
        ));

        text.push(Text::raw(format!("{}\n", details.receiver_processes.len())));

        for process in details.receiver_processes.iter() {
            text.push(Text::raw(format!(" • {}\n", process)));
        }
    }

    let paragraph = Paragraph::new(text.iter())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Service Details")
                .title_style(Style::default()),
        )
        .wrap(true);

    frame.render_widget(paragraph, area);
}
