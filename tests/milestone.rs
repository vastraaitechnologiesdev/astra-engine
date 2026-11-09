use astra_html::parse_html;

#[test]
fn test_milestone_html_document() {
    let html = r#"<!DOCTYPE html>
<html>
    <head>
        <title>Astra Engine</title>
    </head>
    <body>
        <h1>Hello Astra!</h1>
        <p>Our own browser engine.</p>
    </body>
</html>"#;

    let doc = parse_html(html);

    // Verify root contains doctype and html element
    let root = doc.root();
    let root_children: Vec<_> = doc.children(root).collect();
    assert!(
        root_children.len() >= 2,
        "Root should have at least DOCTYPE and HTML element"
    );

    // Verify DOCTYPE
    let doctype_id = root_children[0];
    let doctype_node = doc.get(doctype_id).expect("DOCTYPE node exists");
    assert!(doctype_node.is_doctype());
    assert_eq!(doctype_node.as_doctype().unwrap().name(), "html");

    // Verify <html> element
    let html_elements = doc.get_elements_by_tag_name("html");
    assert_eq!(html_elements.len(), 1);
    let html_id = html_elements[0];

    // Verify <head> and <body> elements under <html>
    let html_child_elements: Vec<_> = doc.element_children(html_id).collect();
    assert_eq!(html_child_elements.len(), 2);
    let head_id = html_child_elements[0];
    let body_id = html_child_elements[1];

    assert_eq!(
        doc.get(head_id).unwrap().as_element().unwrap().tag_name(),
        "head"
    );
    assert_eq!(
        doc.get(body_id).unwrap().as_element().unwrap().tag_name(),
        "body"
    );

    // Verify <head> contains <title>
    let head_child_elements: Vec<_> = doc.element_children(head_id).collect();
    assert_eq!(head_child_elements.len(), 1);
    let title_id = head_child_elements[0];
    assert_eq!(
        doc.get(title_id).unwrap().as_element().unwrap().tag_name(),
        "title"
    );
    assert_eq!(doc.text_content(title_id), "Astra Engine");

    // Verify <body> contains <h1> and <p>
    let body_child_elements: Vec<_> = doc.element_children(body_id).collect();
    assert_eq!(body_child_elements.len(), 2);
    let h1_id = body_child_elements[0];
    let p_id = body_child_elements[1];

    assert_eq!(
        doc.get(h1_id).unwrap().as_element().unwrap().tag_name(),
        "h1"
    );
    assert_eq!(doc.text_content(h1_id), "Hello Astra!");

    assert_eq!(doc.get(p_id).unwrap().as_element().unwrap().tag_name(), "p");
    assert_eq!(doc.text_content(p_id), "Our own browser engine.");

    // Print dumped tree to confirm representation
    let dump = doc.dump_tree();
    println!("Milestone Tree Dump:\n{dump}");
    assert!(dump.contains("<!DOCTYPE html>"));
    assert!(dump.contains("<html>"));
    assert!(dump.contains("<head>"));
    assert!(dump.contains("<title>"));
    assert!(dump.contains("<body>"));
    assert!(dump.contains("<h1>"));
    assert!(dump.contains("<p>"));
}
