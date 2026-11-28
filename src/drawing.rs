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

/// Makes the drawing, replacing an earlier one of the same name, and leaves it open.
/// `label` names a board id ("Laminex Avignon Walnut Natural").
pub fn layouts(nested: &Nesting, label: impl Fn(&str) -> String) -> Result<Value, PluginError> {
    let listed = run("drawing.list", &json!({}))?;
    for d in listed["drawings"].as_array().into_iter().flatten().filter(|d| d["name"] == NAME) {
        run("drawing.delete", &json!({"drawing": d["id"]}))?;
    }
    if nested.sheets.is_empty() {
        return Ok(json!({"sheets": 0}));
    }
    let count = nested.sheets.len();
    let title = |i: usize| format!("{} {}", i + 1, label(&nested.sheets[i].board));
    let made = run(
        "drawing.create",
        &json!({"name": NAME, "template": "iso-a3-part", "first_sheet_name": title(0)}),
    )?;
    let mut sheet_id = made["sheets"][0]["id"].as_str().unwrap_or_default().to_string();
    for (i, s) in nested.sheets.iter().enumerate() {
        if i > 0 {
            sheet_id = run("drawing.sheet.add", &json!({"name": title(i)}))?["sheet"].as_str().unwrap_or_default().to_string();
        }
        let board = format!("{}, {} mm", label(&s.board), s.thickness);
        run(
            "drawing.sheet.set",
            &json!({"sheet": sheet_id, "title_block_overrides": {
                "title": NAME, "material": board, "document_type": "Cutting layout",
                "general_tolerances": "Sizes are cut sizes, in mm"}}),
        )?;
        // The board to scale: its length across the paper, its width up it.
        let (x0, y0, w, h) = AREA;
        let k = (w / s.stock.length).min(h / s.stock.width);
        let at = |x: f64, y: f64| [x0 + x * k, y0 + y * k];
        run("drawing.sketch.create", &json!({"owner": sheet_id, "name": format!("Sheet {}", i + 1)}))?;
        run("sketch.rectangle.two_point", &json!({"p0": at(0.0, 0.0), "p1": at(s.stock.length, s.stock.width)}))?;
        for p in &s.placements {
            let r = &p.rect;
            run("sketch.rectangle.two_point", &json!({"p0": at(r.x, r.y), "p1": at(r.x + r.length, r.y + r.width)}))?;
        }
        run("sketch.finish", &json!({}))?;
        let used: f64 = s.placements.iter().map(|p| p.rect.length * p.rect.width).sum();
        run(
            "drawing.note.add",
            &json!({"anchor": {"sheet": sheet_id}, "at": [x0, HEADING_Y], "cap_mm": 3.5, "content": format!(
                "{board}: sheet {} of {count}, {} × {}, {} parts, {:.0} % used",
                i + 1, short(s.stock.length), short(s.stock.width), s.placements.len(),
                100.0 * used / (s.stock.length * s.stock.width))}),
        )?;
        for p in &s.placements {
            let r = &p.rect;
            // Text sized to the part on paper: two lines, the longer about 14 characters.
            let (pw, ph) = (r.length * k, r.width * k);
            let cap = (pw / 12.0).min(ph / 4.0).clamp(0.8, 3.0);
            let [cx, cy] = at(r.x + r.length / 2.0, r.y + r.width / 2.0);
            run(
                "drawing.note.add",
                &json!({"anchor": {"sheet": sheet_id}, "at": [cx, cy], "cap_mm": cap, "align": "centre",
                        "content": format!("{}\n{} × {}", p.part, short(r.length), short(r.width))}),
            )?;
        }
    }
    Ok(json!({"sheets": count, "unplaced": nested.unplaced}))
}
