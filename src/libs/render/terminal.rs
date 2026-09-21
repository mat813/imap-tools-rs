use std::{
    fmt::Display,
    io::{IsTerminal as _, Stdout, stdout},
};

use exn::{Result, ResultExt as _};
use ratatui::{
    Terminal, TerminalOptions, Viewport,
    backend::CrosstermBackend,
    layout::Constraint,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::libs::render::traits::{Renderer, RendererError, RendererUsable, TableSpec};

#[cfg_attr(feature = "tracing", derive(Debug))]
pub struct TerminalRenderer<'a> {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    table_rows: Vec<Row<'a>>,
    column_widths: Vec<u16>,
    headers: Row<'a>,
    title: String,
}

impl RendererUsable for TerminalRenderer<'_> {
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", ret))]
    fn is_usable() -> bool {
        stdout().is_terminal()
    }
}

impl<const N: usize> Renderer<N> for TerminalRenderer<'_> {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(spec), ret, err(level = "info"))
    )]
    fn new(spec: TableSpec<N>) -> Result<Self, RendererError> {
        let mut terminal = ratatui::try_init_with_options(TerminalOptions {
            viewport: Viewport::Inline(0),
        })
        .or_raise(|| RendererError::TerminalInit)?;
        terminal.clear().or_raise(|| RendererError::TerminalClear)?;
        Ok(Self {
            terminal,
            table_rows: vec![],
            // A column is at least as wide as its header
            column_widths: spec
                .labels
                .iter()
                .map(|label| u16::try_from(Line::from(label.as_str()).width()).unwrap_or(u16::MAX))
                .collect(),
            title: spec.title,
            headers: spec
                .labels
                .into_iter()
                .map(|label| Cell::new(label).style(Style::default().add_modifier(Modifier::BOLD)))
                .collect(),
        })
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self, row), err(level = "info"))
    )]
    fn add_row(&mut self, row: &[&dyn Display; N]) -> Result<(), RendererError> {
        let str_row: Vec<_> = row.iter().map(std::string::ToString::to_string).collect();

        #[cfg(feature = "tracing")]
        tracing::trace!(row = ?str_row);

        let new_row = Row::new(str_row.iter().cloned().enumerate().map(|(idx, content)| {
            let mut style = Style::default();
            if idx == 0 {
                style = style.add_modifier(Modifier::BOLD);
            }
            Cell::new(content).style(style)
        }));
        self.table_rows.push(new_row);

        #[expect(clippy::indexing_slicing, reason = "it's ok")]
        for (idx, cell) in str_row.iter().enumerate() {
            let width = cell.len();
            let width = u16::try_from(width).or_raise(|| RendererError::TerminalU16 { width })?;
            if width > self.column_widths[idx] {
                self.column_widths[idx] = width;
            }
        }

        let table_width = self.table_rows.len();
        let table_width = u16::try_from(table_width)
            .or_raise(|| RendererError::TerminalU16 { width: table_width })?;
        self.terminal = ratatui::try_init_with_options(TerminalOptions {
            viewport: Viewport::Inline(table_width + 3),
        })
        .or_raise(|| RendererError::TerminalInit)?;

        let rows = self.table_rows.clone();
        let widths = self.column_widths();
        let headers = self.headers.clone();

        self.terminal
            .draw(|frame| {
                let table = Table::new(rows, widths).header(headers).block(
                    Block::default()
                        .title(self.title.as_str())
                        .borders(Borders::ALL),
                );
                frame.render_widget(table, frame.area());
            })
            .or_raise(|| RendererError::TerminalDraw)?;

        Ok(())
    }
}

impl TerminalRenderer<'_> {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), ret)
    )]
    fn column_widths(&self) -> Vec<Constraint> {
        let last_idx = self.column_widths.len() - 1;
        self.column_widths
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

impl Drop for TerminalRenderer<'_> {
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(self)))]
    fn drop(&mut self) {
        ratatui::restore();
    }
}
