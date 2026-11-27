//! The board catalogue and the job's boards by role (docs/PLAN.md, Boards). The catalogue comes as
//! `boards.tsv`, an index `examples/catalog.rs` writes from Cabinet Studio's catalogue: scanning it
//! is quick enough for every keystroke in the Boards panel's search, where reading the whole
//! catalogue was not (owner, 2026-11-27).
use serde_json::{Value, json};

const INDEX: &str = include_str!("boards.tsv");

/// The roles a board can take, as the Boards panel's "Use for" choice shows them.
pub const ROLES: &[(&str, &str)] = &[("carcass", "Carcass"), ("fronts", "Fronts"), ("backs", "Backs"), ("benchtop", "Benchtop")];

/// The Brand choice's first option.
pub const ALL_BRANDS: &str = "All brands";

/// The most rows the board table shows at once: a panel's data is limited to 1 MiB.
pub const MOST_ROWS: usize = 300;

/// One line of the index.
pub struct Board<'a> {
    pub id: &'a str,
    pub brand: &'a str,
    pub range: &'a str,
    pub decor: &'a str,
    pub finish: &'a str,
    pub thickness: f64,
    pub sheet: &'a str,
    pub colour: &'a str,
    /// "sheet", "top" (laminate tops and stone) or "lining".
    pub use_as: &'a str,
}

fn parse(line: &str) -> Option<Board<'_>> {
    let mut f = line.split('\t');
    Some(Board {
        id: f.next()?,
        brand: f.next()?,
        range: f.next()?,
        decor: f.next()?,
        finish: f.next()?,
        thickness: f.next()?.parse().ok()?,
        sheet: f.next()?,
        colour: f.next()?,
        use_as: f.next()?,
    })
}

pub fn all() -> impl Iterator<Item = Board<'static>> {
    INDEX.lines().filter_map(parse)
}

pub fn find(id: &str) -> Option<Board<'static>> {
    all().find(|b| b.id == id)
}

/// The brands, in catalogue order, after "All brands": the Brand choice's options.
pub fn brands() -> Vec<String> {
    let mut out = vec![ALL_BRANDS.to_string()];
    for b in all() {
        if !out.iter().any(|x| x == b.brand) {
            out.push(b.brand.to_string());
        }
    }
    out
}

pub fn role_of(label: &str) -> &'static str {
    ROLES.iter().find(|(id, l)| *l == label || *id == label).map_or("carcass", |(id, _)| id)
}

pub fn label_of(role: &str) -> &'static str {
    ROLES.iter().find(|(id, _)| *id == role).map_or("Carcass", |(_, l)| l)
}

/// Benchtop materials for a benchtop; sheet boards, linings left out, for the rest.
fn suits(b: &Board, role: &str) -> bool {
    if role == "benchtop" { b.use_as == "top" } else { b.use_as == "sheet" }
}

/// The boards for `role` of `brand` whose text has every word of `search`, in catalogue order, at
/// most [`MOST_ROWS`], and how many matched in all.
pub fn matching(role: &str, brand: &str, search: &str) -> (Vec<Board<'static>>, usize) {
    let terms: Vec<String> = search.split_whitespace().map(str::to_lowercase).collect();
    let mut rows = Vec::new();
    let mut n = 0;
    for b in all() {
        if !suits(&b, role) || (brand != ALL_BRANDS && !brand.is_empty() && b.brand != brand) {
            continue;
        }
        if !terms.is_empty() {
            let text = format!("{} {} {} {} {}", b.brand, b.range, b.decor, b.finish, b.thickness).to_lowercase();
            if !terms.iter().all(|t| text.contains(t.as_str())) {
                continue;
            }
        }
        n += 1;
        if rows.len() < MOST_ROWS {
            rows.push(b);
        }
    }
    (rows, n)
}

/// The job with `board` in `role`: a cabinet role sets the core's board and its thickness; the
/// benchtop's waits for PLUG-3.
pub fn assign(job: &mut Value, role: &str, board: &Board) {
    let field = match role {
        "carcass" => Some(("carcass_board", "carcass_thickness")),
        "fronts" => Some(("front_board", "front_thickness")),
        "backs" => Some(("back_board", "back_thickness")),
        _ => None,
    };
    if !job["standard"].is_object() {
        job["standard"] = json!({});
    }
    if let Some((id, thickness)) = field {
        job["standard"][id] = json!(board.id);
        job["standard"][thickness] = json!(board.thickness);
    }
    if !job["boards"].is_object() {
        job["boards"] = json!({});
    }
    job["boards"][role] = json!({"id": board.id, "colour": board.colour, "name": name(board)});
}

/// A board in words: "Laminex Delana Oak Natural, 18 mm".
pub fn name(b: &Board) -> String {
    let finish = if b.finish.is_empty() { String::new() } else { format!(" {}", b.finish) };
    format!("{} {}{finish}, {} mm", b.brand, b.decor, b.thickness)
}
