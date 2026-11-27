//! What the plugin keeps in the design (docs/PLAN.md, Data): the job's setup and each cabinet's
//! own choices, and how they become Cabinet Studio's `Cabinet`. Only translation lives here: the
//! dialogs' choices into the core's model. Every construction rule stays in the core.
//!
//! Plain Rust with no SolidCraft imports, so `examples/catalog.rs` can draw the dialogs' pictures
//! from it on the host.
use cabinet_core::model::{FrontLayout, HingeSide, SectionKind, StackSection};
use cabinet_core::{Cabinet, CabinetKind, ConstructionStandard};
use serde_json::{Map, Value, json};

/// The fronts a cabinet type can have, as the dialog's picture choices, in the order shown.
pub const PRESETS: &[(&str, &str)] = &[
    ("door", "One door"),
    ("doors", "Two doors"),
    ("drawer-door", "Drawer over a door"),
    ("drawers-2", "Two drawers"),
    ("drawers-3", "Three drawers"),
    ("drawers-4", "Four drawers"),
    ("pots", "A drawer over two deep ones"),
    ("open", "Open"),
    ("own", "Its own fronts"),
];

/// The front presets that suit `kind`, the first being its usual one.
pub fn presets_for(kind: CabinetKind) -> &'static [&'static str] {
    match kind {
        CabinetKind::Base => &["doors", "door", "drawer-door", "open"],
        CabinetKind::DrawerBank => &["drawers-3", "drawers-2", "drawers-4", "pots"],
        CabinetKind::Sink => &["doors", "door"],
        CabinetKind::BlindCorner => &["door"],
        CabinetKind::Wall => &["doors", "door", "open"],
        CabinetKind::Tall => &["doors", "door"],
        CabinetKind::Wardrobe => &["doors", "door", "open"],
        CabinetKind::OpenShelf => &["open"],
        CabinetKind::CornerBase | CabinetKind::CornerWall | CabinetKind::Rangehood => &["own"],
    }
}

/// What the dialog calls each cabinet type, in the order it shows them.
pub const KINDS: &[(&str, &str)] = &[
    ("base", "Base"),
    ("drawer_bank", "Drawers"),
    ("sink", "Sink"),
    ("blind_corner", "Blind corner"),
    ("corner_base", "L corner"),
    ("wall", "Wall"),
    ("corner_wall", "Wall corner"),
    ("rangehood", "Rangehood"),
    ("tall", "Tall"),
    ("wardrobe", "Wardrobe"),
    ("open_shelf", "Open shelf"),
];

/// How fronts open, one choice for what Cabinet Studio splits over four pickers (opening,
/// handle range, handle place, the sharknose pickers): docs/PLAN.md, Job setup.
#[allow(dead_code)]
pub const OPENS: &[(&str, &str)] = &[
    ("handle", "Handles"),
    ("knob", "Knobs"),
    ("edge_pull", "Edge pulls"),
    ("finger_pull", "Finger pull, routed"),
    ("sharknose", "Sharknose"),
    ("tip_on", "TIP-ON, push to open"),
    ("servo_drive", "SERVO-DRIVE, electric"),
];

pub fn kind(name: &str) -> Option<CabinetKind> {
    serde_json::from_value(json!(name)).ok()
}

/// Which of the job's standard sizes a type takes: base, wall or tall.
pub fn size_class(kind: CabinetKind) -> &'static str {
    match kind {
        CabinetKind::Wall | CabinetKind::CornerWall | CabinetKind::Rangehood | CabinetKind::OpenShelf => "wall",
        CabinetKind::Tall | CabinetKind::Wardrobe => "tall",
        _ => "base",
    }
}

/// The job's setup: the construction standard as the core reads it, and the standard sizes.
#[derive(Clone, Debug)]
pub struct Job {
    pub standard: ConstructionStandard,
    /// `{"base": [height, depth], "wall": …, "tall": …}`.
    pub sizes: Map<String, Value>,
    pub detail: String,
    /// The benchtop's board, chosen with the Boards button.
    pub benchtop_board: String,
}

impl Job {
    /// The job stored in the design, over the core's defaults for anything it doesn't say.
    pub fn from_stored(v: &Value) -> Job {
        let mut standard = serde_json::to_value(ConstructionStandard::default()).unwrap_or(Value::Null);
        if let (Some(base), Some(over)) = (standard.as_object_mut(), v["standard"].as_object()) {
            for (k, x) in over {
                base.insert(k.clone(), x.clone());
            }
        }
        let mut sizes = Map::new();
        for (class, kind) in [("base", CabinetKind::Base), ("wall", CabinetKind::Wall), ("tall", CabinetKind::Tall)] {
            let (_, d, h) = kind.default_size();
            let stored = &v["sizes"][class];
            let pick = |i: usize, dflt: f64| stored[i].as_f64().filter(|x| *x > 0.0).unwrap_or(dflt);
            sizes.insert(class.into(), json!([pick(0, h), pick(1, d)]));
        }
        Job {
            standard: serde_json::from_value(standard).unwrap_or_default(),
            sizes,
            detail: v["detail"].as_str().unwrap_or("design").into(),
            benchtop_board: v["boards"]["benchtop"]["id"].as_str().unwrap_or_default().into(),
        }
    }

