mod expected_output {
    use crate::clean_docs_impl;
    use quote::quote;
    use syn::parse::Parser;
    use syn::punctuated::Punctuated;
    use syn::{parse2, NestedMeta, Token};

    macro_rules! make_test {
        (
            [$name:ident]

            input {
                #[clean_docs($($args:tt)*)]
                $($mac:tt)*
            }

            expect {
                $($expected:tt)*
            }
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
        input {
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
        }

        expect {
            #[macro_export]
            macro_rules! simple_macro {
                ($e:expr) => {
                    $crate::__simple_macro!(@impl $e)
                }
            }

            #[doc(hidden)]
            #[macro_export]
            macro_rules! __simple_macro {
                (@impl $e:expr) => {
                    format!("{}", $e)
                }
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
        }
    }
}
