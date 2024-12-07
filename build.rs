fn main() {
    #[cfg(feature = "eyra-as-std")]
    println!("cargo:rustc-link-arg=-nostartfiles");
}
