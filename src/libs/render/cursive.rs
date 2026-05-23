// NOTE: This renderer owns the terminal for its entire lifetime.  Any concurrent
// writes to stdout/stderr (tracing logs, indicatif progress bars) will corrupt
// the display — accepted limitation of a full-screen TUI renderer.
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

use crate::libs::render::traits::{Renderer, RendererError, RendererUsable};

#[cfg_attr(feature = "tracing", derive(Debug))]
struct TableState {
    headers: &'static [&'static str],
    data: Vec<Vec<String>>,
}

const TABLE_NAME: &str = "table";

type TableStateView = TableView<TableDataRow<String>, usize>;

impl TableState {
    const fn new(headers: &'static [&'static str]) -> Self {
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
            .column_header(self.headers.to_vec())
            .data(self.data.clone())
            .data_orientation(cursive::align::HAlign::Left)
            .sortable(false)
            .build()
    }
}

/// Interactive TUI renderer backed by the `cursive` crate.
///
/// Rows are displayed incrementally as they arrive via [`add_row`].  A `cursive`
/// event loop runs in a background thread; the main thread communicates with it
/// through a [`CbSink`].  `q`, `Esc`, and `Ctrl+C` set a shared quit flag and
/// stop the event loop; the next [`add_row`] call propagates the interruption as
/// an error so the calling command unwinds cleanly.
#[cfg_attr(feature = "tracing", derive(Debug))]
pub struct CursiveRenderer {
    /// The content to be rendered
    state: TableState,
    /// The formatting
    format: Vec<String>,
    /// Shared stop flag; set by the cursive thread when the user requests quit.
    quit: Arc<AtomicBool>,
    /// Callback channel to the cursive background thread
    cb_sink: CbSink,
    /// Handle to the cursive background thread; `take()`-n in `drop` to join it.
    handle: Option<task::JoinHandle<()>>,
}

impl RendererUsable for CursiveRenderer {
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", ret))]
    fn is_usable() -> bool {
        stdout().is_terminal()
    }
}

impl<const N: usize> Renderer<N> for CursiveRenderer {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            level = "trace",
            skip(title, format, headers),
            ret,
            err(level = "info")
        )
    )]
    fn new(
        title: &'static str,
        format: &'static [&'static str; N],
        headers: &'static [&'static str; N],
    ) -> Result<Self, RendererError> {
        if !cfg!(test) && !stdout().is_terminal() {
            bail!(RendererError::CursiveRequireTerminal);
        }

        let quit = Arc::new(AtomicBool::new(false));

        let format = format.iter().map(|s| format!("{{value{s}}}")).collect();

        let state = TableState::new(headers);

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
            // Best-effort: signal cursive to quit so the terminal gets restored.
            let _ = self.cb_sink.send(Box::new(|s: &mut Cursive| s.quit()));
        }
        // Block until the cursive thread finishes (user pressed q/Esc/Ctrl+C).
        self.handle
            .take()
            .map(|handle| task::spawn_blocking(async || handle.await));
    }
}
