Hide Internal Rules in `macro_rules!` Docs
==========================================

When generating docs for `macro_rules!` macros, `rustdoc` will include every
rule, including internal rules that are only supposed to be called from within
your macro. The `clean_docs` attribute will hide your internal rules from
`rustdoc`.

## Example:
```rust
#[macro_export]
macro_rules! messy {
    (@impl $e:expr) => {
        format!("{}", $e)
    };
    ($e:expr) => {
        messy!(@impl $e)
    };
}

#[clean_docs]
#[macro_export]
macro_rules! clean {
    (@impl $e:expr) => {
        format!("{}", $e)
    };
    ($e:expr) => {
        clean!(@impl $e)
    };
}
```

would be documented as
```rust
macro_rules! messy {
    (@impl $e:expr) => { ... };
    ($e:expr) => { ... };
}

macro_rules! clean {
    ($e:expr) => { ... };
}
```

## How does it work?
The macro above is transformed into
```rust
#[macro_export]
macro_rules! clean {
    ($e:expr) => {
        $crate::__clean!(@impl $e)
    };
}

#[macro_export]
macro_rules! __clean {
    (@impl $e:expr) => {
        format!("{}", $e)
    };
}

macro_rules! clean {
    (@impl $e:expr) => {
        format!("{}", $e)
    };
    ($e:expr) => {
        clean!(@impl $e)
    };
}
```

The last, non-`macro_export` macro is there becuase Rust doesn't allow
macro-expanded macro to be invoked by absolute path (i.e. `$crate::__mac`).

The solution is to shadow the `macro_export`ed macro with a local version
that doesn't use absolute paths.

By default this transformation only happens when `rustdoc` is building the
documentation for your macro, so `clean_docs` shouldn't affect your normal
compilation times (see [`always`](#always)).

## Arguments
You can use these optional arguments to configure `clean_macro`.

```rust
#[clean_docs(impl = "#internal", internal = "__internal_mac", always = true)]
```

### `impl`
A string representing the "flag" at the begining of an internal rule. Defaults to `"@"`.

```rust
#[clean_docs(impl = "#internal")]
#[macro_export]
macro_rules! mac {
    (#internal $e:expr) => {
        format!("{}", $e)
    };
    ($e:expr) => {
        mac!(#internal $e)
    };
}
```

### `internal`
A string representing the identifier to use for the internal version of your macro.
By default `clean_docs` prepends `__` (two underscores) to the main macro's identifier.

```rust
#[clean_docs(internal = "__internal_mac")]
#[macro_export]
macro_rules! mac {
    (@impl $e:expr) => {
        format!("{}", $e)
    };
    ($e:expr) => {
        mac!(@impl $e)
    };
}
```

### `always`
A boolean that tells `clean_docs` whether it should transform the macro
even when not building documentation. This is mainly used for testing
purposes. Defaults to `false`.

```rust
#[clean_docs(always = true)]
#[macro_export]
macro_rules! mac {
    (@impl $e:expr) => {
        format!("{}", $e)
    };
    ($e:expr) => {
        mac!(@impl $e)
    };
}
```
