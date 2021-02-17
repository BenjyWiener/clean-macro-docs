use clean_macro_docs::clean_docs;

#[clean_docs(always = true)]
#[macro_export]
macro_rules! simple_macro {
    (@impl $e:expr) => {
        format!("{}", $e)
    };
    ($e:expr) => {
        simple_macro!(@impl $e)
    };
}

#[clean_docs(impl = "#internal", always = true)]
#[macro_export]
macro_rules! custom_impl_macro {
    (#internal $e:expr) => {
        format!("{}", $e)
    };
    ($e:expr) => {
        custom_impl_macro!(#internal $e)
    };
}

#[clean_docs(always = true)]
#[macro_export]
macro_rules! multiple_internal_macro {
    (@impl[0] $e:expr) => {
        multiple_internal_macro!(@impl[1] ->[$e]<-)
    };
    (@impl[1] $($t:tt)+) => {
        multiple_internal_macro!(@impl[2] stringify!($($t)+))
    };
    (@impl[2] $e:expr) => {
        format!("{}", $e)
    };
    ($e:expr) => {
        multiple_internal_macro!(@impl[0] $e)
    };
}

#[clean_docs(always = true)]
#[macro_export]
macro_rules! multiple_public_macro {
    (@impl[parens] $e:expr) => {
        format!("({})", $e)
    };
    (@impl[braces] $e:expr) => {
        format!("{{{}}}", $e)
    };
    (@impl[brackets] $e:expr) => {
        format!("[{}]", $e)
    };
    (() $e:expr) => {
        multiple_public_macro!(@impl[parens] $e)
    };
    ({} $e:expr) => {
        multiple_public_macro!(@impl[braces] $e)
    };
    ([] $e:expr) => {
        multiple_public_macro!(@impl[brackets] $e)
    };
}

#[clean_docs(always = true)]
#[macro_export]
macro_rules! no_internal_macro {
    ($e:expr) => {
        format!("{}", $e)
    };
}

#[clean_docs(always = true)]
#[macro_export]
macro_rules! back_and_forth_macro {
    (@impl $t:tt) => {
        (back_and_forth_macro!($t), back_and_forth_macro!($t))
    };
    ([$t:tt]) => {
        back_and_forth_macro!(@impl $t)
    };
    ($t:tt) => {
        $t
    };
}

#[test]
fn simple() {
    assert_eq!(simple_macro!(54321), "54321");
}

#[test]
fn custom_impl() {
    assert_eq!(custom_impl_macro!(54321), "54321");
}

#[test]
fn multiple_internal() {
    assert_eq!(multiple_internal_macro!(54321), "-> [54321] <-");
}

#[test]
fn multiple_public() {
    assert_eq!(multiple_public_macro!(() 54321), "(54321)");
    assert_eq!(multiple_public_macro!({} 54321), "{54321}");
    assert_eq!(multiple_public_macro!([] 54321), "[54321]");
}

#[test]
fn no_internal() {
    assert_eq!(no_internal_macro!(54321), "54321");
}

#[test]
fn back_and_forth() {
    assert_eq!(back_and_forth_macro!([[3]]), ((3, 3), (3, 3)));
}
