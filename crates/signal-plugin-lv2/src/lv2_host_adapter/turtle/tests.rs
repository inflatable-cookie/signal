use super::*;

const LV2: &str = "http://lv2plug.in/ns/lv2core#";

fn iri(text: &str) -> TurtleTerm {
    TurtleTerm::Iri(text.to_string())
}

#[test]
fn parses_prefixes_continuations_and_object_lists() {
    let doc = TurtleDocument::parse(
        "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n\
         @prefix doap: <http://usefulinc.com/ns/doap#> .\n\
         # a comment\n\
         <http://example.com/p> a lv2:Plugin ;\n\
             doap:name \"Example\" ;\n\
             lv2:optionalFeature lv2:hardRTCapable , <http://x#y> ;\n\
             .\n",
    )
    .expect("subset document parses");
    let subject = iri("http://example.com/p");
    assert!(doc.has_type(&subject, &format!("{LV2}Plugin")));
    assert_eq!(
        doc.object(&subject, "http://usefulinc.com/ns/doap#name"),
        Some(&TurtleTerm::Literal("Example".into())),
    );
    let features: Vec<_> = doc
        .objects(&subject, &format!("{LV2}optionalFeature"))
        .collect();
    assert_eq!(
        features,
        vec![&iri(&format!("{LV2}hardRTCapable")), &iri("http://x#y")],
    );
}

#[test]
fn parses_blank_node_port_lists_with_numbers() {
    let doc = TurtleDocument::parse(
        "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n\
         <http://example.com/p>\n\
             lv2:port [\n\
                 a lv2:AudioPort , lv2:InputPort ;\n\
                 lv2:index 0 ;\n\
                 lv2:symbol \"in_l\" ;\n\
             ] , [\n\
                 a lv2:ControlPort , lv2:InputPort ;\n\
                 lv2:index 1 ;\n\
                 lv2:default 0.5 ;\n\
                 lv2:minimum -1.0 ;\n\
                 lv2:maximum 1e1 ;\n\
             ] .\n",
    )
    .expect("port shape parses");
    let subject = iri("http://example.com/p");
    let ports: Vec<_> = doc.objects(&subject, &format!("{LV2}port")).collect();
    assert_eq!(ports.len(), 2);
    let control = ports[1].clone();
    assert!(doc.has_type(&control, &format!("{LV2}ControlPort")));
    assert_eq!(
        doc.object(&control, &format!("{LV2}index"))
            .and_then(TurtleTerm::as_number),
        Some(1.0),
    );
    assert_eq!(
        doc.object(&control, &format!("{LV2}default"))
            .and_then(TurtleTerm::as_number),
        Some(0.5),
    );
    assert_eq!(
        doc.object(&control, &format!("{LV2}minimum"))
            .and_then(TurtleTerm::as_number),
        Some(-1.0),
    );
    assert_eq!(
        doc.object(&control, &format!("{LV2}maximum"))
            .and_then(TurtleTerm::as_number),
        Some(10.0),
    );
}

#[test]
fn parses_string_escapes_language_tags_and_datatypes() {
    let doc = TurtleDocument::parse(
        "@prefix doap: <http://usefulinc.com/ns/doap#> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
         <http://e/p> doap:name \"Say \\\"hi\\\"\\n\"@en ;\n\
             doap:shortdesc \"typed\"^^xsd:string .\n",
    )
    .expect("literal forms parse");
    let subject = iri("http://e/p");
    assert_eq!(
        doc.object(&subject, "http://usefulinc.com/ns/doap#name")
            .and_then(TurtleTerm::as_literal),
        Some("Say \"hi\"\n"),
    );
    assert_eq!(
        doc.object(&subject, "http://usefulinc.com/ns/doap#shortdesc")
            .and_then(TurtleTerm::as_literal),
        Some("typed"),
    );
}

#[test]
fn parses_booleans_and_labeled_blank_nodes() {
    let doc = TurtleDocument::parse(
        "@prefix ex: <http://e#> .\n\
         _:b ex:flag true .\n\
         _:b ex:other false .\n",
    )
    .expect("labeled blanks parse");
    assert_eq!(doc.triples.len(), 2);
    assert_eq!(doc.triples[0].subject, doc.triples[1].subject);
    assert_eq!(doc.triples[0].object, TurtleTerm::Bool(true));
    assert_eq!(doc.triples[1].object, TurtleTerm::Bool(false));
}

#[test]
fn rejects_constructs_outside_the_subset_without_panicking() {
    let rejected = [
        "@base <http://example.com/> .",
        "PREFIX ex: <http://e#>\n<http://p> ex:x 1 .",
        "@prefix ex: <http://e#> .\n<http://p> ex:list ( 1 2 ) .",
        "@prefix ex: <http://e#> .\n<http://p> ex:name \"\"\"long\"\"\" .",
        "@prefix ex: <http://e#> .\n[ ex:x 1 ] ex:y 2 .",
        "<http://p> unknown:pred 1 .",
        "@prefix ex: <http://e#> .\n<http://p> ex:x \"unterminated .",
        "@prefix ex: <http://e#> .\n<http://p> ex:x 1",
    ];
    for source in rejected {
        assert!(
            TurtleDocument::parse(source).is_err(),
            "should reject: {source}",
        );
    }
}

#[test]
fn merge_keeps_blank_nodes_distinct() {
    let mut left =
        TurtleDocument::parse("@prefix ex: <http://e#> .\n<http://a> ex:port [ ex:index 0 ] .")
            .expect("left parses");
    let right =
        TurtleDocument::parse("@prefix ex: <http://e#> .\n<http://b> ex:port [ ex:index 1 ] .")
            .expect("right parses");
    left.merge(&right);
    let a_port = left
        .object(&iri("http://a"), "http://e#port")
        .cloned()
        .expect("left port");
    let b_port = left
        .object(&iri("http://b"), "http://e#port")
        .cloned()
        .expect("right port");
    assert_ne!(a_port, b_port, "blank ids must not collide across merges");
    assert_eq!(
        left.object(&b_port, "http://e#index")
            .and_then(TurtleTerm::as_number),
        Some(1.0),
    );
}
