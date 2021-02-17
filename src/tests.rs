mod expected_output {
    use crate::clean_docs_impl;
    use quote::quote;
    use syn::parse::Parser;
    use syn::punctuated::Punctuated;
    use syn::{parse2, NestedMeta, Token};

    macro_rules! make_test {
        (
            [$name:ident]

            input (
                #[clean_docs($($args:tt)*)]
                $($mac:tt)*
            )

            expect (
                $($expected:tt)*
            )
        ) => {
            #[test]
            fn $name() {
                let args = Punctuated::<NestedMeta, Token![,]>::parse_terminated
                    .parse2(quote! {
                        $($args)*
                    })
                    .unwrap()
                    .into_iter()
                    .collect();

                let input = parse2(quote! {
                    $($mac)*
                })
                .unwrap();

                let output = clean_docs_impl(args, input);

                let expected = quote! {
                    $($expected)*
                };

                assert_eq!(output.to_string(), expected.to_string());
            }
        };
    }

    make_test! { [simple]
        input (
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
        )

        expect (
            #[macro_export]
            macro_rules! simple_macro {
                ($e:expr) => {
                    $crate::__simple_macro!(@impl $e)
                };
            }

            #[doc(hidden)]
            #[macro_export]
            macro_rules! __simple_macro {
                (@impl $e:expr) => {
                    format!("{}", $e)
                };
            }

            #[allow(unused_macros)]
            macro_rules! simple_macro {
                (@impl $e:expr) => {
                    format!("{}", $e)
                };
                ($e:expr) => {
                    simple_macro!(@impl $e)
                };
            }
        )
    }

    make_test! { [custom_impl]
        input (
            #[clean_docs(impl = "^internal", always = true)]
            #[macro_export]
            macro_rules! custom_impl_macro {
                (^internal $e:expr) => {
                    format!("{}", $e)
                };
                ($e:expr) => {
                    custom_impl_macro!(^internal $e)
                };
            }
        )

        expect (
            #[macro_export]
            macro_rules! custom_impl_macro {
                ($e:expr) => {
                    $crate::__custom_impl_macro!(^internal $e)
                };
            }

            #[doc(hidden)]
            #[macro_export]
            macro_rules! __custom_impl_macro {
                (^internal $e:expr) => {
                    format!("{}", $e)
                };
            }

            #[allow(unused_macros)]
            macro_rules! custom_impl_macro {
                (^internal $e:expr) => {
                    format!("{}", $e)
                };
                ($e:expr) => {
                    custom_impl_macro!(^internal $e)
                };
            }
        )
    }

    make_test! { [multiple_internal]
        input (
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
        )

        expect (
            #[macro_export]
            macro_rules! multiple_internal_macro {
                ($e:expr) => {
                    $crate::__multiple_internal_macro!(@impl[0] $e)
                };
            }

            #[doc(hidden)]
            #[macro_export]
            macro_rules! __multiple_internal_macro {
                (@impl[0] $e:expr) => {
                    $crate::__multiple_internal_macro!(@impl[1] ->[$e]<-)
                };
                (@impl[1] $($t:tt)+) => {
                    $crate::__multiple_internal_macro!(@impl[2] stringify!($($t)+))
                };
                (@impl[2] $e:expr) => {
                    format!("{}", $e)
                };
            }

            #[allow(unused_macros)]
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
        )
    }

    make_test! { [multiple_public]
        input (
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
        )

        expect (
            #[macro_export]
            macro_rules! multiple_public_macro {
                (() $e:expr) => {
                    $crate::__multiple_public_macro!(@impl[parens] $e)
                };
                ({} $e:expr) => {
                    $crate::__multiple_public_macro!(@impl[braces] $e)
                };
                ([] $e:expr) => {
                    $crate::__multiple_public_macro!(@impl[brackets] $e)
                };
            }

            #[doc(hidden)]
            #[macro_export]
            macro_rules! __multiple_public_macro {
                (@impl[parens] $e:expr) => {
                    format!("({})", $e)
                };
                (@impl[braces] $e:expr) => {
                    format!("{{{}}}", $e)
                };
                (@impl[brackets] $e:expr) => {
                    format!("[{}]", $e)
                };
            }

            #[allow(unused_macros)]
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
        )
    }

    make_test! { [no_internal]
        input (
            #[clean_docs(always = true)]
            #[macro_export]
            macro_rules! no_internal_macro {
                ($e:expr) => {
                    format!("{}", $e)
                };
            }
        )

        expect (
            #[macro_export]
            macro_rules! no_internal_macro {
                ($e:expr) => {
                    format!("{}", $e)
                };
            }
        )
    }
}
