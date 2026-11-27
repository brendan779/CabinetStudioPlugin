//! Cabinet Studio as a SolidCraft plugin (docs/PLAN.md). The design keeps the job's setup and each
//! cabinet's choices (`spec`); every change rebuilds, through Cabinet Studio's core, only the
//! cabinets whose SolidCraft commands changed.
wit_bindgen::generate!({ path: "wit", world: "panel-dialog-plugin" });

mod boards;
mod spec;

use cabinet_core::run::{Run, RunItem, build_run};
use cabinet_core::{PartList, geom::Vec3};
use craft::plugin::types::{Datum, FieldFailure, PluginError, Value as Cell};
use craft::plugin::{commands, control, files, ui, user_storage};
use serde_json::{Value, json};
use solidcraft_bridge::emit::{Detail, emit_cabinet};
use spec::Job;

struct Plugin;

impl exports::craft::plugin::entry::Guest for Plugin {
    fn execute(entry: String, params: String) -> Result<String, PluginError> {
        let params: Value = serde_json::from_str(&params).map_err(|e| invalid("params", e))?;
        match entry.as_str() {
            "setup" => setup(&params),
            "add" => add(&params),
            "edit" => edit(&params),
            "rebuild" => {
                let (job, cabs) = load()?;
                sync(&job, cabs, true)
            }
            "show" => {
                let (job, cabs) = load()?;
                show(&job, &cabs)?;
                ui::show_panel("cabinets", true)?;
                Ok(json!({"cabinets": cabs.len()}).to_string())
            }
            "select" => select(&params),
            "export" => match params["make"].as_str().unwrap_or("csv") {
                "csv" => export(),
                _ => later("PLUG-4"),
            },
            "board-filter" => board_filter(&params),
            "board-pick" => board_pick(&params),
            "benchtop" => later("PLUG-3"),
            _ => Err(invalid("entry", format!("no entry `{entry}`"))),
        }
    }
}

/// The ribbon is laid out ahead of what it opens: these say when they come.
fn later(milestone: &str) -> Result<String, PluginError> {
    Err(PluginError::Unsupported(format!("not built yet: it comes in {milestone} (docs/PLAN.md)")))
}

fn run(command: &str, args: &Value) -> Result<Value, PluginError> {
    let reply = commands::run(command, &args.to_string())?;
    serde_json::from_str(&reply).map_err(|e| invalid(command, e))
}

// ---- What the design keeps ----

/// The design's storage: `{"job": {...}, "cabinets": [...]}`.
fn document() -> Result<Value, PluginError> {
    Ok(run("storage.get", &json!({}))?["document"].clone())
}

/// The job's setup as stored, or the shop standard for a job that has none yet.
fn stored_job(doc: &Value) -> Value {
    if doc["job"].is_object() {
        return doc["job"].clone();
    }
    user_storage::get("shop")
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(json!({}))
}

fn load() -> Result<(Value, Vec<Value>), PluginError> {
    let doc = document()?;
    Ok((stored_job(&doc), doc["cabinets"].as_array().cloned().unwrap_or_default()))
}

fn save(job: &Value, cabs: &[Value]) -> Result<(), PluginError> {
    run("storage.set", &json!({"document": {"job": job, "cabinets": cabs}}))?;
    Ok(())
}

// ---- Building ----

