use chrono::Timelike;
use pos_domain::{CashEntry, CashKind, PaymentMethod, Product, Sale};
use rust_decimal::Decimal;

use crate::report::{profit, summarize, SalesSummary};
use crate::{cash, Span};

#[derive(Clone, Debug, PartialEq)]
pub struct MethodTotal {
    pub method: PaymentMethod,
    pub sales: Decimal,
    pub income: Decimal,
    pub expense: Decimal,
}

impl MethodTotal {
    pub fn balance(&self) -> Decimal {
        self.sales + self.income - self.expense
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HourTotal {
    pub hour: u32,
    pub count: usize,
    pub total: Decimal,
}

#[derive(Clone, Debug)]
pub struct DailyReport {
    pub span: Span,
    pub sales: Vec<Sale>,
    pub voided: usize,
    pub summary: SalesSummary,
    pub entries: Vec<CashEntry>,
    pub income: Decimal,
    pub expense: Decimal,
    pub profit: Decimal,
    pub methods: Vec<MethodTotal>,
    pub hours: Vec<HourTotal>,
}

impl DailyReport {
    pub fn build(
        span: Span,
        sales: Vec<Sale>,
        entries: Vec<CashEntry>,
        products: &[Product],
    ) -> Self {
        let (voided, sales): (Vec<Sale>, Vec<Sale>) = sales
            .into_iter()
            .filter(|sale| span.contains(sale.created_at))
            .partition(Sale::is_voided);
        let entries: Vec<CashEntry> = entries
            .into_iter()
            .filter(|entry| span.contains(entry.created_at) && !entry.is_voided())
            .collect();
        Self {
            span,
            voided: voided.len(),
            summary: summarize(&sales),
            income: cash::total(&entries, CashKind::Entrada),
            expense: cash::total(&entries, CashKind::Gasto),
            profit: profit(&sales, products),
            methods: methods(&sales, &entries),
            hours: hours(&sales),
            sales,
            entries,
        }
    }

    pub fn balance(&self) -> Decimal {
        self.summary.total + self.income - self.expense
    }

    pub fn expected_cash(&self) -> Decimal {
        self.methods
            .iter()
            .find(|item| item.method == PaymentMethod::Efectivo)
            .map(MethodTotal::balance)
            .unwrap_or_default()
    }
}

fn methods(sales: &[Sale], entries: &[CashEntry]) -> Vec<MethodTotal> {
    PaymentMethod::ALL
        .into_iter()
        .map(|method| {
            let own: Vec<CashEntry> = entries
                .iter()
                .filter(|entry| entry.payment_method == method)
                .cloned()
                .collect();
            MethodTotal {
                method,
                sales: sales
                    .iter()
                    .filter(|sale| sale.payment_method == method)
                    .map(|sale| sale.total)
                    .sum(),
                income: cash::total(&own, CashKind::Entrada),
                expense: cash::total(&own, CashKind::Gasto),
            }
        })
        .collect()
}

fn hours(sales: &[Sale]) -> Vec<HourTotal> {
    (0..24)
        .map(|hour| {
            let within = || sales.iter().filter(|sale| sale.created_at.hour() == hour);
            HourTotal {
                hour,
                count: within().count(),
                total: within().map(|sale| sale.total).sum(),
            }
        })
        .collect()
}
