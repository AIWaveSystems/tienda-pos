use std::collections::HashMap;
use std::sync::Arc;

use chrono::TimeDelta;
use pos_domain::{now, Product, Sale};
use pos_storage::Store;
use rust_decimal::Decimal;

use crate::daily::DailyReport;
use crate::{Period, Result, Span};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SalesSummary {
    pub total: Decimal,
    pub count: usize,
    pub average: Decimal,
}

#[derive(Clone, Debug, Default)]
pub struct InventorySummary {
    pub products: usize,
    pub low_stock: Vec<Product>,
    pub value: Decimal,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TopProduct {
    pub name: String,
    pub quantity: i64,
    pub revenue: Decimal,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DailySales {
    pub date: String,
    pub count: usize,
    pub total: Decimal,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SellerSales {
    pub name: String,
    pub count: usize,
    pub total: Decimal,
}

#[derive(Clone, Debug, Default)]
pub struct Overview {
    pub sales: SalesSummary,
    pub inventory: InventorySummary,
    pub top: Vec<TopProduct>,
    pub daily: Vec<DailySales>,
    pub sellers: Vec<SellerSales>,
    pub profit: Decimal,
}

pub struct ReportService {
    store: Arc<dyn Store>,
}

impl ReportService {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self { store }
    }

    pub fn overview(&self, top: usize, period: Period) -> Result<Overview> {
        let today = now().date();
        let start = period.start(today);
        let mut sales = self.store.sales().all()?;
        sales.retain(|sale| !sale.is_voided() && sale.created_at.date() >= start);
        let products = self.store.products().all()?;
        Ok(Overview {
            sales: summarize(&sales),
            profit: profit(&sales, &products),
            sellers: sellers(&sales),
            inventory: inventory(products),
            top: top_products(&sales, top),
            daily: daily(&sales, period.days(today)),
        })
    }

    pub fn daily(&self, span: Span) -> Result<DailyReport> {
        Ok(DailyReport::build(
            span,
            self.store.sales().all()?,
            self.store.cash().all()?,
            &self.store.products().all()?,
        ))
    }

    pub fn client_summary(&self, client_id: &str) -> Result<SalesSummary> {
        let mut sales = self.store.sales().all()?;
        sales.retain(|sale| sale.client_id == client_id && !sale.is_voided());
        Ok(summarize(&sales))
    }
}

pub fn summarize(sales: &[Sale]) -> SalesSummary {
    let total: Decimal = sales.iter().map(|sale| sale.total).sum();
    let average = if sales.is_empty() {
        Decimal::ZERO
    } else {
        (total / Decimal::from(sales.len())).round_dp(2)
    };
    SalesSummary {
        total,
        count: sales.len(),
        average,
    }
}

fn inventory(mut products: Vec<Product>) -> InventorySummary {
    products.retain(|product| product.active);
    InventorySummary {
        products: products.len(),
        value: products.iter().map(Product::inventory_value).sum(),
        low_stock: products.into_iter().filter(Product::low_stock).collect(),
    }
}

fn top_products(sales: &[Sale], limit: usize) -> Vec<TopProduct> {
    let mut totals: HashMap<&str, TopProduct> = HashMap::new();
    for item in sales.iter().flat_map(|sale| &sale.items) {
        let entry = totals
            .entry(item.product_id.as_str())
            .or_insert_with(|| TopProduct {
                name: item.product_name.clone(),
                quantity: 0,
                revenue: Decimal::ZERO,
            });
        entry.quantity += item.quantity;
        entry.revenue += item.subtotal;
    }
    let mut ranked: Vec<TopProduct> = totals.into_values().collect();
    ranked.sort_by(|a, b| b.revenue.cmp(&a.revenue).then(a.name.cmp(&b.name)));
    ranked.truncate(limit);
    ranked
}

fn sellers(sales: &[Sale]) -> Vec<SellerSales> {
    let mut totals: HashMap<&str, SellerSales> = HashMap::new();
    for sale in sales {
        let name = match sale.seller.as_str() {
            "" => "Sin vendedor",
            name => name,
        };
        let entry = totals.entry(name).or_insert_with(|| SellerSales {
            name: name.to_owned(),
            count: 0,
            total: Decimal::ZERO,
        });
        entry.count += 1;
        entry.total += sale.total;
    }
    let mut ranked: Vec<SellerSales> = totals.into_values().collect();
    ranked.sort_by(|a, b| b.total.cmp(&a.total).then(a.name.cmp(&b.name)));
    ranked
}

pub(crate) fn profit(sales: &[Sale], products: &[Product]) -> Decimal {
    let costs: HashMap<&str, Decimal> = products
        .iter()
        .map(|product| (product.id.as_str(), product.cost))
        .collect();
    sales
        .iter()
        .map(|sale| {
            let cost: Decimal = sale
                .items
                .iter()
                .filter_map(|item| {
                    Some(Decimal::from(item.quantity) * costs.get(item.product_id.as_str())?)
                })
                .sum();
            sale.subtotal - sale.discount - cost
        })
        .sum()
}

fn daily(sales: &[Sale], days: i64) -> Vec<DailySales> {
    let today = now().date();
    (0..days)
        .rev()
        .map(|offset| {
            let date = today - TimeDelta::days(offset);
            let of_day = || sales.iter().filter(|sale| sale.created_at.date() == date);
            DailySales {
                date: date.to_string(),
                count: of_day().count(),
                total: of_day().map(|sale| sale.total).sum(),
            }
        })
        .collect()
}