fn lists(job: &Job, cabs: &[Value]) -> Result<Vec<PartList>, PluginError> {
    let items = cabs
        .iter()
        .map(|c| {
            spec::cabinet(c, job)
                .map(|k| RunItem::Cabinet(Box::new(k)))
                .ok_or_else(|| invalid("cabinet", format!("can't read {c}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    build_run(&Run { origin: Vec3::default(), items }).map_err(|e| invalid("cabinet", e))
}

/// A short fingerprint of a cabinet's commands: equal fingerprints build the same bodies.
fn fingerprint(cmds: &[Value]) -> String {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    serde_json::to_string(cmds).unwrap_or_default().hash(&mut h);
    format!("{:016x}", h.finish())
}

/// Brings the design's cabinets to `cabs`: each cabinet whose commands changed is taken out and
/// built again, a cabinet no longer listed is taken out, and the rest are left alone. `all`
/// rebuilds every one. Runs as a long job, with progress per cabinet.
fn sync(job_value: &Value, mut cabs: Vec<Value>, all: bool) -> Result<String, PluginError> {
    let job = Job::from_stored(job_value);
    let detail = if job.detail == "manufacturing" { Detail::Manufacturing } else { Detail::Design };
    let built = lists(&job, &cabs)?;
    // The chosen boards' own colours on the parts made of them.
    let colours: std::collections::BTreeMap<String, String> = job_value["boards"]
        .as_object()
        .map(|o| {
            o.values()
                .filter_map(|b| Some((b["id"].as_str()?.to_string(), b["colour"].as_str()?.to_string())))
                .filter(|(_, c)| c.starts_with('#'))
                .collect()
        })
        .unwrap_or_default();
    let before: Vec<Value> = document()?["cabinets"].as_array().cloned().unwrap_or_default();
    // Gone from the job: their bodies go too.
    for old in &before {
        if old["built"].is_string() && !cabs.iter().any(|c| c["id"] == old["id"]) {
            let _ = run("component.delete", &json!({"component": old["id"]}));
        }
    }
    let mut rebuilt = Vec::new();
    let n = cabs.len().max(1) as f32;
    for (i, c) in cabs.iter_mut().enumerate() {
        let id = c["id"].as_str().unwrap_or_default().to_string();
        let Some(list) = built.iter().find(|l| l.cabinet == id) else { continue };
        // The fingerprint covers what the cabinet's commands do, not the bookkeeping in `c`.
        let mut plain = c.clone();
        if let Some(o) = plain.as_object_mut() {
            o.remove("built");
        }
        let mut cmds = emit_cabinet(list, detail, Some(&plain));
        cmds.extend(solidcraft_bridge::emit::board_colours(list, &colours));
        let print = fingerprint(&cmds);
        let was = before.iter().find(|b| b["id"] == id.as_str()).and_then(|b| b["built"].as_str());
        if !all && was == Some(print.as_str()) {
            c["built"] = json!(print);
            continue;
        }
        control::progress(i as f32 / n, &format!("Building {id}"));
        if control::cancelled() {
            return Err(PluginError::Cancelled);
        }
        if was.is_some() {
            let _ = run("component.delete", &json!({"component": id}));
        }
        for cmd in &cmds {
            run(cmd["command"].as_str().unwrap_or_default(), &cmd["params"])?;
        }
        c["built"] = json!(print);
        rebuilt.push(id);
    }
    save(job_value, &cabs)?;
    let _ = show(job_value, &cabs).and_then(|_| ui::show_panel("cabinets", true));
    if !rebuilt.is_empty() {
        run("view.fit", &json!({}))?;
    }
    let warnings: Vec<String> = built.iter().flat_map(|l| l.warnings.clone()).collect();
    Ok(json!({"rebuilt": rebuilt, "cabinets": cabs.len(), "warnings": warnings}).to_string())
}

// ---- Entries ----

/// A cabinet's choices from a dialog's values.
fn spec_from(p: &Value, id: &str) -> Value {
    let mut s = json!({"id": id});
    for k in ["kind", "width", "height", "depth", "fronts", "hinge", "shelves", "pipe_cutout", "liner",
              "blind_side", "door_style", "opens"] {
        if !p[k].is_null() {
            s[k] = p[k].clone();
        }
    }
    s
}

/// The cabinet a picked body belongs to: bodies are named `<cabinet>.<part>`.
fn picked(pick: &Value, cabs: &[Value]) -> Option<usize> {
    pick.as_array()?.iter().find_map(|p| {
        let id = p["name"].as_str()?.split('.').next()?;
        cabs.iter().position(|c| c["id"] == id)
    })
}

fn next_id(cabs: &[Value]) -> String {
    let n = (1..).find(|n| !cabs.iter().any(|c| c["id"] == format!("C{n:02}"))).unwrap_or(1);
    format!("C{n:02}")
}

fn add(p: &Value) -> Result<String, PluginError> {
    let (job, mut cabs) = load()?;
    let id = next_id(&cabs);
    let at = picked(&p["beside"], &cabs).map_or(cabs.len(), |i| i + 1);
    cabs.insert(at, spec_from(p, &id));
    sync(&job, cabs, false)
}

fn edit(p: &Value) -> Result<String, PluginError> {
    let (job, mut cabs) = load()?;
    let i = picked(&p["pick"], &cabs).ok_or_else(|| invalid("pick", "pick one of the cabinets: what is picked isn't one"))?;
    if p["remove"].as_bool() == Some(true) {
        cabs.remove(i);
    } else {
        let id = cabs[i]["id"].as_str().unwrap_or_default().to_string();
        let built = cabs[i]["built"].clone();
        cabs[i] = spec_from(p, &id);
        cabs[i]["built"] = built;
    }
    sync(&job, cabs, false)
}

/// The setup fields that are the core's standard under the same name.
const STANDARD_FIELDS: &[&str] = &[
    "door_style", "overlay", "handle_finish", "sharknose_angle", "sharknose_door_edge", "front_grain",
    "drawer_system", "drawer_sides", "drawer_frame", "hinge_boss", "top_joint", "bottom_joint", "back",
    "fixings", "biscuit", "dowel", "dowel_spacing", "domino", "fixing_density", "carcass_thickness",
    "toe_kick", "leg_fixing", "wall_hanging",
];

/// The setup's fields of the standard: its pickers, and the core's size rows under them.
fn standard_fields() -> Vec<String> {
    let mut f: Vec<String> = STANDARD_FIELDS.iter().map(|s| s.to_string()).collect();
    f.extend(cabinet_core::options::measurements().into_iter().map(|m| m.field));
    f
}

/// The stored job from the Job setup dialog's values.
fn job_from(p: &Value) -> Value {
    let mut standard = json!({});
    for f in standard_fields() {
        if !p[&f].is_null() {
            standard[&f] = p[&f].clone();
        }
    }
    if let Some(size) = p["handle_size"].as_str() {
        standard["handle_size"] = json!(size);
    }
    spec::set_opens(&mut standard, p["opens"].as_str().unwrap_or("handle"), p["handle_position"].as_str());
    let size = |c: &str| json!([p[format!("{c}_height")], p[format!("{c}_depth")]]);
    json!({
        "standard": standard,
        "sizes": {"base": size("base"), "wall": size("wall"), "tall": size("tall")},
        "detail": p["detail"].as_str().unwrap_or("design"),
    })
}

fn setup(p: &Value) -> Result<String, PluginError> {
    let job = job_from(p);
    if p["save_shop"].as_bool() == Some(true) {
        user_storage::set("shop", &job.to_string())?;
    }
    let (_, cabs) = load()?;
    sync(&job, cabs, false)
}

/// A row click in the panel: that cabinet's bodies selected in the view, ready for Edit.
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
    let (job, cabs) = load()?;
    let built = lists(&Job::from_stored(&job), &cabs)?;
    let csv = cabinet_core::cutlist::to_csv(&cabinet_core::cutlist::cut_list(&built, "", 1.0));
    files::write_file("exports", "cut-list.csv", csv.as_bytes())?;
    Ok(json!({"bytes": csv.len(), "rows": csv.lines().count().saturating_sub(1)}).to_string())
}

// ---- Boards (PLUG-2) ----

/// The Boards tab's "Use for" and search, kept on the Mac between runs.
fn board_filter_stored() -> Value {
    user_storage::get("board_filter").ok().flatten().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(json!({}))
}

/// A change to "Use for" or the search: the event's path names the field, its value the text.
fn board_filter(p: &Value) -> Result<String, PluginError> {
    let mut filter = board_filter_stored();
    let value = p["value"]["text"].as_str().or(p["value"].as_str()).unwrap_or_default();
    match p["path"].as_str().unwrap_or_default() {
        "/filter/role" => filter["role"] = json!(boards::role_of(value)),
        "/filter/search" => filter["search"] = json!(value),
        _ => {}
    }
    user_storage::set("board_filter", &filter.to_string())?;
    let (job, cabs) = load()?;
    show(&job, &cabs)?;
    Ok(filter.to_string())
}

/// A row picked in the Boards table: that board for the role chosen, and the cabinets rebuilt.
fn board_pick(p: &Value) -> Result<String, PluginError> {
    let id = p["row"].as_str().unwrap_or_default();
    let Some(board) = boards::find(id) else {
        return Ok(json!({"picked": null}).to_string());
    };
    let role = board_filter_stored()["role"].as_str().unwrap_or("carcass").to_string();
    let (mut job, cabs) = load()?;
    boards::assign(&mut job, &role, board);
    sync(&job, cabs, false)
}

// ---- The Cabinets panel ----

fn show(job_value: &Value, cabs: &[Value]) -> Result<(), PluginError> {
    let job = Job::from_stored(job_value);
    let built = lists(&job, cabs).unwrap_or_default();
    let mut cells = Vec::new();
    for (row, c) in cabs.iter().enumerate() {
        let cell = |key: &str, value: Cell| Datum { path: format!("/cabinets/{row}/{key}"), value };
        let id = c["id"].as_str().unwrap_or_default();
        let kind = spec::kind(c["kind"].as_str().unwrap_or("base"));
        let label = spec::KINDS.iter().find(|(k, _)| Some(*k) == c["kind"].as_str()).map_or("", |(_, l)| *l);
        let size = spec::cabinet(c, &job).map_or(String::new(), |k| format!("{:.0} × {:.0} × {:.0}", k.width, k.height, k.depth));
        let warnings = built.iter().find(|l| l.cabinet == id).map_or(0, |l| l.warnings.len());
        cells.push(cell("id", Cell::Text(id.into())));
        cells.push(cell("kind", Cell::Text(label.into())));
        cells.push(cell("size", Cell::Text(size)));
        cells.push(cell("fronts", Cell::Text(kind.map_or(String::new(), |k| spec::fronts_label(c, k)))));
        cells.push(cell("warnings", Cell::Text(if warnings == 0 { String::new() } else { warnings.to_string() })));
    }
    let mut n = 0;
    for l in &built {
        for w in &l.warnings {
            cells.push(Datum { path: format!("/warnings/{n}"), value: Cell::Text(format!("{}: {w}", l.cabinet)) });
            n += 1;
        }
    }
    // The Boards tab: the job's boards, the filter, and the catalogue's matching boards.
    for (row, (role, label)) in boards::ROLES.iter().enumerate() {
        let name = job_value["boards"][*role]["name"].as_str().unwrap_or("Not chosen").to_string();
        cells.push(Datum { path: format!("/job_boards/{row}/role"), value: Cell::Text((*label).into()) });
        cells.push(Datum { path: format!("/job_boards/{row}/board"), value: Cell::Text(name) });
    }
    let filter = board_filter_stored();
    let role = filter["role"].as_str().unwrap_or("carcass");
    let search = filter["search"].as_str().unwrap_or("");
    cells.push(Datum { path: "/filter/role".into(), value: Cell::Text(boards::label_of(role).into()) });
    cells.push(Datum { path: "/filter/search".into(), value: Cell::Text(search.into()) });
    let (rows, total) = boards::matching(role, search);
    for (row, b) in rows.iter().enumerate() {
        let cell = |key: &str, value: Cell| Datum { path: format!("/boards/{row}/{key}"), value };
        cells.push(cell("id", Cell::Text(b.id.clone())));
        cells.push(cell("brand", Cell::Text(b.brand.clone())));
        cells.push(cell("range", Cell::Text(b.range.clone())));
        cells.push(cell("decor", Cell::Text(b.decor.clone())));
        cells.push(cell("finish", Cell::Text(b.finish.clone())));
        cells.push(cell("thickness", Cell::Number(b.thickness)));
        cells.push(cell("sheet", Cell::Text(boards::sheet(b))));
    }
    if total > rows.len() {
        let row = rows.len();
        cells.push(Datum { path: format!("/boards/{row}/id"), value: Cell::Text("more".into()) });
        cells.push(Datum {
            path: format!("/boards/{row}/decor"),
            value: Cell::Text(format!("{} more: type in the search to narrow them", total - row)),
        });
    }
    ui::clear_data("cabinets")?;
    ui::set_data("cabinets", &cells)
}

impl exports::craft::plugin::ui_provider::Guest for Plugin {
    /// The same panel as `runtime.ui` in plugin.json.
    fn describe() -> ui::Declarations {
        use ui::{ButtonSpec, Column, Declarations, DockZone, Field, FieldKind, FormSpec, Layout, ListSpec, Node, PanelSpec, SelectionMode, Tab, TableSpec, Widget};
        let button = |label: &str, entry: &str| ButtonSpec { label: label.into(), icon: None, entry: entry.into() };
        let column = |key: &str, label: &str| Column { key: key.into(), label: label.into(), unit: None };
        let table = TableSpec {
            bind: "/cabinets".into(),
            row_id: "id".into(),
            columns: vec![column("id", "Cabinet"), column("kind", "Type"), column("size", "W × H × D"),
                          column("fronts", "Fronts"), column("warnings", "Warnings")],
            select: SelectionMode::Single,
            on_select: Some("select".into()),
            on_edit: None,
        };
        let boards = TableSpec {
            bind: "/boards".into(),
            row_id: "id".into(),
            columns: vec![column("brand", "Brand"), column("range", "Range"), column("decor", "Decor"), column("finish", "Finish"),
                          Column { key: "thickness".into(), label: "Thick".into(), unit: Some("length".into()) },
                          column("sheet", "Sheet")],
            select: SelectionMode::Single,
            on_select: Some("board-pick".into()),
            on_edit: None,
        };
        let job_boards = TableSpec {
            bind: "/job_boards".into(),
            row_id: "role".into(),
            columns: vec![column("role", "Role"), column("board", "Board")],
            select: SelectionMode::None,
            on_select: None,
            on_edit: None,
        };
        let filter = FormSpec {
            bind: "/filter".into(),
            fields: vec![
                Field { key: "role".into(), label: "Use for".into(), kind: FieldKind::Choice,
                        choices: boards::ROLES.iter().map(|(_, l)| (*l).to_string()).collect(), on_change: Some("board-filter".into()) },
                Field { key: "search".into(), label: "Search".into(), kind: FieldKind::Text, choices: vec![], on_change: Some("board-filter".into()) },
            ],
        };
        let nodes = vec![
            Node { widget: Widget::Tabs(vec![Tab { label: "Cabinets".into(), child: 1 }, Tab { label: "Boards".into(), child: 6 }]) },
            Node { widget: Widget::Group(vec![2, 3, 4, 5]) },
            Node { widget: Widget::Toolbar(vec![button("Refresh", "show"), button("Rebuild all", "rebuild")]) },
            Node { widget: Widget::Table(table) },
            Node { widget: Widget::Text("Warnings".into()) },
            Node { widget: Widget::ItemList(ListSpec { bind: "/warnings".into(), on_select: None }) },
            Node { widget: Widget::Group(vec![7, 8, 9]) },
            Node { widget: Widget::Table(job_boards) },
            Node { widget: Widget::Form(filter) },
            Node { widget: Widget::Table(boards) },
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

// ---- The dialogs' hooks ----

impl exports::craft::plugin::dialog_provider::Guest for Plugin {
    fn update_fields(entry: String, values: String) -> Result<String, PluginError> {
        let values: Value = serde_json::from_str(&values).map_err(|e| invalid("values", e))?;
        let patch = match entry.as_str() {
            "setup" => setup_fields(&values)?,
            "add" => cabinet_fields(&values, None)?,
            "edit" => {
                let (_, cabs) = load()?;
                match picked(&values["pick"], &cabs) {
                    Some(i) => cabinet_fields(&values, Some(&cabs[i]))?,
                    None => {
                        let what = if values["pick"].as_array().is_some_and(|a| !a.is_empty()) {
                            "That isn't one of the cabinets. Pick any part of a cabinet, or click its row in the Cabinets panel"
                        } else {
                            "Pick any part of a cabinet, or click its row in the Cabinets panel"
                        };
                        let mut p = json!({"pick": {"hint": what}});
                        for f in ["kind", "width", "height", "depth", "fronts", "shelves", "door_style", "opens", "remove"] {
                            p[f] = json!({"enabled": false});
                        }
                        p
                    }
                }
            }
            _ => json!({}),
        };
        Ok(patch.to_string())
    }
}

/// Job setup: the stored setup's values, and only the fields that apply (the core's `applies`).
fn setup_fields(values: &Value) -> Result<Value, PluginError> {
    let doc = document()?;
    let stored = stored_job(&doc);
    let job = Job::from_stored(&stored);
    let standard = serde_json::to_value(&job.standard).unwrap_or(Value::Null);
    let mut patch = json!({});
    for f in standard_fields() {
        if !standard[&f].is_null() {
            patch[&f] = json!({"value": standard[&f]});
        }
    }
    patch["opens"] = json!({"value": spec::opens_of(&standard)});
    if standard["handle_size"] != "knob" {
        patch["handle_size"] = json!({"value": standard["handle_size"]});
    }
    if matches!(standard["handle_place"].as_str(), Some("corner" | "centre")) {
        patch["handle_position"] = json!({"value": standard["handle_place"]});
    }
    for c in ["base", "wall", "tall"] {
        let s = &job.sizes[c];
        patch[format!("{c}_height")] = json!({"value": s[0]});
        patch[format!("{c}_depth")] = json!({"value": s[1]});
    }
    patch["detail"] = json!({"value": job.detail});
    // What applies follows the dialog's current values, through the core's own rules.
    let mut now = standard.clone();
    for f in standard_fields() {
        if !values[&f].is_null() {
            now[&f] = values[&f].clone();
        }
    }
    let opens = values["opens"].as_str().unwrap_or("handle");
    spec::set_opens(&mut now, opens, values["handle_position"].as_str());
    if let Ok(s) = serde_json::from_value::<cabinet_core::ConstructionStandard>(now) {
        let groups = cabinet_core::options::option_groups();
        for f in standard_fields() {
            let sized = cabinet_core::options::measurements().iter().any(|m| m.field == f);
            if sized || groups.iter().any(|g| g.field == f) {
                patch[&f]["visible"] = json!(cabinet_core::options::applies(&f, &s));
            }
        }
    }
    let handled = matches!(opens, "handle" | "knob");
    patch["handle_size"]["visible"] = json!(opens == "handle");
    patch["handle_position"]["visible"] = json!(handled);
    patch["handle_finish"]["visible"] = json!(handled || opens == "edge_pull");
    if doc["job"].is_null() {
        patch["opens"]["hint"] = json!("A new job starts from your shop standard, or Cabinet Studio's");
    }
    // The boards are chosen in the Cabinets panel's Boards tab; the setup shows them.
    let chosen: Vec<String> = boards::ROLES
        .iter()
        .filter_map(|(r, l)| stored["boards"][*r]["name"].as_str().map(|n| format!("{l}: {n}")))
        .collect();
    patch["door_style"]["hint"] = json!(if chosen.is_empty() {
        "Boards: none chosen yet. Choose them in the Cabinets panel's Boards tab".to_string()
    } else {
        format!("Boards: {}. Change them in the Cabinets panel's Boards tab", chosen.join("; "))
    });
    Ok(patch)
}

/// Add and Edit cabinet: the picked cabinet's values (Edit), the fronts its type can have, and
/// what the job's sizes are.
fn cabinet_fields(values: &Value, cab: Option<&Value>) -> Result<Value, PluginError> {
    let (stored, _) = load()?;
    let job = Job::from_stored(&stored);
    let mut patch = json!({});
    if let Some(c) = cab {
        patch["pick"] = json!({"hint": format!("Changing {}", c["id"].as_str().unwrap_or_default())});
        for f in ["kind", "width", "height", "depth", "fronts", "hinge", "shelves", "pipe_cutout", "liner",
                  "blind_side", "door_style", "opens", "remove"] {
            patch[f] = if c[f].is_null() { json!({"enabled": true}) } else { json!({"value": c[f], "enabled": true}) };
        }
        // A size stored as 0 (the job's, in 0.1.0) shows as the job's size for its type.
        if let Some(kind) = c["kind"].as_str().and_then(spec::kind) {
            let (h, d) = job.size(kind);
            for (f, v) in [("height", h), ("depth", d)] {
                if c[f].as_f64().is_none_or(|x| x <= 0.0) {
                    patch[f]["value"] = json!(v);
                }
            }
        }
    }
    let kind_name = values["kind"].as_str().or(cab.and_then(|c| c["kind"].as_str())).unwrap_or("base");
    if let Some(kind) = spec::kind(kind_name) {
        let presets = spec::presets_for(kind);
        // SolidCraft takes choices only in the manifest's order; the usual one is the default.
        let ordered: Vec<&str> = spec::PRESETS.iter().map(|(id, _)| *id).filter(|id| presets.contains(id)).collect();
        patch["fronts"]["choices"] = json!(ordered);
        patch["fronts"]["default"] = json!(presets[0]);
        let (h, d) = job.size(kind);
        let usual = cabinet_core::Cabinet::new("X", kind);
        let class = spec::size_class(kind);
        patch["height"]["default"] = json!(h);
        patch["depth"]["default"] = json!(d);
        patch["height"]["hint"] = json!(format!("The job's {class} height is {h:.0} mm"));
        patch["depth"]["hint"] = json!(format!("The job's {class} depth is {d:.0} mm"));
        patch["width"]["default"] = json!(usual.width);
        patch["shelves"]["default"] = json!(usual.adjustable_shelves);
        patch["shelves"]["visible"] = json!(presets[0] != "own" && kind != cabinet_core::CabinetKind::DrawerBank);
    }
    Ok(patch)
}

fn invalid(field: &str, reason: impl ToString) -> PluginError {
    PluginError::InvalidInput(FieldFailure { field: field.into(), reason: reason.to_string() })
}

export!(Plugin);
