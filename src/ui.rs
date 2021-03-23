// Copyright 2020 Mathias Kraus - All rights reserved
//
// Licensed under the Apache License, Version 2.0 <LICENSE or
// http://www.apache.org/licenses/LICENSE-2.0>. This file may not be
// copied, modified, or distributed except according to those terms.

mod memory;
mod process;
mod service;

use crate::App;

use tui::backend::Backend;
use tui::layout::{Constraint, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Paragraph, Tabs, Wrap};
use tui::{Frame, Terminal};

use std::io;

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), io::Error> {
    terminal.draw(|mut frame| {
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(3)].as_ref())
            .split(frame.size());

        draw_tabbar(&mut frame, chunks[0], app);
        draw_main_view(&mut frame, chunks[1], app);
    })
}

fn draw_tabbar<B>(frame: &mut Frame<B>, area: Rect, app: &App)
where
    B: Backend,
{
    let tabs = Tabs::new(app.tabs.titles.iter().cloned().map(Spans::from).collect())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Introspection Topics"),
        )
        .select(app.tabs.index)
        .style(Style::default())
        .highlight_style(Style::default().fg(Color::Yellow));

    frame.render_widget(tabs, area);
}

fn draw_main_view<B>(frame: &mut Frame<B>, area: Rect, app: &App)
where
    B: Backend,
{
    match app.tabs.index {
        0 => {
            let mut text = Vec::<Spans>::new();

            text.push(Spans::from(vec![Span::raw("unimplemented!")]));
            text.push(Spans::from(vec![Span::raw("")]));
            text.push(Spans::from(vec![Span::raw(
                "use arrow keys to navigate to the next page!",
            )]));

            let paragraph = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title("Overview"))
                .wrap(Wrap { trim: false });

            frame.render_widget(paragraph, area);
        }
        1 => memory::draw(frame, area, app),
        2 => process::draw(frame, area, app),
        3 => service::draw(frame, area, app),
        _ => {}
    }
}
