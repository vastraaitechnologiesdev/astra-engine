use astra_dom::{Document, DomError};

#[test]
fn test_document_initialization() {
    let doc = Document::new();
    assert_eq!(doc.node_count(), 1);
    let root = doc.get(doc.root()).expect("root node must exist");
    assert!(root.is_document());
    assert_eq!(doc.children(doc.root()).count(), 0);
}

#[test]
fn test_node_creation_and_payloads() {
    let mut doc = Document::new();

    let elem_id = doc.create_element("div");
    let text_id = doc.create_text("Hello World");
    let comment_id = doc.create_comment("Sample comment");
    let doctype_id = doc.create_doctype("html", None, None);

    assert_eq!(
        doc.get(elem_id).unwrap().as_element().unwrap().tag_name(),
        "div"
    );
    assert_eq!(
        doc.get(text_id).unwrap().as_text().unwrap().content(),
        "Hello World"
    );
    assert_eq!(
        doc.get(comment_id).unwrap().as_comment().unwrap().content(),
        "Sample comment"
    );
    assert_eq!(
        doc.get(doctype_id).unwrap().as_doctype().unwrap().name(),
        "html"
    );
}

#[test]
fn test_element_attributes() {
    let mut doc = Document::new();
    let elem_id = doc.create_element("a");

    {
        let elem = doc.get_mut(elem_id).unwrap().as_element_mut().unwrap();
        elem.set_attribute("href", "https://example.com");
        elem.set_attribute("class", "btn btn-primary active");
        elem.set_attribute("id", "main-link");
    }

    let elem = doc.get(elem_id).unwrap().as_element().unwrap();
    assert_eq!(elem.get_attribute("href"), Some("https://example.com"));
    assert_eq!(elem.get_attribute("HREF"), Some("https://example.com"));
    assert_eq!(elem.id(), Some("main-link"));
    assert_eq!(elem.class_name(), Some("btn btn-primary active"));

    let classes: Vec<&str> = elem.classes().collect();
    assert_eq!(classes, vec!["btn", "btn-primary", "active"]);

    {
        let elem = doc.get_mut(elem_id).unwrap().as_element_mut().unwrap();
        assert_eq!(
            elem.remove_attribute("href"),
            Some("https://example.com".to_string())
        );
    }
    let elem = doc.get(elem_id).unwrap().as_element().unwrap();
    assert_eq!(elem.get_attribute("href"), None);
}

#[test]
fn test_parent_child_relationships_and_ordering() {
    let mut doc = Document::new();
    let root = doc.root();

    let body = doc.create_element("body");
    doc.append_child(root, body).unwrap();

    let p1 = doc.create_element("p");
    let p2 = doc.create_element("p");
    let p3 = doc.create_element("p");

    doc.append_child(body, p1).unwrap();
    doc.append_child(body, p3).unwrap();
    doc.insert_before(body, p2, p3).unwrap();

    let children: Vec<_> = doc.children(body).collect();
    assert_eq!(children, vec![p1, p2, p3]);

    assert_eq!(doc.first_child(body), Some(p1));
    assert_eq!(doc.last_child(body), Some(p3));
    assert_eq!(doc.previous_sibling(p2), Some(p1));
    assert_eq!(doc.next_sibling(p2), Some(p3));
    assert_eq!(doc.parent(p2), Some(body));
}

#[test]
fn test_detachment_and_reinsertion() {
    let mut doc = Document::new();
    let root = doc.root();

    let list = doc.create_element("ul");
    doc.append_child(root, list).unwrap();

    let item1 = doc.create_element("li");
    let item2 = doc.create_element("li");
    doc.append_child(list, item1).unwrap();
    doc.append_child(list, item2).unwrap();

    // Detach item1
    doc.detach(item1).unwrap();
    assert_eq!(doc.children(list).collect::<Vec<_>>(), vec![item2]);
    assert_eq!(doc.parent(item1), None);
    assert_eq!(doc.first_child(list), Some(item2));

    // Re-insert item1 after item2 (by appending)
    doc.append_child(list, item1).unwrap();
    assert_eq!(doc.children(list).collect::<Vec<_>>(), vec![item2, item1]);
}

#[test]
fn test_cycle_detection_prevention() {
    let mut doc = Document::new();
    let root = doc.root();

    let div = doc.create_element("div");
    let span = doc.create_element("span");

    doc.append_child(root, div).unwrap();
    doc.append_child(div, span).unwrap();

    // Trying to append div to span should fail because span is a descendant of div
    let err = doc.append_child(span, div).unwrap_err();
    assert_eq!(
        err,
        DomError::CycleDetected {
            parent: span,
            child: div
        }
    );

    // Self-attachment should fail
    let err = doc.append_child(div, div).unwrap_err();
    assert_eq!(
        err,
        DomError::CycleDetected {
            parent: div,
            child: div
        }
    );
}

#[test]
fn test_tree_traversals_and_text_content() {
    let mut doc = Document::new();
    let root = doc.root();

    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();

    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    let h1 = doc.create_element("h1");
    let t1 = doc.create_text("Title");
    doc.append_child(body, h1).unwrap();
    doc.append_child(h1, t1).unwrap();

    let p = doc.create_element("p");
    let t2 = doc.create_text("Paragraph ");
    let b = doc.create_element("b");
    let t3 = doc.create_text("bold");
    doc.append_child(body, p).unwrap();
    doc.append_child(p, t2).unwrap();
    doc.append_child(p, b).unwrap();
    doc.append_child(b, t3).unwrap();

    // Text content
    assert_eq!(doc.text_content(h1), "Title");
    assert_eq!(doc.text_content(p), "Paragraph bold");
    assert_eq!(doc.text_content(body), "TitleParagraph bold");

    // Element children
    let body_elements: Vec<_> = doc.element_children(body).collect();
    assert_eq!(body_elements, vec![h1, p]);

    // Ancestors
    let p_ancestors: Vec<_> = doc.ancestors(p).collect();
    assert_eq!(p_ancestors, vec![body, html, root]);

    // Queries
    let headings = doc.get_elements_by_tag_name("h1");
    assert_eq!(headings, vec![h1]);
}
