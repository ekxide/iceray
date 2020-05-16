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

    draw_process_list(frame, chunks[0], app);
    draw_process_details(frame, chunks[1], app);
}

pub fn draw_process_list<B>(frame: &mut Frame<B>, area: Rect, app: &App)
where
    B: Backend,
{
    let mut text = Vec::<Text>::new();

    text.push(Text::raw("\n"));

    text.push(Text::styled(
        "   PID | Sender | Receiver | Runnables | Process Name\n",
        Style::default().modifier(Modifier::BOLD),
    ));

    text.push(Text::raw(
        " ----------------------------------------------------------\n",
    ));

    for (index, (process_name, details)) in app.processes.map.iter().enumerate() {
        let style = if app.processes.selection.0 == index {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        text.push(Text::styled(format!(" {:>5} | ", details.pid), style));
        text.push(Text::styled(
            format!("{:>6} | ", details.sender_ports.len()),
            style,
        ));
        text.push(Text::styled(
            format!("{:>8} | ", details.receiver_ports.len()),
            style,
        ));
        text.push(Text::styled(
            format!("{:>9} | ", details.runnables.len()),
            style,
        ));
        text.push(Text::styled(format!("{}\n", process_name), style));
    }

    let paragraph = Paragraph::new(text.iter())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Process List")
                .title_style(Style::default()),
        )
        .wrap(false);

    frame.render_widget(paragraph, area);
}

pub fn draw_process_details<B>(frame: &mut Frame<B>, area: Rect, app: &App)
where
    B: Backend,
{
    let mut text = Vec::<Text>::new();

    text.push(Text::raw("\n"));

    let process_name = &app.processes.selection.1;
    if let Some(details) = app.processes.map.get(process_name) {
        //     if let Some(process) = app
        //         .processes
        //         .list
        //         .as_ref()
        //         .and_then(|list| list.get_process(app.processes.selection.0))
        //     {
        text.push(Text::styled(
            " Name: ",
            Style::default().modifier(Modifier::BOLD),
        ));
        text.push(Text::raw(format!("{}\n", process_name)));

        text.push(Text::styled(
            " PID: ",
            Style::default().modifier(Modifier::BOLD),
        ));
        text.push(Text::raw(format!("{:}\n", details.pid)));

        text.push(Text::styled(
            " Sender Ports: ",
            Style::default().modifier(Modifier::BOLD),
        ));
        text.push(Text::raw(format!("{:}\n", details.sender_ports.len())));
        for port in details.sender_ports.iter() {
            text.push(Text::raw(format!(
                " • {} • {} • {}\n",
                port.service_id, port.instance_id, port.event_id
            )));
        }

        text.push(Text::styled(
            " Receiver Ports: ",
            Style::default().modifier(Modifier::BOLD),
        ));
        text.push(Text::raw(format!("{:}\n", details.receiver_ports.len())));
        for port in details.receiver_ports.iter() {
            text.push(Text::raw(format!(
                " • {} • {} • {}\n",
                port.service_id, port.instance_id, port.event_id
            )));
        }

        text.push(Text::styled(
            " Runnables: ",
            Style::default().modifier(Modifier::BOLD),
        ));
        text.push(Text::raw(format!("{:}\n", details.runnables.len())));
        if details.runnables.len() >= 1 {
            text.push(Text::styled(
                "     Listing runnables needs implementation!",
                Style::default().fg(Color::Red),
            ));
        }
    }

    let paragraph = Paragraph::new(text.iter())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Process Details")
                .title_style(Style::default()),
        )
        .wrap(true);

    frame.render_widget(paragraph, area);
}
