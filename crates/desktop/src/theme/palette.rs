use eframe::egui::Color32;

pub struct Palette {
    pub bg: Color32,
    pub sidebar: Color32,
    pub surface: Color32,
    pub surface_alt: Color32,
    pub border: Color32,
    pub fg: Color32,
    pub muted: Color32,
    pub subtle: Color32,
    pub accent: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,
    pub purple: Color32,
    pub on_accent: Color32,
}

const fn rgb(value: u32) -> Color32 {
    Color32::from_rgb((value >> 16) as u8, (value >> 8) as u8, value as u8)
}

pub const DARK: Palette = Palette {
    bg: rgb(0x1e1e2e),
    sidebar: rgb(0x181825),
    surface: rgb(0x313244),
    surface_alt: rgb(0x45475a),
    border: rgb(0x585b70),
    fg: rgb(0xcdd6f4),
    muted: rgb(0xa6adc8),
    subtle: rgb(0x7f849c),
    accent: rgb(0x89b4fa),
    success: rgb(0xa6e3a1),
    warning: rgb(0xf9e2af),
    danger: rgb(0xf38ba8),
    purple: rgb(0xcba6f7),
    on_accent: rgb(0x181825),
};

pub const LIGHT: Palette = Palette {
    bg: rgb(0xf7f8fb),
    sidebar: rgb(0xe9ecf3),
    surface: rgb(0xffffff),
    surface_alt: rgb(0xdfe3ec),
    border: rgb(0xb9c0ce),
    fg: rgb(0x2b2f3d),
    muted: rgb(0x545b6e),
    subtle: rgb(0x6b7285),
    accent: rgb(0x1e5fd0),
    success: rgb(0x1f7a34),
    warning: rgb(0x8f5b00),
    danger: rgb(0xc2244f),
    purple: rgb(0x7438c9),
    on_accent: rgb(0xffffff),
};
