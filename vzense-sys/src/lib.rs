/*!
# vzense-sys

Generates and uses Vzense C library bindings as a Rust crate. This crate is used as a base layer in `vzense-rust`.
*/

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod dcam560 {
    include!("../bindings/dcam560.rs");
}

pub mod scepter {
    include!("../bindings/scepter.rs");
}
