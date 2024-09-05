fn main() {
    slint_build::compile("src/ui/login.slint").unwrap();
    slint_build::compile("src/ui/main.slint").unwrap();
}