//! PLUG-0: the spike. Cabinet Studio's core inside a SolidCraft plugin, with one of each piece the
//! plan relies on (docs/PLAN.md): the ribbon, a dialog with picture choices that fills itself in
//! from the picked cabinet (a dialog hook), a docked panel fed from the design's storage, and a file
//! export. Entries not built yet answer which milestone brings them.
wit_bindgen::generate!({ path: "wit", world: "panel-dialog-plugin" });

use cabinet_core::run::{Run, RunItem, build_run};
use cabinet_core::{Cabinet, CabinetKind, PartList, geom::Vec3};
use craft::plugin::types::{Datum, FieldFailure, PluginError, Value as Cell};
use craft::plugin::{commands, files, ui};
use serde_json::{Value, json};
use solidcraft_bridge::emit::{Detail, emit_cabinet};

struct Plugin;

impl exports::craft::plugin::entry::Guest for Plugin {
    fn execute(entry: String, params: String) -> Result<String, PluginError> {
        let params: Value = serde_json::from_str(&params).map_err(|e| invalid("params", e))?;
        match entry.as_str() {
            "add" => cabinet(&params, false),
            "edit" => cabinet(&params, true),
            "show" => {
                let cabs = stored()?;
                show(&cabs)?;
                ui::show_panel("cabinets", true)?;
                Ok(json!({"cabinets": cabs.len()}).to_string())
            }
            "select" => select(&params),
            "export" => match params["make"].as_str().unwrap_or("csv") {
                "csv" => export(),
                _ => later("PLUG-4"),
            },
            "setup" | "boards" | "rebuild" => later("PLUG-1"),
            "benchtop" => later("PLUG-3"),
            _ => Err(invalid("entry", format!("no entry `{entry}`"))),
        }
    }
}

/// The ribbon is laid out ahead of what it opens: these say when they come.
fn later(milestone: &str) -> Result<String, PluginError> {
    Err(PluginError::Unsupported(format!("not built yet: it comes in {milestone} (docs/PLAN.md)")))
}

/// The calls a run made, counted as they go: the host allows 256 to an ordinary run.
static mut CALLS: u32 = 0;

fn run(command: &str, args: &Value) -> Result<Value, PluginError> {
    // SAFETY: a component instance runs on one thread.
    unsafe { CALLS += 1 };
    let reply = commands::run(command, &args.to_string())?;
    serde_json::from_str(&reply).map_err(|e| invalid(command, e))
}

fn calls() -> u32 {
    unsafe { CALLS }
}

/// The cabinets kept in the design, in run order.
fn stored() -> Result<Vec<Value>, PluginError> {
    let reply = run("storage.get", &json!({}))?;
    Ok(reply["document"]["cabinets"]
        .as_array()
        .cloned()
        .unwrap_or_default())
}

fn kind(name: &str) -> Result<CabinetKind, PluginError> {
    serde_json::from_value(json!(name)).map_err(|e| invalid("kind", e))
}

/// The cabinet a picked body belongs to: bodies are named `<cabinet>.<part>`.
fn picked(params: &Value, cabs: &[Value]) -> Option<String> {
    params["pick"].as_array()?.iter().find_map(|p| {
        let id = p["name"].as_str()?.split('.').next()?;
        cabs.iter().any(|c| c["id"] == id).then(|| id.to_string())
    })
}

fn lists(cabs: &[Value]) -> Result<Vec<PartList>, PluginError> {
    let items = cabs
        .iter()
        .map(|c| {
            let mut k = Cabinet::new(c["id"].as_str().unwrap_or("C"), kind(c["kind"].as_str().unwrap_or("base"))?);
            k.width = c["width"].as_f64().unwrap_or(600.0);
            Ok(RunItem::Cabinet(Box::new(k)))
        })
        .collect::<Result<Vec<_>, PluginError>>()?;
    build_run(&Run { origin: Vec3::default(), items }).map_err(|e| invalid("cabinet", e))
}

