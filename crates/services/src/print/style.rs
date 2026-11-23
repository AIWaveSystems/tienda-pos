use pos_config::PrinterFormat;

const PAGE: &str = include_str!("page.css");
const THERMAL: &str = include_str!("thermal.css");

pub fn style(format: PrinterFormat) -> String {
    match format {
        PrinterFormat::A4 => PAGE.to_owned(),
        PrinterFormat::Thermal80 => THERMAL.replace("__WIDTH__", "80"),
        PrinterFormat::Thermal58 => THERMAL.replace("__WIDTH__", "58"),
    }
}
