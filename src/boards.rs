//! PLUG-2: the board catalogue, built into Cabinet Studio's `cabinet-catalog`, and the job's boards
//! by role (docs/PLAN.md, The panel). The panel's Boards tab lists the catalogue as text; picking a
//! row gives the chosen role that board.
use cabinet_catalog::{Board, BoardKind};
use serde_json::{Value, json};
use std::sync::OnceLock;

/// The roles a board can take, as the Boards tab's "Use for" choice shows them.
pub const ROLES: &[(&str, &str)] = &[("carcass", "Carcass"), ("fronts", "Fronts"), ("backs", "Backs"), ("benchtop", "Benchtop")];

/// The most rows the Boards table shows at once: a panel's data is limited to 1 MiB.
pub const MOST_ROWS: usize = 200;

pub fn all() -> &'static [Board] {
    static BOARDS: OnceLock<Vec<Board>> = OnceLock::new();
    BOARDS.get_or_init(|| cabinet_catalog::embedded::catalog().map(|c| c.boards).unwrap_or_default())
}

pub fn find(id: &str) -> Option<&'static Board> {
    all().iter().find(|b| b.id == id)
}

pub fn role_of(label: &str) -> &'static str {
    ROLES.iter().find(|(id, l)| *l == label || *id == label).map_or("carcass", |(id, _)| id)
}

pub fn label_of(role: &str) -> &'static str {
    ROLES.iter().find(|(id, _)| *id == role).map_or("Carcass", |(_, l)| l)
}

/// Whether a board can take `role`: benchtop materials for a benchtop, sheet boards for the rest.
fn suits(b: &Board, role: &str) -> bool {
    let top = matches!(b.kind, BoardKind::LaminateTop | BoardKind::SinteredStone);
    match role {
        "benchtop" => top,
        _ => !top && b.kind != BoardKind::Lining,
    }
}

/// What a row shows of a board, and what the search looks in.
pub fn sheet(b: &Board) -> String {
    b.sheets
        .iter()
        .max_by(|x, y| (x.length * x.width).total_cmp(&(y.length * y.width)))
        .map_or(String::new(), |s| format!("{:.0} × {:.0}", s.length, s.width))
}

fn words(b: &Board) -> String {
    format!("{} {} {} {} {} {:?}", b.brand, b.range, b.decor, b.finish, b.thickness, b.kind).to_lowercase()
}

/// The boards for `role` whose text has every word of `search`, in catalogue order, and how many
/// matched in all.
pub fn matching(role: &str, search: &str) -> (Vec<&'static Board>, usize) {
    let terms: Vec<String> = search.split_whitespace().map(str::to_lowercase).collect();
    let all: Vec<&Board> = all()
        .iter()
        .filter(|b| suits(b, role) && terms.iter().all(|t| words(b).contains(t.as_str())))
        .collect();
    let n = all.len();
    (all.into_iter().take(MOST_ROWS).collect(), n)
}

/// The job's standard and boards with `board` in `role`: a cabinet role sets the core's board and
/// its thickness, the benchtop waits for PLUG-3.
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
    job["boards"][role] = json!({"id": board.id, "colour": board.preview_colour, "name": name(board)});
}

/// A board in words: "Polytec Natural Oak Ravine, 16 mm".
pub fn name(b: &Board) -> String {
    let finish = if b.finish.is_empty() { String::new() } else { format!(" {}", b.finish) };
    format!("{} {}{finish}, {} mm", b.brand, b.decor, b.thickness)
}
