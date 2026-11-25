use super::table::Listing;

const BYTE_ORDER_MARK: &str = "\u{feff}";

pub fn csv(listing: &Listing) -> String {
    let header: Vec<&str> = listing.columns.iter().map(|column| column.label).collect();
    let mut text = format!("{BYTE_ORDER_MARK}{}", line(&header));
    for row in &listing.rows {
        let cells: Vec<String> = row
            .iter()
            .zip(&listing.columns)
            .map(|(value, column)| {
                if column.numeric {
                    value.replace(',', "")
                } else {
                    neutral(value)
                }
            })
            .collect();
        text.push_str(&line(&cells));
    }
    text
}

fn line<T: AsRef<str>>(cells: &[T]) -> String {
    let quoted: Vec<String> = cells.iter().map(|cell| quote(cell.as_ref())).collect();
    format!("{}\r\n", quoted.join(","))
}

fn quote(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        return format!("\"{}\"", value.replace('"', "\"\""));
    }
    value.to_owned()
}

fn neutral(value: &str) -> String {
    if value.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        return format!("'{value}");
    }
    value.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::print::Column;

    #[test]
    fn values_are_quoted_and_formulas_neutralized() {
        let listing = Listing {
            columns: vec![Column::text("Nombre"), Column::number("Precio")],
            rows: vec![
                vec!["Pan, \"fino\"".into(), "1,500.00".into()],
                vec!["=1+1".into(), "-2".into()],
            ],
            ..Listing::default()
        };
        assert_eq!(
            csv(&listing),
            "\u{feff}Nombre,Precio\r\n\"Pan, \"\"fino\"\"\",1500.00\r\n'=1+1,-2\r\n"
        );
    }
}