/// Adds a cabinet at the end of the run, or changes the picked one.
fn cabinet(p: &Value, edit: bool) -> Result<String, PluginError> {
    let detail = match p["detail"].as_str() {
        Some("manufacturing") => Detail::Manufacturing,
        _ => Detail::Design,
    };
    let mut cabs = stored()?;
    let id = match edit {
        true => picked(p, &cabs).ok_or_else(|| invalid("pick", "pick one of the cabinets: what is picked isn't one"))?,
        false => {
            // The next free number: ids stay stable when a cabinet is taken out.
            let n = (1..).find(|n| !cabs.iter().any(|c| c["id"] == format!("C{n:02}"))).unwrap_or(1);
            format!("C{n:02}")
        }
    };
    let spec = json!({"id": id, "kind": p["kind"].as_str().unwrap_or("base"), "width": p["width"].as_f64().unwrap_or(600.0)});
    match cabs.iter().position(|c| c["id"] == id) {
        Some(i) => {
            cabs[i] = spec.clone();
            run("component.delete", &json!({"component": id}))?;
        }
        None => cabs.push(spec.clone()),
    }
    let built = lists(&cabs)?;
    let list = built
        .iter()
        .find(|l| l.cabinet == id)
        .ok_or_else(|| invalid("cabinet", "not built"))?;
    let before = calls();
    let cmds = emit_cabinet(list, detail, Some(&spec));
    for c in &cmds {
        run(c["command"].as_str().unwrap_or_default(), &c["params"])?;
    }
    let building = calls() - before;
    run("storage.set", &json!({"document": {"cabinets": cabs}}))?;
    let shown = show(&cabs).is_ok() && ui::show_panel("cabinets", true).is_ok();
    run("view.fit", &json!({}))?;
    Ok(json!({"id": id, "commands": cmds.len(), "building_calls": building, "calls": calls(), "panel_updated": shown}).to_string())
}

/// A row click in the panel: that cabinet's bodies selected in the view, ready for the Cabinet button.
fn select(p: &Value) -> Result<String, PluginError> {
    let row = p["row"].as_str().unwrap_or_default().to_string();
    let reply = run("component.list", &json!({"compact": true, "bodies": true}))?;
    let mut items = Vec::new();
    collect_bodies(&reply, &row, &mut items);
    if items.is_empty() {
        return Err(invalid("row", format!("no bodies for {row}")));
    }
    run("select.set", &json!({"items": items}))?;
    Ok(json!({"selected": row, "bodies": items.len()}).to_string())
}

/// Bodies named `<row>.…` anywhere in `component.list`'s reply.
fn collect_bodies(v: &Value, row: &str, out: &mut Vec<Value>) {
    match v {
        Value::String(s) if s.split('.').next() == Some(row) && s.contains('.') => {
            if !out.iter().any(|i| i["name"] == *s) {
                out.push(json!({"type": "body", "name": s}));
            }
        }
        Value::Array(a) => a.iter().for_each(|x| collect_bodies(x, row, out)),
        Value::Object(o) => o.values().for_each(|x| collect_bodies(x, row, out)),
        _ => {}
    }
}

/// The cut list as CSV, into the folder granted as `exports`.
fn export() -> Result<String, PluginError> {
    let built = lists(&stored()?)?;
    let csv = cabinet_core::cutlist::to_csv(&cabinet_core::cutlist::cut_list(&built, "", 1.0));
    files::write_file("exports", "cut-list.csv", csv.as_bytes())?;
    Ok(json!({"bytes": csv.len(), "rows": csv.lines().count().saturating_sub(1)}).to_string())
}