    pub fn size(&self, kind: CabinetKind) -> (f64, f64) {
        let s = &self.sizes[size_class(kind)];
        (s[0].as_f64().unwrap_or(0.0), s[1].as_f64().unwrap_or(0.0))
    }
}

/// The standard's fields that "How fronts open" stands for, from one choice.
pub fn set_opens(s: &mut Value, opens: &str, place: Option<&str>) {
    let (opening, handle_place) = match opens {
        "tip_on" => ("tip_on", None),
        "servo_drive" => ("servo_drive", None),
        "edge_pull" => ("handles", Some("edge_pull")),
        "finger_pull" => ("handles", Some("finger_pull")),
        "sharknose" => ("handles", Some("sharknose")),
        _ => ("handles", Some(place.unwrap_or("corner"))),
    };
    s["opening"] = json!(opening);
    if let Some(p) = handle_place {
        s["handle_place"] = json!(p);
    }
    if opens == "knob" {
        s["handle_size"] = json!("knob");
    } else if s["handle_size"] == "knob" {
        s["handle_size"] = json!("cc128");
    }
    // Generic handles only: the plugin carries no supplier's range yet.
    s["handle_range"] = json!("generic");
}

/// The one choice a standard's opening fields come to.
pub fn opens_of(s: &Value) -> &'static str {
    match (s["opening"].as_str(), s["handle_place"].as_str(), s["handle_size"].as_str()) {
        (Some("tip_on"), ..) => "tip_on",
        (Some("servo_drive"), ..) => "servo_drive",
        (_, Some("edge_pull"), _) => "edge_pull",
        (_, Some("finger_pull"), _) => "finger_pull",
        (_, Some("sharknose"), _) => "sharknose",
        (_, _, Some("knob")) => "knob",
        _ => "handle",
    }
}

fn hinge(name: &str) -> HingeSide {
    if name == "right" { HingeSide::Right } else { HingeSide::Left }
}

/// The core's front layout for a preset.
pub fn layout(preset: &str, hinge_side: &str) -> Option<FrontLayout> {
    let drawers = |r: &[f64]| FrontLayout::Drawers { ratios: r.to_vec() };
    Some(match preset {
        "door" => FrontLayout::Door { hinge_side: hinge(hinge_side) },
        "doors" => FrontLayout::DoorPair,
        "open" => FrontLayout::Open,
        "drawers-2" => drawers(&[1.0, 1.0]),
        "drawers-3" => drawers(&[1.0, 1.0, 1.0]),
        "drawers-4" => drawers(&[1.0, 1.0, 1.0, 1.0]),
        "pots" => drawers(&[1.0, 2.0, 2.0]),
        "drawer-door" => FrontLayout::Stack {
            sections: vec![
                StackSection { height: 0.0, kind: SectionKind::DoorPair },
                StackSection { height: DRAWER_OVER_DOOR, kind: SectionKind::Drawers { count: 1 } },
            ],
        },
        _ => return None,
    })
}

/// A drawer over doors: the drawer front's height. Our choice, as in Cabinet Studio's template.
const DRAWER_OVER_DOOR: f64 = 150.0;

/// A stored cabinet as the core's `Cabinet`, over the job's setup.
pub fn cabinet(spec: &Value, job: &Job) -> Option<Cabinet> {
    let kind = kind(spec["kind"].as_str()?)?;
    let mut cab = Cabinet::new(spec["id"].as_str()?, kind);
    let (h, d) = job.size(kind);
    cab.width = spec["width"].as_f64().filter(|x| *x > 0.0).unwrap_or(cab.width);
    cab.height = spec["height"].as_f64().filter(|x| *x > 0.0).unwrap_or(h);
    cab.depth = spec["depth"].as_f64().filter(|x| *x > 0.0).unwrap_or(d);
    // The job's standard, with what this cabinet does differently.
    let mut standard = serde_json::to_value(&job.standard).ok()?;
    if let Some(style) = spec["door_style"].as_str().filter(|s| *s != "job") {
        standard["door_style"] = json!(style);
    }
    if let Some(opens) = spec["opens"].as_str().filter(|s| *s != "job") {
        let place = standard["handle_place"].as_str().map(String::from);
        set_opens(&mut standard, opens, place.as_deref());
    }
    cab.standard = serde_json::from_value(standard).ok()?;
    let preset = spec["fronts"].as_str().unwrap_or("");
    let preset = if presets_for(kind).contains(&preset) { preset } else { presets_for(kind)[0] };
    if let Some(l) = layout(preset, spec["hinge"].as_str().unwrap_or("left")) {
        cab.fronts = l;
    }
    if let Some(n) = spec["shelves"].as_u64() {
        cab.adjustable_shelves = n as u32;
    }
    if kind == CabinetKind::Sink {
        cab.sink.back_cutout = spec["pipe_cutout"].as_bool().unwrap_or(true);
        cab.sink.liner = spec["liner"].as_bool().unwrap_or(true);
    }
    if kind == CabinetKind::BlindCorner {
        cab.blind.side = hinge(spec["blind_side"].as_str().unwrap_or("left"));
    }
    Some(cab)
}

