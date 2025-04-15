fn main() {
    println!("cargo:rustc-link-search=native=/Applications/CPLEX_Studio2211/cplex/lib/x86-64_osx/static_pic");
    println!("cargo:rustc-link-search=native=/Applications/CPLEX_Studio2211/cplex/bin/x86-64_osx");
    println!("cargo:rustc-link-lib=dylib=cplex2211")
}