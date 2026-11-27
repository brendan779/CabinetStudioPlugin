//! The board catalogue and the job's boards by role (docs/PLAN.md, Boards). The catalogue comes as
//! `boards.tsv`, an index `examples/catalog.rs` writes from Cabinet Studio's catalogue: scanning it
//! fits a dialog hook's budget, where reading the whole catalogue does not.
use serde_json::{Value, json};

const INDEX: &str = include_str!("boards.tsv");

/// The roles a board can take, as the Boards dialog's "Use for" shows them.
pub const ROLES: &[(&str, &str)] = &[("carcass", "Carcass"), ("fronts", "Fronts"), ("backs", "Backs"), ("benchtop", "Benchtop")];

/// One line of the index.
pub struct Board<'a> {
    pub id: &'a str,
    pub brand: &'a str,
    pub range: &'a str,
    pub decor: &'a str,
    pub finish: &'a str,
    pub thickness: f64,
    /// The largest sheet ("3600 × 1800"): the cutting layouts' stock.
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

/// The index's boards: its first line is the brands' table of contents.
fn body() -> &'static str {
    INDEX.split_once('\n').map_or("", |(_, rest)| rest)
}

pub fn all() -> impl Iterator<Item = Board<'static>> {
    body().lines().filter_map(parse)
}

/// One brand's boards, read from its span of the index alone: a dialog hook may spend only 10
/// million instructions (SolidCraft's fixed budget), too few to read the whole index.
pub fn of_brand(brand: &str) -> Vec<Board<'static>> {
    let head = INDEX.split_once('\n').map_or("", |(h, _)| h);
    let span = head.split('\t').find_map(|e| {
        let (b, range) = e.rsplit_once('=')?;
        let (s, f) = range.split_once('-')?;
        (b == brand).then(|| (s.parse::<usize>().ok(), f.parse::<usize>().ok()))
    });
    match span {
        Some((Some(s), Some(f))) => body().get(s..f).unwrap_or("").lines().filter_map(parse).collect(),
        _ => Vec::new(),
    }
}

pub fn role_of(label: &str) -> &'static str {
    ROLES.iter().find(|(id, l)| *l == label || *id == label).map_or("carcass", |(id, _)| id)
}

pub fn label_of(role: &str) -> &'static str {
    ROLES.iter().find(|(id, _)| *id == role).map_or("Carcass", |(_, l)| l)
}

/// Benchtop materials for a benchtop; sheet boards, linings left out, for the rest.
pub fn suits(b: &Board, role: &str) -> bool {
    if role == "benchtop" { b.use_as == "top" } else { b.use_as == "sheet" }
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

/// What the Boards dialog offers for its choices so far: each list in the manifest's order (sorted),
/// the choice kept where it still fits, or the first that does.
#[derive(Debug, Default)]
pub struct Narrowed {
    pub ranges: Vec<String>,
    pub range: String,
    pub decors: Vec<String>,
    pub decor: String,
    pub finishes: Vec<String>,
    pub finish: String,
    pub thicknesses: Vec<f64>,
    /// False when nothing in the brand matches `find`.
    pub matched: bool,
}

/// Narrows the Boards dialog's lists. "Find decor" looks through every range of the brand, so
/// Range offers only ranges that have a match, and a range without one gives way to the one with
/// the most (0.4.1 looked in the current range alone, and found nothing: owner, 2026-11-28).
pub fn narrow(brand: &str, role: &str, range: &str, find: &str, decor: &str, finish: &str) -> Narrowed {
    let find = find.trim().to_lowercase();
    let rows: Vec<Board> = of_brand(brand).into_iter().filter(|b| suits(b, role)).collect();
    let hits: Vec<&Board> = rows.iter().filter(|b| find.is_empty() || b.decor.to_lowercase().contains(&find)).collect();
    let matched = !hits.is_empty();
    let pool: Vec<&Board> = if matched { hits } else { rows.iter().collect() };
    let sorted = |mut v: Vec<&str>| {
        v.sort();
        v.dedup();
        v.into_iter().map(String::from).collect::<Vec<String>>()
    };
    let ranges = sorted(pool.iter().map(|b| b.range).collect());
    let range = if ranges.iter().any(|r| r == range) {
        range.to_string()
    } else {
        // The range with the most boards: Laminex opens on Decorated MDF, not Compact Laminate.
        ranges
            .iter()
            .max_by_key(|r| (pool.iter().filter(|b| &b.range == *r).count(), std::cmp::Reverse((*r).clone())))
            .cloned()
            .unwrap_or_default()
    };
    let in_range: Vec<&&Board> = pool.iter().filter(|b| b.range == range).collect();
    let decors = sorted(in_range.iter().map(|b| b.decor).collect());
    let decor = if decors.iter().any(|d| d == decor) { decor.to_string() } else { decors.first().cloned().unwrap_or_default() };
    let of_decor: Vec<&&&Board> = in_range.iter().filter(|b| b.decor == decor).collect();
    let finishes = sorted(of_decor.iter().map(|b| b.finish).collect());
    let finish = if finishes.iter().any(|f| f == finish) { finish.to_string() } else { finishes.first().cloned().unwrap_or_default() };
    let mut thicknesses: Vec<f64> = of_decor.iter().filter(|b| b.finish == finish).map(|b| b.thickness).collect();
    thicknesses.sort_by(f64::total_cmp);
    thicknesses.dedup();
    Narrowed { ranges, range, decors, decor, finishes, finish, thicknesses, matched }
}

/// The board with `id`, from the whole index.
pub fn find(id: &str) -> Option<Board<'static>> {
    all().find(|b| b.id == id)
}

/// A board's largest sheet as length and width.
pub fn sheet_size(b: &Board) -> Option<(f64, f64)> {
    let (l, w) = b.sheet.split_once(" × ")?;
    Some((l.trim().parse().ok()?, w.trim().parse().ok()?))
}
