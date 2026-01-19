use eframe::egui;

include!(concat!(env!("OUT_DIR"), "/embedded_icon_png.rs"));

pub fn window_icon() -> Option<egui::IconData> {
    if ICON_PNG.is_empty() {
        return None;
    }

    let image = image::load_from_memory(ICON_PNG).ok()?.into_rgba8();
    let (width, height) = image.dimensions();

    Some(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
}
