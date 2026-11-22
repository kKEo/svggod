//! The library, used as another crate uses it: only what `svggod` exports.

use svggod::{Effort, Error, Profile, Renderer, Severity, Validator};

const SPEC: &str = "version: 1\ndiagram: flow\ntitle: Request path\nedges:\n  - \"browser -> api: request\"\n  - \"api -> database: query\"\n";

#[test]
fn render_gives_svg_that_passes_its_own_rules() {
    let svg = svggod::render(SPEC).unwrap();
    assert!(svg.starts_with("<svg"), "{}", &svg[..40]);
    let findings = svggod::validate("request.svg", &svg);
    assert!(findings.iter().all(|f| f.severity < Severity::Warning), "{findings:?}");
}

#[test]
fn render_is_deterministic_and_options_change_the_drawing() {
    let plain = Renderer::new();
    assert_eq!(plain.render(SPEC).unwrap(), plain.render(SPEC).unwrap());
    let dark = Renderer::new().theme("blueprint-dark").effort(Effort::High).render(SPEC).unwrap();
    assert_ne!(dark, plain.render(SPEC).unwrap());
    assert!(!Renderer::new().text_as_paths(true).render(SPEC).unwrap().contains("<text"));
}

#[test]
fn errors_carry_a_code_a_path_and_a_message() {
    let e: Error = svggod::render("version: 1\ndiagram: flow\ntitle: T\nnodes:\n  - { id: a, label: A, kind: blob }\n").unwrap_err();
    assert_eq!(e.code, "invalid-value");
    assert_eq!(e.path.as_deref(), Some("nodes[0].kind"));
    // It is an ordinary error: it has a message and goes through `?` into a boxed error.
    let boxed: Box<dyn std::error::Error + Send + Sync> = e.into();
    assert!(boxed.to_string().contains("blob"));

    assert_eq!(svggod::render("{ not json").unwrap_err().code, "malformed-spec");
    assert_eq!(Renderer::new().theme("nope").render(SPEC).unwrap_err().code, "unknown-theme");
    assert!(svggod::themes().iter().any(|t| t == "blueprint"));
}

#[test]
fn validator_selects_rules_and_reads_config() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10"/></svg>"#;
    let all = svggod::validate("icon.svg", svg);
    assert!(all.iter().any(|f| f.rule == "viewbox-missing"), "{all:?}");
    assert!(all.iter().all(|f| f.file == "icon.svg"));

    let only = Validator::new().only(&["viewbox-missing"]).unwrap().validate("icon.svg", svg);
    assert!(!only.is_empty() && only.iter().all(|f| f.rule == "viewbox-missing"));
    let skipped = Validator::new().skip(&["viewbox-missing"]).unwrap().validate("icon.svg", svg);
    assert!(skipped.iter().all(|f| f.rule != "viewbox-missing"));
    let off = Validator::new().config("[rules]\nviewbox-missing = \"off\"\n").unwrap().validate("icon.svg", svg);
    assert!(off.iter().all(|f| f.rule != "viewbox-missing"));
    let strict = Validator::new().profile(Profile::Strict).validate("icon.svg", svg);
    assert!(strict.iter().filter(|f| f.severity == Severity::Error).count() >= all.iter().filter(|f| f.severity == Severity::Error).count());

    assert_eq!(Validator::new().only(&["no-such-rule"]).err().unwrap().code, "unknown-rule");
    assert_eq!(Validator::new().config("[rules]\nnope = \"off\"\n").err().unwrap().code, "unknown-rule");
    assert_eq!(svggod::validate("x.svg", "<svg")[0].rule, "xml-well-formed");
}

#[test]
fn fix_applies_what_the_rules_offer() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><title>Dot</title><rect width="10" height="10"/></svg>"#;
    let fixed = Validator::new().fix("icon.svg", svg, false).unwrap();
    assert!(fixed.applied.contains_key("viewbox-missing"), "{:?}", fixed.applied);
    assert!(fixed.svg.contains("viewBox"));
    assert!(fixed.remaining.iter().all(|f| f.rule != "viewbox-missing"));
}

#[test]
fn describe_reads_a_spec_and_a_drawing_that_carries_one() {
    let d = svggod::describe(SPEC).unwrap();
    assert_eq!(d.title, "Request path");
    assert!(d.short.contains("browser") && d.text.len() > d.short.len());
    let svg = Renderer::new().embed_spec(true).render(SPEC).unwrap();
    assert_eq!(svggod::describe(&svg).unwrap(), d);
    assert_eq!(svggod::describe(&svggod::render(SPEC).unwrap()).unwrap_err().code, "no-spec");
}

#[cfg(feature = "preview")]
#[test]
fn rasterize_gives_a_png_of_the_asked_width() {
    let image = svggod::rasterize(&svggod::render(SPEC).unwrap(), 400).unwrap();
    assert_eq!(image.width, 400);
    assert!(image.height > 0 && image.png.starts_with(b"\x89PNG"));
    assert_eq!(svggod::rasterize("<svg", 400).unwrap_err().code, "malformed-xml");
}

#[test]
fn the_types_cross_threads() {
    fn shared<T: Send + Sync>() {}
    shared::<Renderer>();
    shared::<Validator>();
    shared::<Error>();
    shared::<svggod::Finding>();
}
