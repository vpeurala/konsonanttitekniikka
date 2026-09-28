use std::env;
use std::path::Path;

fn main() {
    // macOS shows an app's CFBundleName in the menu bar. A bare executable
    // has no bundle, so embed an Info.plist into the binary instead.
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        let plist = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("macos/Info.plist");
        println!(
            "cargo:rustc-link-arg-bins=-Wl,-sectcreate,__TEXT,__info_plist,{}",
            plist.display()
        );
        println!("cargo:rerun-if-changed=macos/Info.plist");
    }
}
