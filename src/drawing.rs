//! Cutting layouts as a SolidCraft drawing (PLUG-7): a sheet of paper for each board sheet the
//! nesting opens, the board drawn to scale with each part's rectangle, name and size, and the
//! board in the title block. Made in the design, so it prints and exports as SolidCraft's own
//! drawings do; the PDFs stay as exports.
use cabinet_core::nesting::Nesting;
use serde_json::{Value, json};

use crate::{PluginError, run};

/// The drawing's name: a run replaces the one it made before.
pub const NAME: &str = "Cutting layouts";

/// Where a board goes on A3 landscape (420 × 297), in paper mm: inside the border, above the
/// title block (180 × 61 at the lower right), with a line for the heading above it.
const AREA: (f64, f64, f64, f64) = (24.0, 78.0, 384.0, 190.0);
const HEADING_Y: f64 = 276.0;

fn short(x: f64) -> String {
    format!("{x:.0}")
}

/// Makes the drawing's sheets, replacing the ones an earlier run made, and leaves it open. The
/// drawing itself is kept, so a sheet of it open in the window stays open: SolidCraft 0.51 draws a
/// sheet's sketch only on a sheet open in the window, so the boards are drawn to scale when it is,
/// and listed as a table of parts when it isn't. `label` names a board id.
pub fn layouts(nested: &Nesting, label: impl Fn(&str) -> String) -> Result<Value, PluginError> {
    if nested.sheets.is_empty() {
        return Ok(json!({"sheets": 0}));
    }
    let count = nested.sheets.len();
    let title = |i: usize| format!("{} {}", i + 1, label(&nested.sheets[i].board));
    let listed = run("drawing.list", &json!({}))?;
    let existing = listed["drawings"].as_array().into_iter().flatten().find(|d| d["name"] == NAME).cloned();
    let (mut old, mut first): (Vec<String>, Option<String>) = (Vec::new(), None);
    match existing {
        Some(d) => {
            let opened = run("drawing.open", &json!({"drawing": d["id"]}))?;
            old = opened["sheets"].as_array().into_iter().flatten().filter_map(|s| s["id"].as_str().map(String::from)).collect();
        }
        None => {
            let made = run("drawing.create", &json!({"name": NAME, "template": "iso-a3-part", "first_sheet_name": title(0)}))?;
            first = made["sheets"][0]["id"].as_str().map(String::from);
        }
    }
    let mut drawn = 0;
    for (i, s) in nested.sheets.iter().enumerate() {
        let sheet_id = match first.take() {
            Some(id) => id,
            None => run("drawing.sheet.add", &json!({"name": title(i)}))?["sheet"].as_str().unwrap_or_default().to_string(),
        };
        let board = format!("{}, {} mm", label(&s.board), s.thickness);
        run(
            "drawing.sheet.set",
            &json!({"sheet": sheet_id, "name": title(i), "title_block_overrides": {
                "title": NAME, "material": board, "document_type": "Cutting layout",
                "general_tolerances": "Sizes are cut sizes, in mm"}}),
        )?;
        let note = |at: [f64; 2], cap: f64, align: &str, content: String| {
            run("drawing.note.add", &json!({"anchor": {"sheet": sheet_id}, "at": at, "cap_mm": cap, "align": align, "content": content}))
        };
        let (x0, y0, w, h) = AREA;
        let used: f64 = s.placements.iter().map(|p| p.rect.length * p.rect.width).sum();
        note([x0, HEADING_Y], 3.5, "left", format!(
            "{board}: sheet {} of {count}, {} × {}, {} parts, {:.0} % used",
            i + 1, short(s.stock.length), short(s.stock.width), s.placements.len(),
            100.0 * used / (s.stock.length * s.stock.width)))?;
        // The board to scale: its length across the paper, its width up it.
        let k = (w / s.stock.length).min(h / s.stock.width);
        let at = |x: f64, y: f64| [x0 + x * k, y0 + y * k];
        if run("drawing.sketch.create", &json!({"owner": sheet_id, "name": format!("Sheet {}", i + 1)})).is_ok() {
            run("sketch.rectangle.two_point", &json!({"p0": at(0.0, 0.0), "p1": at(s.stock.length, s.stock.width)}))?;
            for p in &s.placements {
                let r = &p.rect;
                run("sketch.rectangle.two_point", &json!({"p0": at(r.x, r.y), "p1": at(r.x + r.length, r.y + r.width)}))?;
            }
            run("sketch.finish", &json!({}))?;
            for p in &s.placements {
                let r = &p.rect;
                // Text sized to the part on paper: two lines, the longer about 14 characters.
                let cap = (r.length * k / 12.0).min(r.width * k / 4.0).clamp(0.8, 3.0);
                let c = at(r.x + r.length / 2.0, r.y + r.width / 2.0);
                note(c, cap, "centre", format!("{}\n{} × {}", p.part, short(r.length), short(r.width)))?;
            }
            drawn += 1;
        } else {
            // A table of the sheet's parts: a note a column, their lines in step.
            let rows = s.placements.len().max(1) as f64;
            let cap = (h / (rows + 1.0) / 1.7).clamp(1.2, 3.5);
            let column = |head: &str, cells: Vec<String>| std::iter::once(head.to_string()).chain(cells).collect::<Vec<_>>().join("\n");
            let top = y0 + h;
            note([x0, top], cap, "left", column("Part", s.placements.iter().map(|p| p.part.clone()).collect()))?;
            note([x0 + 200.0, top], cap, "right", column("Length", s.placements.iter().map(|p| short(p.rect.length)).collect()))?;
            note([x0 + 240.0, top], cap, "right", column("Width", s.placements.iter().map(|p| short(p.rect.width)).collect()))?;
            note([x0 + 290.0, top], cap, "right", column("At (x, y)", s.placements.iter().map(|p| format!("{}, {}", short(p.rect.x), short(p.rect.y))).collect()))?;
        }
    }
    for id in &old {
        run("drawing.sheet.delete", &json!({"sheet": id, "contents": "delete"}))?;
    }
    Ok(json!({"sheets": count, "drawn": drawn, "unplaced": nested.unplaced}))
}
