// SPDX-License-Identifier: Apache-2.0

use crate::app::App;

use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

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
    let mut text = Vec::<Line>::new();

    text.push(Line::from(vec![Span::raw("")]));

    text.push(Line::from(vec![Span::styled(
        "    PID | Publisher | Subscriber | Nodes | Process Name",
        Style::default().add_modifier(Modifier::BOLD),
    )]));

    text.push(Line::from(vec![Span::raw(
        " ----------------------------------------------------------",
    )]));

    for (index, (process_name, details)) in app.processes.map.iter().enumerate() {
        let style = if app.processes.selection.0 == index {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        text.push(Line::from(vec![
            Span::styled(format!(" {:>6} | ", details.pid), style),
            Span::styled(format!("{:>9} | ", details.publisher_ports.len()), style),
            Span::styled(format!("{:>10} | ", details.subscriber_ports.len()), style),
            Span::styled(format!("{:>5} | ", details.nodes.len()), style),
            Span::styled(format!("{}", process_name), style),
        ]));
    }

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Process List"))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

pub fn draw_process_details<B>(frame: &mut Frame<B>, area: Rect, app: &App)
where
    B: Backend,
{
    let mut text = Vec::<Line>::new();

    text.push(Line::from(vec![Span::raw("")]));

    let process_name = &app.processes.selection.1;
    if let Some(details) = app.processes.map.get(process_name) {
        text.push(Line::from(vec![
            Span::styled(" Name: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", process_name)),
        ]));

        text.push(Line::from(vec![
            Span::styled(" PID: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{:}", details.pid)),
        ]));

        text.push(Line::from(vec![
            Span::styled(
                " Publisher Ports: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{:}", details.publisher_ports.len())),
        ]));
        for port in details.publisher_ports.iter() {
            text.push(Line::from(vec![Span::raw(format!(
                " • {} • {} • {}",
                port.service_id, port.instance_id, port.event_id
            ))]));
        }

        text.push(Line::from(vec![
            Span::styled(
                " Subscriber Ports: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{:}", details.subscriber_ports.len())),
        ]));
        for port in details.subscriber_ports.iter() {
            text.push(Line::from(vec![Span::raw(format!(
                " • {} • {} • {}",
                port.service_id, port.instance_id, port.event_id
            ))]));
        }

        text.push(Line::from(vec![
            Span::styled(" Nodes: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{:}", details.nodes.len())),
        ]));
        if details.nodes.len() >= 1 {
            text.push(Line::from(vec![Span::styled(
                "     Listing nodes needs implementation!",
                Style::default().fg(Color::Red),
            )]));
        }
    }

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Process Details"),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}
