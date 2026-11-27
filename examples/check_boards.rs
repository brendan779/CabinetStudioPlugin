//! Checks the Boards dialog's narrowing (src/boards.rs) on the real board index, as no headless
//! test draws a dialog. `scripts/build.sh --test` runs it.
//!
//!     cargo run -q --example check_boards --target aarch64-apple-darwin
#[path = "../src/boards.rs"]
#[allow(dead_code)]
mod boards;

fn main() {
    // The owner's case (0.4.1): "delana" found nothing in Laminex's first range.
    let n = boards::narrow("Laminex", "carcass", "Compact Laminate", "delana", "", "");
    assert!(n.matched, "delana is a Laminex decor");
    assert_eq!(n.range, "Decorated MDF", "{:?}", n.ranges);
    assert!(n.decors.iter().all(|d| d.to_lowercase().contains("delana")), "{:?}", n.decors);
    assert_eq!(n.decor, "Delana Oak");
    assert!(n.thicknesses.contains(&18.0), "{:?}", n.thicknesses);

    // Nothing typed: the brand's biggest range, every decor in it.
    let n = boards::narrow("Laminex", "carcass", "", "", "", "");
    assert_eq!(n.range, "Decorated MDF");
    assert!(n.decors.len() > 100 && n.decors.len() <= 256, "{}", n.decors.len());

    // A range that still fits is kept; nothing found keeps the brand's ranges but matches none.
    assert_eq!(boards::narrow("Laminex", "carcass", "Compact Laminate", "", "", "").range, "Compact Laminate");
    assert!(!boards::narrow("Laminex", "carcass", "", "zzqx", "", "").matched);

    // Every brand and role: lists sorted (the manifest's order) and at most 256 (SolidCraft's).
    let brands: Vec<String> = boards::all().map(|b| b.brand.to_string()).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
    for brand in &brands {
        for (role, _) in boards::ROLES {
            let n = boards::narrow(brand, role, "", "", "", "");
            for list in [&n.ranges, &n.decors, &n.finishes] {
                assert!(list.windows(2).all(|w| w[0] < w[1]) && list.len() <= 256, "{brand} {role}");
            }
        }
    }
    println!("boards narrowing: ok ({} brands)", brands.len());
}
