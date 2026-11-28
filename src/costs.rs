//! Costs (PLUG-8): the core's quote from the shop's price list. The price list is the shop's, kept
//! in user storage so it carries from job to job; the Cut List dialog's Prices section edits the
//! prices of the job's own boards, and the rates.
use cabinet_core::quote::{PriceList, Quote, Section};
use serde_json::{Value, json};

use crate::boards;
use crate::spec::Job;

const KEY: &str = "prices";
/// What the cut list calls a part with no board chosen, and the price list keys it by.
pub const GENERIC: &str = "Generic";

/// The Prices section's fields: (field, what it prices).
pub const BOARD_FIELDS: &[(&str, &str)] =
    &[("price_carcass", "carcass"), ("price_fronts", "fronts"), ("price_backs", "backs"), ("price_benchtop", "benchtop")];

pub fn stored() -> PriceList {
    crate::craft::plugin::user_storage::get(KEY)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn store(prices: &PriceList) -> Result<(), crate::PluginError> {
    crate::craft::plugin::user_storage::set(KEY, &serde_json::to_string(prices).unwrap_or_default())
}

/// The board each role's parts are cut from, as the cut list names it.
pub fn role_board(job_value: &Value, role: &str) -> String {
    let job = Job::from_stored(job_value);
    let standard = serde_json::to_value(&job.standard).unwrap_or(Value::Null);
    let id = match role {
        "carcass" => standard["carcass_board"].as_str().unwrap_or_default().to_string(),
        "fronts" => standard["front_board"].as_str().unwrap_or_default().to_string(),
        "backs" => standard["back_board"].as_str().unwrap_or_default().to_string(),
        _ => job.benchtop_board.clone(),
    };
    if id.is_empty() { GENERIC.into() } else { id }
}

/// The dialog's fields as the stored price list has them, for this job's boards.
pub fn fields(job_value: &Value) -> Value {
    let p = stored();
    let mut patch = json!({});
    for (field, role) in BOARD_FIELDS {
        patch[*field] = json!({"value": p.boards.get(&role_board(job_value, role)).copied().unwrap_or(0.0)});
    }
    patch["edge_tape_per_m"] = json!({"value": p.edge_tape_per_m.unwrap_or(0.0)});
    patch["labour_per_hour"] = json!({"value": p.labour_per_hour.unwrap_or(0.0)});
    patch["minutes_per_part"] = json!({"value": p.minutes_per_part});
    patch["minutes_per_cabinet"] = json!({"value": p.minutes_per_cabinet});
    patch["minutes_per_hardware_item"] = json!({"value": p.minutes_per_hardware_item});
    patch["markup_percent"] = json!({"value": p.markup * 100.0});
    patch["tax_percent"] = json!({"value": p.tax * 100.0});
    patch
}

/// The stored price list with the dialog's values: a price of 0 is no price (reported missing).
pub fn from_dialog(p: &Value, job_value: &Value) -> PriceList {
    let mut prices = stored();
    let price = |k: &str| p[k].as_f64().filter(|x| *x > 0.0);
    for (field, role) in BOARD_FIELDS {
        if p[*field].is_null() {
            continue;
        }
        let board = role_board(job_value, role);
        match price(field) {
            Some(x) => prices.boards.insert(board, x),
            None => prices.boards.remove(&board),
        };
    }
    if !p["edge_tape_per_m"].is_null() {
        prices.edge_tape_per_m = price("edge_tape_per_m");
    }
    if !p["labour_per_hour"].is_null() {
        prices.labour_per_hour = price("labour_per_hour");
    }
    let number = |k: &str, to: &mut f64, scale: f64| {
        if let Some(x) = p[k].as_f64().filter(|x| *x >= 0.0) {
            *to = x / scale;
        }
    };
    number("minutes_per_part", &mut prices.minutes_per_part, 1.0);
    number("minutes_per_cabinet", &mut prices.minutes_per_cabinet, 1.0);
    number("minutes_per_hardware_item", &mut prices.minutes_per_hardware_item, 1.0);
    number("markup_percent", &mut prices.markup, 100.0);
    number("tax_percent", &mut prices.tax, 100.0);
    prices
}

/// A board id in words, as the quote's lines show it.
fn board_label(id: &str) -> String {
    boards::find(id).map_or_else(|| if id == GENERIC { "Generic board".into() } else { id.to_string() }, |b| boards::name(&b))
}

/// The quote's lines with the boards named, not their ids.
pub fn named(mut q: Quote) -> Quote {
    for l in &mut q.lines {
        if matches!(l.section, Section::Board | Section::Benchtop) {
            let label = board_label(&l.key);
            l.description = l.description.replacen(&l.key, &label, 1);
        }
    }
    q
}

fn money(x: f64) -> String {
    format!("{x:.2}")
}

/// The quote as a spreadsheet: a line each, then the totals and what has no price.
pub fn csv(q: &Quote) -> String {
    let field = |s: &str| if s.contains([',', '"', '\n']) { format!("\"{}\"", s.replace('"', "\"\"")) } else { s.to_string() };
    let mut out = format!("Section,Item,Quantity,Unit,Unit price ({}),Total ({})\n", q.currency, q.currency);
    for l in &q.lines {
        let section = serde_json::to_value(l.section).ok().and_then(|v| v.as_str().map(String::from)).unwrap_or_default();
        out += &format!(
            "{},{},{},{},{},{}\n",
            section,
            field(&l.description),
            l.quantity,
            field(&l.unit),
            l.unit_price.map_or("no price".into(), money),
            money(l.total)
        );
    }
    out += &format!(",Cost,,,,{}\n,Markup,,,,{}\n,Tax,,,,{}\n,Total,,,,{}\n", money(q.cost), money(q.markup), money(q.tax), money(q.total));
    for m in &q.missing {
        out += &format!(",No price: {},,,,\n", field(m));
    }
    out
}

/// The Cabinets panel's Costs table: (item, quantity, price, total) a row.
pub fn rows(q: &Quote) -> Vec<[String; 4]> {
    let mut rows: Vec<[String; 4]> = q
        .lines
        .iter()
        .map(|l| {
            [
                l.description.clone(),
                format!("{} {}", l.quantity, l.unit),
                l.unit_price.map_or("no price".into(), |p| format!("{} / {}", money(p), l.unit)),
                money(l.total),
            ]
        })
        .collect();
    let total = |name: &str, x: f64| [name.to_string(), String::new(), String::new(), money(x)];
    rows.push(total("Cost", q.cost));
    rows.push(total("Markup", q.markup));
    rows.push(total("Tax", q.tax));
    rows.push(total(&format!("Total ({})", q.currency), q.total));
    rows
}
