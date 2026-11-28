//! SolidCraft sheets for the cutting layouts, job cut list and individual model bodies.
use cabinet_core::cutlist::{CutList, CutRow};
use cabinet_core::nesting::Nesting;
use serde_json::{Value, json};

use crate::{PluginError, run};

/// The drawing's name: a run replaces the one it made before.
pub const NAME: &str = "Cutting layouts";

/// Where the table goes on A3 landscape (420 × 297), in paper mm: inside the border, above the
/// title block (180 × 61 at the lower right), with a line for the heading above it.
const AREA: (f64, f64, f64, f64) = (24.0, 78.0, 384.0, 190.0);
const HEADING_Y: f64 = 276.0;
/// A note's line spacing, as a multiple of its capital height (measured on 0.51's notes).
const LINE: f64 = 1.7;

fn short(x: f64) -> String {
    format!("{x:.0}")
}

/// Keep the drawing id across refreshes. Add the new sheets before removing the old ones
/// because SolidCraft requires a drawing to have at least one sheet.
struct Drawing {
    old: Vec<String>,
    first: Option<String>,
}

impl Drawing {
    fn open(name: &str, first_title: &str) -> Result<Self, PluginError> {
        let listed = run("drawing.list", &json!({}))?;
        let existing = listed["drawings"].as_array().into_iter().flatten().find(|d| d["name"] == name);
        if let Some(d) = existing {
            let opened = run("drawing.open", &json!({"drawing": d["id"]}))?;
            let old =
                opened["sheets"].as_array().into_iter().flatten().filter_map(|s| s["id"].as_str().map(String::from)).collect();
            Ok(Self { old, first: None })
        } else {
            let made = run("drawing.create", &json!({"name": name, "template": "iso-a3-part", "first_sheet_name": first_title}))?;
            Ok(Self { old: vec![], first: Some(sheet_id(&made["sheets"][0]["id"])?) })
        }
    }

    fn sheet(&mut self, name: &str) -> Result<String, PluginError> {
        if crate::craft::plugin::control::cancelled() {
            return Err(PluginError::Cancelled);
        }
        if let Some(first) = self.first.take() {
            return Ok(first);
        }
        sheet_id(&run("drawing.sheet.add", &json!({"name": name}))?["sheet"])
    }

    fn finish(self) -> Result<(), PluginError> {
        for id in self.old {
            run("drawing.sheet.delete", &json!({"sheet": id, "contents": "delete"}))?;
        }
        Ok(())
    }
}

fn sheet_id(id: &Value) -> Result<String, PluginError> {
    id.as_str().map(String::from).ok_or_else(|| crate::invalid("sheet", "SolidCraft returned no sheet id"))
}

fn edges(row: &CutRow) -> String {
    let names = ["L1", "L2", "W1", "W2"]
        .into_iter()
        .zip(row.edges)
        .filter_map(|(name, on)| on.then_some(name))
        .collect::<Vec<_>>()
        .join(" ");
    if names.is_empty() { "—".into() } else { names }
}

fn note(sheet: &str, x: f64, y: f64, cap: f64, text: &str) -> Result<(), PluginError> {
    run("drawing.note.add", &json!({"anchor":{"sheet":sheet}, "at":[x,y], "cap_mm":cap, "content":text}))?;
    Ok(())
}

