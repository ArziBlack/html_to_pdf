fn main() {
    println!(r"cargo:rustc-link-search=native=C:\Program Files\wkhtmltopdf\lib");
    println!("cargo:rustc-link-lib=static=wkhtmltox"); // or `dylib` if you want dynamic linking
}
