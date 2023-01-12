/* Copyright (c) Fortanix, Inc.
 *
 * Licensed under the GNU General Public License, version 2 <LICENSE-GPL or
 * https://www.gnu.org/licenses/gpl-2.0.html> or the Apache License, Version
 * 2.0 <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0>, at your
 * option. This file may not be copied, modified, or distributed except
 * according to those terms. */

use bindgen;

use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::Write;
use std::ffi::OsString;
use std::env;

use crate::headers;

#[derive(Debug)]
struct MbedtlsParseCallbacks;

impl bindgen::callbacks::ParseCallbacks for MbedtlsParseCallbacks {
    fn item_name(&self, original_item_name: &str) -> Option<String> {
        Some(original_item_name.trim_start_matches("mbedtls_").trim_start_matches("MBEDTLS_").to_owned())
    }

    fn enum_variant_name(
        &self,
        _enum_name: Option<&str>,
        original_variant_name: &str,
        _variant_value: bindgen::callbacks::EnumVariantValue
    ) -> Option<String> {
        self.item_name(original_variant_name)
    }

    fn int_macro(&self, _name: &str, value: i64) -> Option<bindgen::callbacks::IntKind> {
        if value < (i32::MIN as i64) || value > (i32::MAX as i64) {
            Some(bindgen::callbacks::IntKind::LongLong)
        } else {
            Some(bindgen::callbacks::IntKind::Int)
        }
    }

    fn blocklisted_type_implements_trait(&self, _name: &str, derive_trait: bindgen::callbacks::DeriveTrait) -> Option<bindgen::callbacks::ImplementsTrait> {
        if derive_trait == bindgen::callbacks::DeriveTrait::Default {
            Some(bindgen::callbacks::ImplementsTrait::Manually)
        } else {
            Some(bindgen::callbacks::ImplementsTrait::Yes)
        }
    }
}

/// Add bindgen 0.19-style union accessor methods. These are deprecated
/// and can be deleted with the next major version bump.
fn generate_deprecated_union_accessors(bindings: &str) -> String {
    #[derive(Default)]
    struct UnionImplBuilder {
        impls: String
    }

    impl<'ast> syn::visit::Visit<'ast> for UnionImplBuilder {
        fn visit_item_union(&mut self, i: &'ast syn::ItemUnion) {
            let union_name = &i.ident;
            let field_name = i.fields.named.iter().map(|field| field.ident.as_ref().unwrap());
            let field_type = i.fields.named.iter().map(|field| &field.ty);
            write!(self.impls, "{}", quote::quote! {
                impl #union_name {
                    #(
                        #[deprecated]
                        pub unsafe fn #field_name(&mut self) -> *mut #field_type {
                            &mut self.#field_name
                        }
                    )*
                }
            }).unwrap();
        }
    }

    let mut impl_builder = UnionImplBuilder::default();
    syn::visit::visit_file(&mut impl_builder, &syn::parse_file(&bindings).unwrap());

    impl_builder.impls
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

