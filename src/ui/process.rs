// Copyright 2020 Mathias Kraus - All rights reserved
//
// Licensed under the Apache License, Version 2.0 <LICENSE or
// http://www.apache.org/licenses/LICENSE-2.0>. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::app::App;

use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};
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
        "   PID | # Sender | # Receiver | # Runnables | Process Name\n",
        Style::default().modifier(Modifier::BOLD),
    ));

    text.push(Text::raw(
        " ----------------------------------------------------------\n",
    ));

    if let Some(list) = app.processes.list.as_ref() {
        list.processes()
            .into_iter()
            .enumerate()
            .for_each(|(index, process)| {
                let style = if app.processes.selection.0 == index {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                };

                text.push(Text::styled(format!(" {:>5} | ", process.pid()), style));
                text.push(Text::styled(format!("{:>8} | ", "?"), style));
                text.push(Text::styled(format!("{:>10} | ", "?"), style));
                text.push(Text::styled(
                    format!("{:>11} | ", process.runnable_count()),
                    style,
                ));
                text.push(Text::styled(
                    format!("{}\n", process.name().unwrap_or("## error ##".to_string())),
                    style,
                ));
            })
    }

    Paragraph::new(text.iter())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Process List")
                .title_style(Style::default()),
        )
        .wrap(false)
        .render(frame, area)
}

pub fn draw_process_details<B>(frame: &mut Frame<B>, area: Rect, app: &App)
where
    B: Backend,
{
    let mut text = Vec::<Text>::new();

    text.push(Text::raw("\n"));

    if let Some(process) = app
        .processes
        .list
        .as_ref()
        .and_then(|list| list.get_process(app.processes.selection.0))
    {
        text.push(Text::styled(
            " Name: ",
            Style::default().modifier(Modifier::BOLD),
        ));
        text.push(Text::raw(format!(
            "{}\n",
            process.name().unwrap_or("## error ##".to_string())
        )));

        text.push(Text::styled(
            " PID: ",
            Style::default().modifier(Modifier::BOLD),
        ));
        text.push(Text::raw(format!("{:}\n", process.pid())));

        text.push(Text::styled(
            " Sender Ports: ",
            Style::default().modifier(Modifier::BOLD),
        ));
        text.push(Text::raw(format!("{:}\n", "not yet implemented")));
        text.push(Text::raw(" • here\n"));
        text.push(Text::raw(" • comes\n"));
        text.push(Text::raw(" • a\n"));
        text.push(Text::raw(" • list\n"));
        text.push(Text::raw(" • of\n"));
        text.push(Text::raw(" • ports\n"));

        text.push(Text::styled(
            " Receiver Ports: ",
            Style::default().modifier(Modifier::BOLD),
        ));
        text.push(Text::raw(format!("{:}\n", "not yet implemented")));
        text.push(Text::raw(" • here\n"));
        text.push(Text::raw(" • comes\n"));
        text.push(Text::raw(" • a\n"));
        text.push(Text::raw(" • list\n"));
        text.push(Text::raw(" • of\n"));
        text.push(Text::raw(" • ports\n"));

        text.push(Text::styled(
            " Runnables: ",
            Style::default().modifier(Modifier::BOLD),
        ));
        text.push(Text::raw(format!("{:}\n", process.runnable_count())));
        if process.runnable_count() >= 1 {
            text.push(Text::styled(
                "     Listing runnables needs implementation!",
                Style::default().fg(Color::Red),
            ));
        }
    }

    Paragraph::new(text.iter())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Process Details")
                .title_style(Style::default()),
        )
        .wrap(true)
        .render(frame, area)
}
