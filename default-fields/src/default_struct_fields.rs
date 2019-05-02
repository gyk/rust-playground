#[allow(unused_macros)]
macro_rules! default_fields {
    (
        $(#[$attr:meta])*
        $visibility:vis struct $name:ident {
            $(pub $fname:ident : $ftype:ty $(= $default:expr)*),* $(,)?
        }
    ) => {
        #[allow(non_snake_case)]
        $(#[$attr])*
        $visibility struct $name {
            $(pub $fname : $ftype,)*
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    $($fname: default_fields! {
                        @maybe_default $($default)*
                    },)*
                }
            }
        }
    };

    (@maybe_default $default:expr) => {
        $default
    };

    (@maybe_default) => {
        Default::default()
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn default_struct_fields() {
        default_fields! {
            #[derive(Debug)]
            pub struct ConnectionInfo {
                pub app: Option<String> = None,
                pub flashVer: Option<String> = Some("LNX 11,1,102,55".to_owned()),
                pub swfUrl: Option<String> = None,
                pub tcUrl: Option<String> = None,
                pub fpad: Option<bool> = Some(false),
                pub audioCodecs: Option<f64> = Some(3575.0_f64),
                pub videoCodecs: Option<f64> = Some(252.0_f64),
                pub videoFunction: Option<f64> = Some(1.0_f64),
                pub pageUrl: Option<String> = None,
                pub objectEncoding: Option<f64> = Some(0.0_f64),
            }
        }

        let conn_info = ConnectionInfo::default();
        println!("===== default_struct_fields =====");
        println!("{:#?}", conn_info);
        println!();

        // Without `pub` and trailing comma / no default values
        default_fields! {
            #[derive(Debug)]
            struct Foo {
                pub bar: String = "whatever".to_owned(),
                pub baz: usize,
                pub qux: Option<String>,
            }
        }

        let foo = Foo::default();
        assert_eq!(foo.baz, 0);
        assert_eq!(foo.qux, None);
        println!("{:#?}", foo);
    }
}
