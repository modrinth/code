//! Theseus utility functions
pub mod fetch;
pub mod platform;

macro_rules! wrap_ref_builder {
    ($id:ident < $init:expr => $transform:block) => {{
        let mut it = $init;
        {
            let $id = &mut it;
            $transform;
        }
        it
    }};
}
