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
    /// The largest sheet ("3600 × 1800"), for the cut list's stock (PLUG-4).
    #[allow(dead_code)]
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
