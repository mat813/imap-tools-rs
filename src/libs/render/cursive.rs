// NOTE: This renderer owns the terminal for its whole lifetime. Concurrent
// writes to stdout or stderr (tracing logs, indicatif progress bars) corrupt
// the display. We accept this limitation of a full-screen TUI renderer.
use std::{
    fmt::Display,
    io::{IsTerminal as _, stdout},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use better_cursive_table::{TableBuilder, TableDataRow, TableView};
use cursive::{
    CbSink, Cursive, CursiveExt as _,
    event::{Event, Key},
    reexports::crossbeam_channel::unbounded,
    view::Nameable as _,
    views::{Dialog, LinearLayout},
};
use exn::{Result, ResultExt as _, bail};
use strfmt::strfmt;
use tokio::task;

use crate::libs::render::traits::{Renderer, RendererError, TableSpec};

#[cfg_attr(feature = "tracing", derive(Debug))]
struct TableState {
    headers: Vec<String>,
    data: Vec<Vec<String>>,
}

const TABLE_NAME: &str = "table";

type TableStateView = TableView<TableDataRow<String>, usize>;

impl TableState {
    const fn new(headers: Vec<String>) -> Self {
        Self {
            headers,
            data: Vec::new(),
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self, row))
    )]
    fn add_row(&mut self, row: Vec<String>) {
        self.data.push(row);
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(self)))]
    fn build(&self) -> TableStateView {
        // TODO: align per column
        TableBuilder::new()
            .column_header(self.headers.clone())
            .data(self.data.clone())
            .data_orientation(cursive::align::HAlign::Left)
            .sortable(false)
            .build()
    }
}

/// Interactive TUI renderer that uses the `cursive` crate.
///
/// The renderer shows each row when it arrives through [`add_row`]. A `cursive`
/// event loop runs in a background thread. The main thread sends commands to it
/// through a [`CbSink`]. `q`, `Esc` and `Ctrl+C` set a shared quit flag and
/// stop the event loop. The next [`add_row`] call returns the interruption as
/// an error, so the calling command stops cleanly.
#[cfg_attr(feature = "tracing", derive(Debug))]
pub struct CursiveRenderer {
    /// The content to render
    state: TableState,
    /// The formatting
    format: Vec<String>,
    /// Shared stop flag. The cursive thread sets it when the user asks to quit.
    quit: Arc<AtomicBool>,
    /// Callback channel to the cursive background thread
    cb_sink: CbSink,
    /// Handle to the cursive background thread. `drop` uses `take()` on it to join the thread.
    handle: Option<task::JoinHandle<()>>,
}

impl<const N: usize> Renderer<N> for CursiveRenderer {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(spec), ret, err(level = "info"))
    )]
    fn new(spec: TableSpec<N>) -> Result<Self, RendererError> {
        if !cfg!(test) && !stdout().is_terminal() {
            bail!(RendererError::CursiveRequireTerminal);
        }

        let quit = Arc::new(AtomicBool::new(false));

        let format = spec
            .format
            .iter()
            .map(|s| format!("{{value{s}}}"))
            .collect();

        let state = TableState::new(spec.labels.into());
        let title = spec.title;

        let table = state.build().with_name(TABLE_NAME);

        let (tx, rx) = unbounded();
        let quit_thread = quit.clone();
        let handle = task::spawn(async move {
            let mut cursive = Cursive::new();

            let layout = LinearLayout::vertical().child(table);
            cursive.add_layer(Dialog::around(layout).title(title));

            let q = quit_thread.clone();
            cursive.add_global_callback('q', move |s| {
                q.store(true, Ordering::Relaxed);
                s.quit();
            });
            let q = quit_thread.clone();
            cursive.add_global_callback(Key::Esc, move |s| {
                q.store(true, Ordering::Relaxed);
                s.quit();
            });
            let q = quit_thread.clone();
            cursive.add_global_callback(Event::CtrlChar('c'), move |s| {
                q.store(true, Ordering::Relaxed);
                s.quit();
            });

            cursive.set_fps(10);

            if tx.send(cursive.cb_sink().clone()).is_ok() {
                cursive.run();
            }
            quit_thread.store(true, Ordering::Relaxed);
        });

        let cb_sink = rx.recv().or_raise(|| RendererError::CursiveBackendInit)?;

        Ok(Self {
            state,
            format,
            quit,
            cb_sink,
            handle: Some(handle),
        })
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self, row), err(level = "info"))
    )]
    fn add_row(&mut self, row: &[&dyn Display; N]) -> Result<(), RendererError> {
        if self.quit.load(Ordering::Relaxed) {
            bail!(RendererError::CursiveInterrupted);
        }

        let row = row
            .iter()
            .zip(&self.format)
            .map(|(row, format)| {
                strfmt!(format, value => row.to_string()).or_raise(|| RendererError::Strfmt {
                    format: format.clone(),
                    display: Box::new(row.to_string()),
                })
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;

        #[cfg(feature = "tracing")]
        tracing::trace!(row = ?row);

        self.state.add_row(row);
        let table = self.state.build();

        let _ = self.cb_sink.send(Box::new(move |siv: &mut Cursive| {
            siv.call_on_name(TABLE_NAME, |view: &mut TableStateView| {
                *view = table;
            });
        }));

        Ok(())
    }
}

impl Drop for CursiveRenderer {
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(self)))]
    fn drop(&mut self) {
        if std::thread::panicking() {
            // Best effort: tell cursive to quit, so that it restores the terminal.
            let _ = self.cb_sink.send(Box::new(|s: &mut Cursive| s.quit()));
        }
        // Wait until the cursive thread ends (the user pressed q, Esc or Ctrl+C).
        self.handle
            .take()
            .map(|handle| task::spawn_blocking(async || handle.await));
    }
}
