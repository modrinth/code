use std::collections::HashMap;
use std::fmt::Write as _;
use std::hash::{DefaultHasher, Hasher};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use tracing_error::ExtractSpanTrace;

/// Minimum interval between two logs of the same error message.
///
/// A failing Tauri command is usually retried by the frontend, so a single
/// persistent fault (a saturated database pool, a missing file) can otherwise
/// produce thousands of identical lines per second.
const REPEAT_INTERVAL: Duration = Duration::from_secs(5);

/// Maximum number of distinct error messages tracked at once. Bounded so the
/// suppression table cannot itself grow without limit.
const MAX_TRACKED_ERRORS: usize = 256;

struct SuppressionState {
    last_logged: Instant,
    suppressed: u64,
}

static RECENT_ERRORS: LazyLock<Mutex<HashMap<u64, SuppressionState>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Hashes a `Display` implementation without allocating an intermediate
/// `String`, so the common (suppressed) path stays cheap under a storm.
struct HashWriter(DefaultHasher);

impl std::fmt::Write for HashWriter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.write(s.as_bytes());
        Ok(())
    }
}

fn error_key(err: &theseus::Error) -> u64 {
    let mut writer = HashWriter(DefaultHasher::new());
    // Writing to a hasher cannot fail.
    let _ = write!(writer, "{err}");
    writer.0.finish()
}

/// Decides whether an occurrence should be logged now. Returns the number of
/// occurrences suppressed since this message was last logged, or `None` if
/// this occurrence should itself be suppressed.
///
/// Split from [`should_log`] so it can be tested without the global table.
fn should_log_in(
    recent: &mut HashMap<u64, SuppressionState>,
    key: u64,
    now: Instant,
) -> Option<u64> {
    match recent.get_mut(&key) {
        Some(state) => {
            if now.duration_since(state.last_logged) < REPEAT_INTERVAL {
                state.suppressed += 1;
                return None;
            }

            let suppressed = std::mem::take(&mut state.suppressed);
            state.last_logged = now;
            Some(suppressed)
        }
        None => {
            // Evict the least recently logged entry to stay within bounds.
            if recent.len() >= MAX_TRACKED_ERRORS
                && let Some(oldest) = recent
                    .iter()
                    .min_by_key(|(_, state)| state.last_logged)
                    .map(|(key, _)| *key)
            {
                recent.remove(&oldest);
            }

            recent.insert(
                key,
                SuppressionState {
                    last_logged: now,
                    suppressed: 0,
                },
            );
            Some(0)
        }
    }
}

fn should_log(key: u64, now: Instant) -> Option<u64> {
    let mut recent = RECENT_ERRORS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    should_log_in(&mut recent, key, now)
}

pub fn display_tracing_error(err: &theseus::Error) {
    let Some(suppressed) = should_log(error_key(err), Instant::now()) else {
        return;
    };

    // When nothing was dropped the output is byte-for-byte what it was
    // before, so existing log greps keep working.
    match (get_span_trace(err), suppressed) {
        (Some(span_trace), 0) => {
            tracing::error!(error = %err, span_trace = %span_trace);
        }
        (Some(span_trace), suppressed) => {
            tracing::error!(
                error = %err,
                span_trace = %span_trace,
                suppressed,
                "identical errors were suppressed since the last log"
            );
        }
        (None, 0) => {
            tracing::error!(error = %err);
        }
        (None, suppressed) => {
            tracing::error!(
                error = %err,
                suppressed,
                "identical errors were suppressed since the last log"
            );
        }
    }
}

pub fn get_span_trace<'a>(
    error: &'a (dyn std::error::Error + 'static),
) -> Option<&'a tracing_error::SpanTrace> {
    error.source().and_then(|e| e.span_trace())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_occurrence_always_logs() {
        let mut recent = HashMap::new();
        assert_eq!(should_log_in(&mut recent, 1, Instant::now()), Some(0));
    }

    #[test]
    fn repeats_within_the_interval_are_suppressed_and_counted() {
        let mut recent = HashMap::new();
        let start = Instant::now();

        assert_eq!(should_log_in(&mut recent, 1, start), Some(0));
        for _ in 0..1000 {
            assert_eq!(should_log_in(&mut recent, 1, start), None);
        }

        // Once the interval has passed, the backlog is reported exactly once.
        let later = start + REPEAT_INTERVAL;
        assert_eq!(should_log_in(&mut recent, 1, later), Some(1000));
        assert_eq!(should_log_in(&mut recent, 1, later), None);
    }

    #[test]
    fn distinct_errors_do_not_suppress_each_other() {
        let mut recent = HashMap::new();
        let now = Instant::now();

        assert_eq!(should_log_in(&mut recent, 1, now), Some(0));
        assert_eq!(should_log_in(&mut recent, 2, now), Some(0));
        assert_eq!(should_log_in(&mut recent, 1, now), None);
    }

    #[test]
    fn tracking_table_stays_bounded() {
        let mut recent = HashMap::new();
        let start = Instant::now();

        for key in 0..(MAX_TRACKED_ERRORS as u64 * 4) {
            // Stagger the timestamps so eviction has a well-defined victim.
            let _ = should_log_in(
                &mut recent,
                key,
                start + Duration::from_millis(key),
            );
            assert!(recent.len() <= MAX_TRACKED_ERRORS);
        }

        assert_eq!(recent.len(), MAX_TRACKED_ERRORS);
    }
}