pub fn layouts(nested: &Nesting, cut: &CutList) -> Result<Value, PluginError> {
    let count = nested.sheets.len();
    let title = |i: usize| format!("{} {}", i + 1, crate::costs::board_label(&nested.sheets[i].board));
    let mut drawing = Drawing::open(NAME, &if count > 0 { title(0) } else { "Cut list 1".into() })?;
    for (i, s) in nested.sheets.iter().enumerate() {
        let sheet_id = drawing.sheet(&title(i))?;
        let board = format!("{}, {} mm", crate::costs::board_label(&s.board), s.thickness);
        run(
            "drawing.sheet.set",
            &json!({"sheet": sheet_id, "name": title(i), "title_block_overrides": {
                "title": NAME, "material": board, "document_type": "Cutting layout",
                "general_tolerances": "Sizes are cut sizes, in mm"}}),
        )?;
        let note = |at: [f64; 2], cap: f64, align: &str, content: String| {
            run(
                "drawing.note.add",
                &json!({"anchor": {"sheet": sheet_id}, "at": at, "cap_mm": cap, "align": align, "content": content}),
            )
        };
        let (x0, y0, _, _) = AREA;
        let used: f64 = s.placements.iter().map(|p| p.rect.length * p.rect.width).sum();
        note(
            [x0, HEADING_Y],
            3.5,
            "left",
            format!(
                "{board}: sheet {} of {count}, {} × {}, {} parts, {:.0} % used",
                i + 1,
                short(s.stock.length),
                short(s.stock.width),
                s.placements.len(),
                100.0 * used / (s.stock.length * s.stock.width)
            ),
        )?;
        // A table of the sheet's parts: a note a column, their lines in step.
        // A note's lines are centred on its place: each column's place is the table's middle.
        let lines = s.placements.len() as f64 + 1.0;
        let top = HEADING_Y - 8.0;
        let cap = ((top - y0) / lines / LINE).clamp(1.0, 3.5);
        let top = top - lines * cap * LINE / 2.0;
        let column =
            |head: &str, cells: Vec<String>| std::iter::once(head.to_string()).chain(cells).collect::<Vec<_>>().join("\n");
        note([x0, top], cap, "left", column("Part", s.placements.iter().map(|p| p.part.clone()).collect()))?;
        note([x0 + 200.0, top], cap, "right", column("Length", s.placements.iter().map(|p| short(p.rect.length)).collect()))?;
        note([x0 + 240.0, top], cap, "right", column("Width", s.placements.iter().map(|p| short(p.rect.width)).collect()))?;
        note(
            [x0 + 290.0, top],
            cap,
            "right",
            column("At (x, y)", s.placements.iter().map(|p| format!("{}, {}", short(p.rect.x), short(p.rect.y))).collect()),
        )?;
    }
    let mut pages = 0;
    // Keep rows for the same stock together and repeat the board and headers on continuation sheets.
    let mut rows: Vec<_> = cut.rows.iter().collect();
    rows.sort_by(|a, b| a.board.cmp(&b.board).then(a.thickness.total_cmp(&b.thickness)));
    for rows in rows.chunk_by(|a, b| a.board == b.board && a.thickness == b.thickness) {
        for chunk in rows.chunks(32) {
            pages += 1;
            let name = format!("Cut list {pages}");
            let sheet = drawing.sheet(&name)?;
            let board = crate::costs::board_label(&chunk[0].board);
            let material = format!("{board}, {} mm", chunk[0].thickness);
            run(
                "drawing.sheet.set",
                &json!({"sheet":sheet,"title_block_overrides":{"title":name,"material":material,"document_type":"Job cut list"}}),
            )?;
            note(&sheet, AREA.0, HEADING_Y, 3.5, &material)?;
            let mut columns = ["Part", "Cut L (mm)", "Cut W (mm)", "Qty", "Edges", "Supply"].map(String::from);
            for row in chunk {
                let cells = [
                    row.part.clone(),
                    format!("{:.1}", row.cut_length),
                    format!("{:.1}", row.cut_width),
                    row.quantity.to_string(),
                    edges(row),
                    if row.templated { "Template" } else { "Stock" }.into(),
                ];
                for (col, cell) in columns.iter_mut().zip(cells) {
                    col.push('\n');
                    col.push_str(&cell);
                }
            }
            let y = HEADING_Y - 9.0 - (chunk.len() + 1) as f64 * 3.0 * LINE / 2.0;
            for (x, col) in [24.0, 152.0, 207.0, 262.0, 289.0, 358.0].into_iter().zip(columns) {
                note(&sheet, x, y, 3.0, &col)?;
            }
        }
    }
    if pages == 0 {
        let sheet = drawing.sheet("Cut list 1")?;
        note(&sheet, AREA.0, HEADING_Y, 3.5, "No parts in the job")?;
        pages = 1;
    }
    drawing.finish()?;
    Ok(json!({"sheets": count, "cut_list_pages":pages, "unplaced": nested.unplaced}))
}

pub fn parts(cut: &CutList) -> Result<Value, PluginError> {
    let components = run("component.list", &json!({"compact":true,"bodies":true}))?;
    let mut drawing = Drawing::open("Part drawings", cut.rows.first().map_or("Parts", |r| &r.part))?;
    let mut views = 0;
    let mut missing = Vec::new();
    for row in &cut.rows {
        let sheet = drawing.sheet(&row.part)?;
        let material = format!("{}, {} mm", crate::costs::board_label(&row.board), row.thickness);
        run(
            "drawing.sheet.set",
            &json!({"sheet":sheet, "title_block_overrides":{"title":row.part,"material":material,"document_type":"Part drawing"}}),
        )?;
        let band = edges(row);
        note(
            &sheet,
            AREA.0,
            HEADING_Y,
            3.5,
            &format!(
                "{}    Cut {:.1} × {:.1} mm    Finished {:.1} × {:.1} mm    Qty {}    Edges: {}",
                row.part, row.cut_length, row.cut_width, row.length, row.width, row.quantity, band
            ),
        )?;
        let component = components["components"].as_array().into_iter().flatten().find(|c| c["name"] == row.cabinet);
        if let Some(c) = component {
            let occurrence = c["occurrences"].as_array().into_iter().flatten().find(|o| o["parent"] == 0);
            let bodies = c["bodies"].as_array().into_iter().flatten();
            if let Some(path) = occurrence.and_then(|o| o["id"].as_u64()).filter(|_| bodies.clone().any(|b| b == &row.part)) {
                let hide: Vec<_> = bodies.filter(|b| b.as_str() != Some(row.part.as_str())).map(|body| json!({"path":[path],"body":body})).collect();
                run(
                    "drawing.view.base",
                    &json!({"sheet":sheet,"source":{"occurrences":[[path]]},"hide_bodies":hide,"view":"iso"}),
                )?;
                views += 1;
                continue;
            }
        }
        note(&sheet, AREA.0, 248.0, 3.5, "No matching model body at the current detail level")?;
        missing.push(&row.part);
    }
    if cut.rows.is_empty() {
        let sheet = drawing.sheet("Parts")?;
        note(&sheet, AREA.0, HEADING_Y, 3.5, "No parts in the job")?;
    }
    drawing.finish()?;
    Ok(json!({"views":views, "missing_bodies":missing}))
}
