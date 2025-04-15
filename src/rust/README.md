### How to get cplex-rs to work on Mac M1 in Prescriptive Analytics

1. `cargo add cplex-rs`
2.  
3. If you try to compile now, you should get some linker errors — this is because cplex-rs looks for a version of CPLEX corresponding to the computer's architecture (i.e. arm64), and the version we've been provided in cs2951o only has a vesrion for `x86-64_osx`. So, we must tell the Cargo's linker here to look by specifying the `cargo:rustc-link-search=native` parameter via `build.rs`. In `build.rs`, include something like the following:

`
fn main() {
    println!("cargo:rustc-link-search=native=/Applications/CPLEX_Studio2211/cplex/lib/x86-64_osx/static_pic");
}
`
And update how the Cargo.toml with `build = "build.rs"` under the `[packages]` section.

Then run `cargo build`, and everything should be hunky dory.


To run:

`cargo run --target x86_64-apple-darwin`