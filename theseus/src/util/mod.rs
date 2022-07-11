//! Theseus utility functions
pub mod fetch;
pub mod platform;

/// Wrap a builder which uses a mut reference into one which outputs an owned value
macro_rules! wrap_ref_builder {
    ($id:ident = $init:expr => $transform:block) => {{
        let mut it = $init;
        {
            let $id = &mut it;
            $transform;
        }
        it
    }};
}

/// Alias a trait, used to avoid needing nightly features
macro_rules! alias_trait {
    ($scope:vis $name:ident : $bound:path $(, $bounds:path)*) => {
        $scope trait $name: $bound $(+ $bounds)* {}
        impl<T: $bound $(+ $bounds)*> $name for T {}
    }
}
