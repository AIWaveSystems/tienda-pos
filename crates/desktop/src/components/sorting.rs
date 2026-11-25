use std::cmp::Ordering;

use super::table::Row;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sorting {
    pub column: usize,
    pub descending: bool,
}

impl Sorting {
    pub fn toggle(current: Option<Self>, column: usize) -> Option<Self> {
        match current {
            Some(sorting) if sorting.column == column && sorting.descending => None,
            Some(sorting) if sorting.column == column => Some(Self {
                column,
                descending: true,
            }),
            _ => Some(Self {
                column,
                descending: false,
            }),
        }
    }

    pub fn order(self, rows: &[Row]) -> Vec<usize> {
        let mut order: Vec<usize> = (0..rows.len()).collect();
        order.sort_by(|a, b| {
            let compared = compare(&rows[*a].cells[self.column], &rows[*b].cells[self.column]);
            if self.descending {
                compared.reverse()
            } else {
                compared
            }
        });
        order
    }

    pub fn mark(current: Option<Self>, column: usize) -> &'static str {
        match current {
            Some(sorting) if sorting.column == column && sorting.descending => " ⏷",
            Some(sorting) if sorting.column == column => " ⏶",
            _ => "",
        }
    }
}

fn compare(left: &str, right: &str) -> Ordering {
    match (number(left), number(right)) {
        (Some(a), Some(b)) => a.total_cmp(&b),
        _ => left.to_lowercase().cmp(&right.to_lowercase()),
    }
}

fn number(text: &str) -> Option<f64> {
    text.replace(',', "").parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_and_text_are_ordered() {
        let rows: Vec<Row> = ["1,000.00", "20.00", "-5.00"]
            .iter()
            .map(|text| Row::new(vec![(*text).to_owned()]))
            .collect();
        let up = Sorting::toggle(None, 0).unwrap();
        assert_eq!(up.order(&rows), vec![2, 1, 0]);
        let down = Sorting::toggle(Some(up), 0).unwrap();
        assert_eq!(down.order(&rows), vec![0, 1, 2]);
        assert_eq!(Sorting::toggle(Some(down), 0), None);
        assert_eq!(compare("ana", "Beto"), Ordering::Less);
    }
}
