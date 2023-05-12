use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "resources/"]
pub struct Resource;
