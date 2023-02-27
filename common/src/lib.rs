pub mod illumos;

use log::SetLoggerError;
pub use log::{debug, error, info, trace, warn};
use miette::Diagnostic;
use slog::{Drain, Logger};
use slog_async::Async;
use slog_scope::{set_global_logger, GlobalLoggerGuard};
use slog_syslog::Facility;
use slog_term::{CompactFormat, TermDecorator};
use std::io::{stdout, Write};
pub use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum LogError {
    #[error(transparent)]
    SlogError(#[from] std::io::Error),
    #[error(transparent)]
    SetLoggerError(#[from] SetLoggerError),
}

type Result<T> = miette::Result<T, LogError>;

pub static AUTHORIZATION_HEADER: &str = "authorization";

fn ignore_error<E>(_err: E) -> std::result::Result<(), slog::Never> {
    Ok(())
}

pub struct SimpleStdoutDrain;

impl slog::Drain for SimpleStdoutDrain {
    type Ok = ();

    type Err = slog::Never;

    fn log(
        &self,
        record: &slog::Record,
        _values: &slog::OwnedKVList,
    ) -> std::result::Result<Self::Ok, Self::Err> {
        #[allow(unused_must_use)]
        stdout()
            .write_all(format!("{}\n", record.msg()).as_bytes())
            .map_err(ignore_error)
            .unwrap();
        #[allow(unused_must_use)]
        stdout().flush().unwrap();
        Ok(())
    }
}

impl SimpleStdoutDrain {
    fn new() -> Self {
        SimpleStdoutDrain {}
    }
}

/**
 * Initialise a logger which writes to stdout, and which does the right thing on
 * both an interactive terminal and when stdout is not a tty.
 */
pub fn init_slog_logging(use_syslog: bool, no_decoration: bool) -> Result<GlobalLoggerGuard> {
    if use_syslog {
        let drain = slog_syslog::unix_3164(Facility::LOG_DAEMON)?.fuse();
        let logger = Logger::root(drain, slog::slog_o!());

        let scope_guard = set_global_logger(logger);
        let _log_guard = slog_stdlog::init()?;

        Ok(scope_guard)
    } else {
        let drain = if no_decoration {
            Async::new(SimpleStdoutDrain::new()).build().fuse()
        } else {
            let decorator = TermDecorator::new().stdout().build();
            let drain = CompactFormat::new(decorator).build().fuse();
            Async::new(drain).build().fuse()
        };

        let logger = Logger::root(drain, slog::slog_o!());

        let scope_guard = set_global_logger(logger);
        let _log_guard = slog_stdlog::init()?;

        Ok(scope_guard)
    }
}

pub fn path_split(full_path: &str) -> Option<(&str, &str)> {
    full_path.rsplit_once('/')
}
