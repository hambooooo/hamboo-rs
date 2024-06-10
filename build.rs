fn main() {
    slint_build::compile_with_config(
        "ui/ui.slint",
        slint_build::CompilerConfiguration::new()
            .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer),
    ).unwrap();
}