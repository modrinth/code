pub mod status;

use lazy_static::lazy_static;
use std::sync::atomic::AtomicBool;

lazy_static! {
    pub static ref SEARCH_READY: AtomicBool = AtomicBool::new(false);
}
