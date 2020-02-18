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
use tui::widgets::{Block, Borders, Paragraph, Tabs, Text, Widget};
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
    Tabs::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Introspection Topics"),
        )
        .titles(&app.tabs.titles)
        .select(app.tabs.index)
        .style(Style::default())
        .highlight_style(Style::default().fg(Color::Yellow))
        .render(frame, area);
}

fn draw_main_view<B>(frame: &mut Frame<B>, area: Rect, app: &App)
where
    B: Backend,
{
    match app.tabs.index {
        0 => {
            let text = [Text::raw(
                "unimplemented!\n\nuse arraw keys to navigate to the next page".to_string(),
            )];
            Paragraph::new(text.iter())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Overview")
                        .title_style(Style::default()),
                )
                .wrap(false)
                .render(frame, area)
        }
        1 => memory::draw(frame, area, app),
        2 => process::draw(frame, area, app),
        3 => service::draw(frame, area, app),
        _ => {}
    }
}
