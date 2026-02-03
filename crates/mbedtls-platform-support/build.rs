use std::collections::{HashMap, HashSet};
use std::env;

fn main() {
    let env_components = env::var("DEP_MBEDTLS_PLATFORM_COMPONENTS").unwrap();
    let mut sys_platform_components = HashMap::<_, HashSet<_>>::new();
    for mut kv in env_components.split(",").map(|component| component.splitn(2, "=")) {
        let k = kv.next().unwrap();
        let v = kv.next().unwrap();
        let v = v.replace("\\\"", "").replace('"', "").replace('\\', "");
        sys_platform_components
            .entry(k)
            .or_insert_with(Default::default)
            .insert(v.clone());
        println!(r#"cargo:rustc-cfg=sys_{}="{}""#, k, v);
    }

    let mut b = cc::Build::new();
    b.include(env::var_os("DEP_MBEDTLS_INCLUDE").unwrap());
    let config_path = env::var("DEP_MBEDTLS_CONFIG_H").unwrap();
    let config_file = format!(r#""{}""#, config_path.replace('\\', "/"));
    b.define("MBEDTLS_CONFIG_FILE", Some(config_file.as_str()));

    b.file("src/rust_printf.c");
    if sys_platform_components
        .get("c_compiler")
        .map_or(false, |comps| comps.contains("freestanding"))
    {
        b.flag("-U_FORTIFY_SOURCE")
            .define("_FORTIFY_SOURCE", Some("0"))
            .flag("-ffreestanding");
    }
    b.compile("librust-mbedtls-platform-support.a");
    // Force correct link order for mbedtls_printf
    println!("cargo:rustc-link-lib=static=mbedtls");
    println!("cargo:rustc-link-lib=static=mbedx509");
    println!("cargo:rustc-link-lib=static=mbedcrypto");
}
