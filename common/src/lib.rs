pub mod illumos;

pub use anyhow::{anyhow, bail, Result};
pub use log::{debug, error, info, trace, warn};
use slog::{Drain, Logger};
use slog_async::Async;
use slog_scope::{set_global_logger, GlobalLoggerGuard};
use slog_syslog::Facility;
use slog_term::{CompactFormat, TermDecorator};
pub use thiserror::Error;

pub static AUTHORIZATION_HEADER: &str = "authorization";

/**
 * Initialise a logger which writes to stdout, and which does the right thing on
 * both an interactive terminal and when stdout is not a tty.
 */
pub fn init_slog_logging(use_syslog: bool) -> Result<GlobalLoggerGuard> {
    if use_syslog {
        let drain = slog_syslog::unix_3164(Facility::LOG_DAEMON)?.fuse();
        let logger = Logger::root(drain, slog::slog_o!());

        let scope_guard = set_global_logger(logger);
        let _log_guard = slog_stdlog::init()?;

        Ok(scope_guard)
    } else {
        let decorator = TermDecorator::new().stdout().build();
        let drain = CompactFormat::new(decorator).build().fuse();
        let drain = Async::new(drain).build().fuse();
        let logger = Logger::root(drain, slog::slog_o!());

        let scope_guard = set_global_logger(logger);
        let _log_guard = slog_stdlog::init()?;

        Ok(scope_guard)
    }
}

pub fn path_split(full_path: &str) -> Option<(&str, &str)> {
    full_path.rsplit_once('/')
}
