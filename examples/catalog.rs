//! Writes what `scripts/make_manifest.py` builds the manifest and the dialogs' pictures from, so
//! they follow Cabinet Studio's core: its construction option groups, the cabinet types and front
//! presets, and each one's front view as the core builds it (rectangles in millimetres).
//!
//!     cargo run -q --example catalog --target aarch64-apple-darwin > target/catalog.json
#[path = "../src/spec.rs"]
#[allow(dead_code)]
mod spec;

use cabinet_core::standins::stand_ins;
use cabinet_core::{Cabinet, CabinetKind, PartList, build_cabinet};
use serde_json::{Value, json};

/// A cabinet seen from the front: its carcass outline, its fronts and its handles, in X and Z.
fn front_view(list: &PartList) -> Value {
    let rect = |b: cabinet_core::geom::Aabb| json!([b.min.x, b.min.z, b.max().x, b.max().z]);
    // Fronts stand in front of the carcass (Y below 0).
    let fronts: Vec<Value> = list.parts.iter().filter(|p| p.bounds.min.y < -1.0).map(|p| rect(p.bounds)).collect();
    let carcass: Vec<Value> = list.parts.iter().filter(|p| p.bounds.min.y >= -1.0).map(|p| rect(p.bounds)).collect();
    let handles: Vec<Value> = stand_ins(list)
        .iter()
        .filter(|s| s.name.contains(".Handle") || s.name.contains(".Leg"))
        .map(|s| rect(s.bounds()))
        .collect();
    json!({"carcass": carcass, "fronts": fronts, "hardware": handles})
}

/// The board index the plugin scans (src/boards.tsv): one board a line, tab-separated, in
/// catalogue order. Reading the whole catalogue on every panel change was too slow (owner).
fn board_index() -> String {
    let cat = cabinet_catalog::embedded::catalog().expect("the built-in catalogue");
    let clean = |s: &str| s.replace(['\t', '\n'], " ");
    let mut out = String::new();
    for b in &cat.boards {
        let top = matches!(b.kind, cabinet_catalog::BoardKind::LaminateTop | cabinet_catalog::BoardKind::SinteredStone);
        let role = if top { "top" } else if b.kind == cabinet_catalog::BoardKind::Lining { "lining" } else { "sheet" };
        let sheet = b
            .sheets
            .iter()
            .max_by(|x, y| (x.length * x.width).total_cmp(&(y.length * y.width)))
            .map_or(String::new(), |s| format!("{:.0} × {:.0}", s.length, s.width));
        out.push_str(&[b.id.clone(), clean(&b.brand), clean(&b.range), clean(&b.decor), clean(&b.finish),
            format!("{}", b.thickness), sheet, b.preview_colour.clone(), role.into()].join("\t"));
        out.push('\n');
    }
    out
}

fn main() {
    if std::env::args().any(|a| a == "--boards") {
        print!("{}", board_index());
        return;
    }
    let job = spec::Job::from_stored(&json!({}));
    let mut kinds = Vec::new();
    for (id, label) in spec::KINDS {
        let kind = spec::kind(id).expect("a core kind");
        let cab = spec::cabinet(&json!({"id": "P", "kind": id}), &job).expect("a cabinet");
        kinds.push(json!({
            "id": id, "label": label, "class": spec::size_class(kind),
            "presets": spec::presets_for(kind), "view": front_view(&build_cabinet(&cab).expect("built")),
        }));
    }
    // Each preset on the type it suits best, at a width that shows it.
    let mut presets = Vec::new();
    for (id, label) in spec::PRESETS {
        let on = spec::KINDS
            .iter()
            .map(|(k, _)| *k)
            .find(|k| spec::presets_for(spec::kind(k).unwrap()).contains(id))
            .unwrap_or("base");
        let cab: Cabinet = spec::cabinet(&json!({"id": "P", "kind": on, "fronts": id}), &job).expect("a cabinet");
        presets.push(json!({"id": id, "label": label, "view": front_view(&build_cabinet(&cab).expect("built"))}));
    }
    let groups = serde_json::to_value(cabinet_core::options::option_groups()).unwrap();
    let measurements = serde_json::to_value(cabinet_core::options::measurements()).unwrap();
    let defaults = serde_json::to_value(cabinet_core::ConstructionStandard::default()).unwrap();
    let sizes: Value = ["base", "wall", "tall"]
        .iter()
        .map(|c| (c.to_string(), job.sizes[*c].clone()))
        .collect::<serde_json::Map<_, _>>()
        .into();
    let _ = CabinetKind::ALL;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "kinds": kinds, "presets": presets, "opens": spec::OPENS.iter().map(|(i, l)| json!({"id": i, "label": l})).collect::<Vec<_>>(),
            "groups": groups, "measurements": measurements, "defaults": defaults, "sizes": sizes,
        }))
        .unwrap()
    );
}