/// The panel's rows: one cell per path, `/cabinets/<row>/<column>`.
fn show(cabs: &[Value]) -> Result<(), PluginError> {
    let mut cells = Vec::new();
    for (row, c) in cabs.iter().enumerate() {
        let cell = |key: &str, value: Cell| Datum { path: format!("/cabinets/{row}/{key}"), value };
        cells.push(cell("id", Cell::Text(c["id"].as_str().unwrap_or_default().into())));
        cells.push(cell("kind", Cell::Text(c["kind"].as_str().unwrap_or_default().replace('_', " "))));
        cells.push(cell("width", Cell::Number(c["width"].as_f64().unwrap_or_default())));
    }
    ui::clear_data("cabinets")?;
    ui::set_data("cabinets", &cells)
}

impl exports::craft::plugin::dialog_provider::Guest for Plugin {
    /// Edit cabinet fills itself in from the picked cabinet. Anything else picked greys the fields
    /// and says so: SolidCraft can't grey the ribbon button itself by the selection (0.50).
    fn update_fields(entry: String, values: String) -> Result<String, PluginError> {
        if entry != "edit" {
            return Ok("{}".into());
        }
        let values: Value = serde_json::from_str(&values).map_err(|e| invalid("values", e))?;
        let cabs = stored()?;
        let fields = |on: bool| json!({"enabled": on});
        let patch = match picked(&values, &cabs).and_then(|id| cabs.iter().find(|c| c["id"] == id.as_str()).cloned()) {
            Some(c) => json!({
                "pick": {"hint": format!("Changing {}", c["id"].as_str().unwrap_or_default())},
                "kind": {"value": c["kind"], "enabled": true},
                "width": {"value": c["width"], "enabled": true},
                "detail": fields(true),
            }),
            None => {
                let what = if values["pick"].as_array().is_some_and(|a| !a.is_empty()) {
                    "That isn't one of the cabinets. Pick any part of a cabinet, or click its row in the Cabinets panel"
                } else {
                    "Pick any part of a cabinet, or click its row in the Cabinets panel"
                };
                json!({"pick": {"hint": what}, "kind": fields(false), "width": fields(false), "detail": fields(false)})
            }
        };
        Ok(patch.to_string())
    }
}

impl exports::craft::plugin::ui_provider::Guest for Plugin {
    /// The same panel as `runtime.ui` in plugin.json.
    fn describe() -> ui::Declarations {
        use ui::{ButtonSpec, Column, Declarations, DockZone, Layout, Node, PanelSpec, SelectionMode, TableSpec, Widget};
        let button = |label: &str, entry: &str| ButtonSpec { label: label.into(), icon: None, entry: entry.into() };
        let column = |key: &str, label: &str, unit: Option<&str>| Column { key: key.into(), label: label.into(), unit: unit.map(Into::into) };
        let table = TableSpec {
            bind: "/cabinets".into(),
            row_id: "id".into(),
            columns: vec![column("id", "Cabinet", None), column("kind", "Type", None), column("width", "Width", Some("length"))],
            select: SelectionMode::Single,
            on_select: Some("select".into()),
            on_edit: None,
        };
        let nodes = vec![
            Node { widget: Widget::Group(vec![1, 2]) },
            Node { widget: Widget::Toolbar(vec![button("Refresh", "show"), button("Export cut list", "export")]) },
            Node { widget: Widget::Table(table) },
        ];
        let panel = PanelSpec {
            id: "cabinets".into(),
            title: "Cabinets".into(),
            zones: vec![DockZone::Right, DockZone::Bottom],
            layout: Layout { root: 0, nodes },
        };
        Declarations { ui_version: 1, panels: vec![panel], workspaces: vec![] }
    }

    fn on_event(_event: ui::UiEvent) -> Result<(), PluginError> {
        Ok(())
    }

    fn on_enter(_workspace: String) -> Result<(), PluginError> {
        Ok(())
    }

    fn on_exit(_workspace: String) -> Result<(), PluginError> {
        Ok(())
    }
}

fn invalid(field: &str, reason: impl ToString) -> PluginError {
    PluginError::InvalidInput(FieldFailure { field: field.into(), reason: reason.to_string() })
}

export!(Plugin);
