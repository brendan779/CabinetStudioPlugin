//! PLUG-0: the spike. Cabinet Studio's core inside a SolidCraft plugin, with one of each piece the
//! plan relies on (docs/PLUGIN_PLAN.md): a dialog with picture choices, a docked panel fed from the
//! design's storage, re-editing through `forms.open`, a file export and a supplier mesh. Each entry
//! reports what it measured, so the plan's open questions are answered by runs.
wit_bindgen::generate!({ path: "wit", world: "panel-plugin" });

use cabinet_core::run::{Run, RunItem, build_run};
use cabinet_core::{Cabinet, CabinetKind, geom::Vec3};
use craft::plugin::types::{Datum, FieldFailure, PluginError, Value as Cell};
use craft::plugin::{commands, files, forms, ui};
use serde_json::{Value, json};
use solidcraft_bridge::emit::{Detail, emit_cabinet};

struct Plugin;

impl exports::craft::plugin::entry::Guest for Plugin {
    fn execute(entry: String, params: String) -> Result<String, PluginError> {
        let params: Value = serde_json::from_str(&params).map_err(|e| invalid("params", e))?;
        match entry.as_str() {
            "cabinet" => cabinet(&params),
            "refresh" => {
                let cabs = stored()?;
                show(&cabs)?;
                Ok(json!({"cabinets": cabs.len()}).to_string())
            }
            "edit" => edit(&params),
            "export" => export(),
            "handle" => handle(&params),
            _ => Err(invalid("entry", format!("no entry `{entry}`"))),
        }
    }
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

/// Adds a cabinet at the end of the run, or rebuilds the one named `id`.
fn cabinet(p: &Value) -> Result<String, PluginError> {
    // `plugin.preview` (telling a preview from a run) is newer than SolidCraft 0.50.
    let preview = false;
    let detail = match p["detail"].as_str() {
        Some("manufacturing") => Detail::Manufacturing,
        _ => Detail::Design,
    };
    let mut cabs = stored()?;
    let editing = p["id"].as_str().filter(|id| !id.is_empty());
    let id = match editing {
        Some(id) => id.to_string(),
        None => format!("C{:02}", cabs.len() + 1),
    };
    let mut cab = Cabinet::new(&id, kind(p["kind"].as_str().unwrap_or("base"))?);
    cab.width = p["width"].as_f64().unwrap_or(600.0);
    let spec = json!({"id": id, "kind": p["kind"], "width": cab.width});
    match cabs.iter().position(|c| c["id"] == id) {
        Some(i) => {
            cabs[i] = spec.clone();
            // An edit replaces the cabinet's component.
            run("component.delete", &json!({"component": id}))?;
        }
        None => cabs.push(spec.clone()),
    }
    // The run places each cabinet beside the one before.
    let items = cabs
        .iter()
        .map(|c| {
            let mut k = Cabinet::new(c["id"].as_str().unwrap_or("C"), kind(c["kind"].as_str().unwrap_or("base"))?);
            k.width = c["width"].as_f64().unwrap_or(600.0);
            Ok(RunItem::Cabinet(Box::new(k)))
        })
        .collect::<Result<Vec<_>, PluginError>>()?;
    let lists = build_run(&Run { origin: Vec3::default(), items })
        .map_err(|e| invalid("cabinet", e))?;
    let list = lists
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
    // Does a dialog-run entry reach the open panel?
    let shown = show(&cabs).is_ok() && ui::show_panel("cabinets", true).is_ok();
    run("view.fit", &json!({}))?;
    Ok(json!({
        "id": id, "preview": preview, "commands": cmds.len(), "building_calls": building,
        "calls": calls(), "panel_updated": shown, "parts": list.parts.len(),
    })
    .to_string())
}

/// The panel's row action: the Cabinet dialog, filled in with that cabinet.
fn edit(p: &Value) -> Result<String, PluginError> {
    let row = p["row"].as_str().unwrap_or_default();
    let cabs = stored()?;
    let c = cabs
        .iter()
        .find(|c| c["id"] == row)
        .ok_or_else(|| invalid("row", format!("no cabinet {row}")))?;
    forms::open("cabinet", &json!({"id": c["id"], "kind": c["kind"], "width": c["width"]}).to_string())?;
    Ok(json!({"opened": row}).to_string())
}

/// The cut list as CSV, into the folder granted as `exports`.
fn export() -> Result<String, PluginError> {
    let cabs = stored()?;
    let items = cabs
        .iter()
        .map(|c| {
            let mut k = Cabinet::new(c["id"].as_str().unwrap_or("C"), kind(c["kind"].as_str().unwrap_or("base"))?);
            k.width = c["width"].as_f64().unwrap_or(600.0);
            Ok(RunItem::Cabinet(Box::new(k)))
        })
        .collect::<Result<Vec<_>, PluginError>>()?;
    let lists = build_run(&Run { origin: Vec3::default(), items }).map_err(|e| invalid("export", e))?;
    let csv = cabinet_core::cutlist::to_csv(&cabinet_core::cutlist::cut_list(&lists, "", 1.0));
    files::write_file("exports", "cut-list.csv", csv.as_bytes())?;
    Ok(json!({"bytes": csv.len(), "rows": csv.lines().count() - 1}).to_string())
}

/// A supplier mesh from the folder granted as `vendor`.
fn handle(p: &Value) -> Result<String, PluginError> {
    let file = p["file"].as_str().unwrap_or("abi/ABI-TEZRA-220.stl");
    let reply = run("file.insert_mesh", &json!({"path": format!("grant:vendor/{file}")}))?;
    Ok(json!({"inserted": reply}).to_string())
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
            on_select: None,
            on_edit: Some("edit".into()),
        };
        let nodes = vec![
            Node { widget: Widget::Group(vec![1, 2]) },
            Node { widget: Widget::Toolbar(vec![button("Refresh", "refresh"), button("Export cut list", "export")]) },
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
