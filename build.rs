fn main() {
    slint_build::compile_with_config(
        "ui/app.slint",
        slint_build::CompilerConfiguration::new()
            .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer),
    )
        .unwrap();
    println!("cargo:rustc-link-arg-bins=-Tlinkall.x");
}