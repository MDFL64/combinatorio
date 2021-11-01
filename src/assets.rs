use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets"]
struct Asset;

pub fn get_asset_string(filename: &str) -> Option<String> {
    // Since rust_embed uses cows and directly reads the FS in debug mode,
    // we unfortunately can't just hand out references to the static data.
    if let Some(file) = Asset::get(filename) {
        std::str::from_utf8(file.data.as_ref()).ok().map(|x| x.to_owned())
    } else {
        None
    }
}
