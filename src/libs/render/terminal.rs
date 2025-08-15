use crate::libs::render::Renderer as RendererTrait;
use eyre::{Result, WrapErr as _};
use ratatui::{
    backend::CrosstermBackend,
    layout::Constraint,
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Terminal, TerminalOptions, Viewport,
};
use std::{
    fmt::Display,
    io::{stdout, IsTerminal as _, Stdout},
};

#[cfg_attr(feature = "tracing", derive(Debug))]
pub struct Renderer<'a> {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    table_rows: Vec<Row<'a>>,
    column_widths: Vec<u16>,
    headers: Row<'a>,
    title: &'static str,
}

impl RendererTrait for Renderer<'_> {
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", ret))]
    fn is_usable() -> bool {
        stdout().is_terminal()
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            level = "trace",
            skip(title, _format, headers),
            ret,
            err(level = "info")
        )
    )]
    fn new(title: &'static str, _format: &'static str, headers: &[&'static str]) -> Result<Self> {
        let mut terminal = ratatui::try_init_with_options(TerminalOptions {
            viewport: Viewport::Inline(0),
        })
        .wrap_err("ratatui init failed")?;
        terminal.clear().wrap_err("terminal clear failed")?;
        Ok(Self {
            terminal,
            table_rows: vec![],
            column_widths: vec![],
            title,
            headers: headers
                .iter()
                .map(|h| {
                    Cell::new((*h).to_owned()).style(Style::default().add_modifier(Modifier::BOLD))
                })
                .collect(),
        })
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self, row), err(level = "info"))
    )]
    fn add_row(&mut self, row: &[&dyn Display]) -> Result<()> {
        let str_row = row
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>();

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

        if self.column_widths.is_empty() {
            let column_count = row.len();

            self.column_widths = vec![0u16; column_count];
        }

        #[expect(clippy::indexing_slicing, reason = "it's ok")]
        for (idx, cell) in str_row.iter().enumerate() {
            let width = cell.len();
            let width = u16::try_from(width)
                .wrap_err_with(|| format!("failed converting {width} in a u16"))?;
            if width > self.column_widths[idx] {
                self.column_widths[idx] = width;
            }
        }

        // let area = self.terminal.get_frame().area();
        // self.terminal.set_cursor_position(area)?;
        self.terminal.clear().wrap_err("terminal clear failed")?;
        let table_width = self.table_rows.len();
        let table_width = u16::try_from(table_width)
            .wrap_err_with(|| format!("convert {table_width} into a u16 failed"))?;
        self.terminal = ratatui::try_init_with_options(TerminalOptions {
            viewport: Viewport::Inline(table_width + 3),
        })
        .wrap_err("ratatui init failed")?;

        let rows = self.table_rows.clone();
        let widths = self.column_widths();
        let headers = self.headers.clone();

        self.terminal
            .draw(|frame| {
                let table = Table::new(rows, widths)
                    .header(headers)
                    .block(Block::default().title(self.title).borders(Borders::ALL));
                frame.render_widget(table, frame.area());
            })
            .wrap_err("termianl draw failed")?;

        Ok(())
    }
}

impl Renderer<'_> {
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

impl Drop for Renderer<'_> {
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(self)))]
    fn drop(&mut self) {
        ratatui::restore();
    }
}
