pub fn main() {
    println!("cargo:rerun-if-changed=*");
    // println!("cargo:rustc-cdylib-link-arg=-Wl,-R,/mnt/c/Users/Jasper/CLionProjects/JavaClassTests/target/release");

    println!("cargo:rustc-link-arg=-Wl,-rpath=$ORIGIN");
    println!("cargo:rustc-link-lib=dylib=jvm");
}