/// What the dialogs show of a stored cabinet: its fronts in words.
pub fn fronts_label(spec: &Value, kind: CabinetKind) -> String {
    let preset = spec["fronts"].as_str().filter(|p| presets_for(kind).contains(p)).unwrap_or(presets_for(kind)[0]);
    PRESETS.iter().find(|(id, _)| *id == preset).map_or(String::new(), |(_, l)| (*l).to_string())
}

/// What a stored run item is: a cabinet (the default, as 0.1–0.4 stored only cabinets), or one of
/// the run's pieces (PLUG-3).
pub fn item_kind(spec: &Value) -> &str {
    spec["item"].as_str().unwrap_or("cabinet")
}

/// The edges a benchtop's dialog offers (the core's profiles; a mitred apron is a stone top's,
/// left for later).
#[allow(dead_code)]
pub const EDGES: &[(&str, &str)] = &[("square", "Square"), ("pencil_round", "Pencil round"), ("bullnose", "Bullnose"), ("bevel", "Bevel")];

/// A stored run item as the core's: a cabinet over the job's setup, or a piece.
pub fn item(spec: &Value, job: &Job, benchtop_board: &str) -> Option<cabinet_core::run::RunItem> {
    use cabinet_core::run::RunItem;
    let id = spec["id"].as_str()?;
    let num = |k: &str, d: f64| spec[k].as_f64().unwrap_or(d);
    Some(match item_kind(spec) {
        "end_panel" => RunItem::EndPanel(serde_json::from_value(json!({
            "id": id, "scribe": num("scribe", 0.0), "to_floor": spec["to_floor"].as_bool().unwrap_or(true),
        })).ok()?),
        "filler" => RunItem::Filler(serde_json::from_value(json!({
            "id": id, "width": num("width", 50.0), "scribe": num("scribe", 0.0),
        })).ok()?),
        "benchtop" => {
            let mut top = json!({
                "id": id,
                "board": benchtop_board,
                "front_overhang": num("front_overhang", 20.0),
                "side_overhang": num("side_overhang", 0.0),
                "edge": {"type": spec["edge"].as_str().unwrap_or("square")},
                "splashback": num("splashback", 0.0),
                "cutouts": [],
            });
            if let Some(t) = spec["thickness"].as_f64().filter(|t| *t > 0.0) {
                top["thickness"] = json!(t);
            }
            if let Some(t) = spec["splashback_thickness"].as_f64().filter(|t| *t > 0.0) {
                top["splashback_thickness"] = json!(t);
            }
            if let Some(over) = spec["sink_over"].as_str().filter(|s| !s.is_empty()) {
                let sink = spec["sink"].as_str().unwrap_or("custom");
                let mut c = json!({"over": over, "kind": "sink", "width": num("sink_width", 760.0), "depth": num("sink_depth", 450.0)});
                if sink != "custom" {
                    c["product"] = json!(sink);
                }
                top["cutouts"].as_array_mut()?.push(c);
            }
            if let Some(over) = spec["cooktop_over"].as_str().filter(|s| !s.is_empty()) {
                top["cutouts"].as_array_mut()?.push(json!({
                    "over": over, "kind": "cooktop", "width": num("cooktop_width", 560.0), "depth": num("cooktop_depth", 490.0),
                }));
            }
            RunItem::Benchtop(serde_json::from_value(top).ok()?)
        }
        _ => RunItem::Cabinet(Box::new(cabinet(spec, job)?)),
    })
}

/// What the Cabinets panel calls an item.
pub fn item_label(spec: &Value) -> String {
    match item_kind(spec) {
        "end_panel" => "End panel".into(),
        "filler" => "Filler".into(),
        "benchtop" => "Benchtop".into(),
        _ => KINDS.iter().find(|(k, _)| Some(*k) == spec["kind"].as_str()).map_or(String::new(), |(_, l)| (*l).to_string()),
    }
}