impl super::BuildConfig {
    pub fn bindgen(&self) {
        let mut input = String::new();
        for h in headers::enabled_ordered() {
            let _ = writeln!(input, "#include <mbedtls/{}>", h);
        }

        let mut cc = cc::Build::new();

        eprint!("\n 1111 end cc.get_compiler().args() \n");
        print_type_of(&cc.get_compiler().args());
        //for arg in cc.get_compiler().args().iter_mut() { 
            //let arg_str = arg.to_str().unwrap();
            //eprint!("\n{}", arg_str);
            //if arg_str.contains("--target") {
                //*arg = OsString::from("--target=arm64-apple-ios13.1-macabi").to_owned();
            //}
        //}
        eprint!("\n\n 1111 start cc.get_compiler().args() \n");

        cc.include(&self.mbedtls_include)
        .flag(&format!(
            "-DMBEDTLS_CONFIG_FILE=\"{}\"",
            self.config_h.to_str().expect("config.h UTF-8 error")
        ));

        eprint!("\n 2222 end cc.get_compiler().args() \n");
        for arg in cc.get_compiler().args().iter() {
            eprint!("\n{}", arg.to_str().unwrap());
        }
        eprint!("\n\n 2222 start cc.get_compiler().args() \n");

        for cflag in &self.cflags {
            cc.flag(cflag);
        }

        eprint!("\n 3333 end cc.get_compiler().args() \n");
        for arg in cc.get_compiler().args().iter() {
            eprint!("\n{}", arg.to_str().unwrap());
        }
        eprint!("\n\n 3333 start cc.get_compiler().args() \n");

        // Determine the sysroot for this compiler so that bindgen
        // uses the correct headers
        let compiler = cc.get_compiler();
        if compiler.is_like_gnu() {
            let output = compiler.to_command().args(&["--print-sysroot"]).output();
            match output {
                Ok(sysroot) => {
                    let path = std::str::from_utf8(&sysroot.stdout).expect("Malformed sysroot");
                    let trimmed_path = path
                        .strip_suffix("\r\n")
                        .or(path.strip_suffix("\n"))
                        .unwrap_or(&path);
                    cc.flag(&format!("--sysroot={}", trimmed_path));
                }
                _ => {} // skip toolchains without a configured sysroot
            };
        }

        // Bindgen utilizes libclang and the current `TARGET` to parse the C files.
        // When the `TARGET` is custom, we need to override it so that bindgen
        // finds the right stdlib headers.
        // See https://docs.rust-embedded.org/embedonomicon/custom-target.html
        // for more details.
        if let Some(target) = env::var_os("RUST_MBEDTLS_BINDGEN_TARGET") {
            env::set_var("TARGET", "arm64-apple-ios13.1-macabi");
        }

        eprint!("\n end cc.get_compiler().args() \n");
        for arg in cc.get_compiler().args().iter() {
            eprint!("\n{}", arg.to_str().unwrap());
        }
        eprint!("\n\n start cc.get_compiler().args() \n");

        let bindings_result = bindgen::builder()
            .clang_args(cc.get_compiler().args().iter().map(|arg| arg.to_str().unwrap()))
            .header_contents("bindgen-input.h", &input)
            .allowlist_function("^(?i)mbedtls_.*")
            .allowlist_type("^(?i)mbedtls_.*")
            .allowlist_var("^(?i)mbedtls_.*")
            .allowlist_recursively(false)
            .blocklist_type("^mbedtls_time_t$")
            .use_core()
            .ctypes_prefix("::types::raw_types")
            .parse_callbacks(Box::new(MbedtlsParseCallbacks))
            .default_enum_style(bindgen::EnumVariation::Consts)
            .generate_comments(false)
            .derive_copy(true)
            .derive_debug(false) // buggy :(
            .derive_default(true)
            .prepend_enum_name(false)
            .translate_enum_integer_types(true)
            .rustfmt_bindings(false)
            .raw_line("#![allow(dead_code, non_snake_case, non_camel_case_types, non_upper_case_globals, invalid_value)]")
            .generate();

        //eprint!("{}", cc_args);

        //eprint!("{}", bindings_result.command_line_flags());
        
        let bindings = bindings_result
            .expect("bindgen error")
            .to_string();

        let union_impls = generate_deprecated_union_accessors(&bindings);

        let bindings_rs = self.out_dir.join("bindings.rs");
        File::create(&bindings_rs)
            .and_then(|mut f| {
                f.write_all(bindings.as_bytes())?;
                f.write_all(union_impls.as_bytes())?;
                f.write_all(b"use crate::types::*;\n")?; // for FILE, time_t, etc.
                Ok(())
            }).expect("bindings.rs I/O error");

        let mod_bindings = self.out_dir.join("mod-bindings.rs");
        fs::write(mod_bindings, b"mod bindings;\n").expect("mod-bindings.rs I/O error");
    }
}
