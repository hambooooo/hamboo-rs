fn main() {
    println!("cargo:rustc-link-arg-bins=-Tlinkall.x");
    slint_build::compile_with_config(
        "ui/appwindow.slint",
        slint_build::CompilerConfiguration::new()
            .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer),
    )
        .unwrap();
}