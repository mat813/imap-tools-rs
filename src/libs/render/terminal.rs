use crate::libs::{error::OurResult, render::Renderer as RendererTrait};
use ratatui::{
    backend::CrosstermBackend,
    layout::Constraint,
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Terminal, TerminalOptions, Viewport,
};
use std::{fmt::Display, io};

pub struct Renderer<'a> {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    table_rows: Vec<Row<'a>>,
    column_widths: Option<Vec<u16>>,
    headers: Row<'a>,
    title: &'static str,
}

impl<'a> RendererTrait for Renderer<'a> {
    fn new(
        title: &'static str,
        _format: &'static str,
        headers: &[&'static str],
    ) -> OurResult<Self> {
        let mut terminal = ratatui::try_init_with_options(TerminalOptions {
            viewport: Viewport::Inline(0),
        })?;
        terminal.clear()?;
        Ok(Self {
            terminal,
            table_rows: vec![],
            column_widths: None,
            title,
            headers: headers
                .iter()
                .map(|h| {
                    Cell::new((*h).to_string()).style(Style::default().add_modifier(Modifier::BOLD))
                })
                .collect(),
        })
    }

    fn add_row(&mut self, row: &[&dyn Display]) -> OurResult<()> {
        let str_row = row
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>();
        let new_row = Row::new(str_row.iter().cloned().enumerate().map(|(idx, content)| {
            let mut style = Style::default();
            if idx == 0 {
                style = style.add_modifier(Modifier::BOLD);
            }
            Cell::new(content).style(style)
        }));
        self.table_rows.push(new_row);

        if self.column_widths.is_none() {
            let column_count = row.len();

            self.column_widths = Some(vec![0u16; column_count]);
        }

        for (idx, cell) in str_row.iter().enumerate() {
            let width = cell.len();
            if u16::try_from(width)? > self.column_widths.as_ref().unwrap()[idx] {
                self.column_widths.as_mut().unwrap()[idx] = u16::try_from(width)?;
            }
        }

        // let area = self.terminal.get_frame().area();
        // self.terminal.set_cursor_position(area)?;
        self.terminal.clear()?;
        self.terminal = ratatui::try_init_with_options(TerminalOptions {
            viewport: Viewport::Inline(u16::try_from(self.table_rows.len())? + 3),
        })?;

        let rows = self.table_rows.clone();
        let widths = self.column_widths();
        let headers = self.headers.clone();

        self.terminal.draw(|frame| {
            let table = Table::new(rows, widths)
                .header(headers)
                .block(Block::default().title(self.title).borders(Borders::ALL));
            frame.render_widget(table, frame.area());
        })?;

        Ok(())
    }
}

impl<'a> Renderer<'a> {
    fn column_widths(&self) -> Vec<Constraint> {
        let widths = self.column_widths.as_ref().unwrap();
        let last_idx = widths.len() - 1;
        widths
            .iter()
            .enumerate()
            .map(|(idx, width)| {
                if idx == last_idx {
                    Constraint::Min(0)
                } else {
                    Constraint::Length(width + 1)
                }
            })
            .collect()
    }
}

impl<'a> Drop for Renderer<'a> {
    fn drop(&mut self) {
        ratatui::restore();
    }
}
