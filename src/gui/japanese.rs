use eframe::egui;
use egui::FontFamily;

// フォント設定用の関数
pub fn setup_custom_fonts(ctx: &egui::Context) {
    // フォント設定を取得
    let mut fonts = egui::FontDefinitions::default();

    // 日本語フォント（可変ウェイト）を追加
    fonts.font_data.insert(
        "noto_sans_jp".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../assets/NotoSansJP-VariableFont_wght.ttf"
        ))
        .into(),
    );

    // フォントファミリーに追加
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "noto_sans_jp".to_owned()); // 一番優先度高く追加

    // モノスペースフォントにも日本語フォントを追加
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push("noto_sans_jp".to_owned());

    // フォント設定を適用
    ctx.set_fonts(fonts);
}
