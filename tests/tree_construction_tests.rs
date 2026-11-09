use astra_dom::QuirksMode;
use astra_html::parse_html;

#[test]
fn test_automatic_structure_insertion() {
    let html = "<title>Example</title><p>Hello</p>";
    let doc = parse_html(html);

    // Root should have <html>
    let html_elements = doc.get_elements_by_tag_name("html");
    assert_eq!(html_elements.len(), 1, "Must contain exactly one <html>");
    let html_id = html_elements[0];

    // Under <html>, must have <head> and <body>
    let html_children: Vec<_> = doc.element_children(html_id).collect();
    assert_eq!(html_children.len(), 2, "<html> must have <head> and <body>");

    let head_id = html_children[0];
    let body_id = html_children[1];

    assert_eq!(
        doc.get(head_id).unwrap().as_element().unwrap().tag_name(),
        "head"
    );
    assert_eq!(
        doc.get(body_id).unwrap().as_element().unwrap().tag_name(),
        "body"
    );

    // Under <head>, must have <title>
    let head_children: Vec<_> = doc.element_children(head_id).collect();
    assert_eq!(head_children.len(), 1);
    let title_id = head_children[0];
    assert_eq!(
        doc.get(title_id).unwrap().as_element().unwrap().tag_name(),
        "title"
    );
    assert_eq!(doc.text_content(title_id), "Example");

    // Under <body>, must have <p>
    let body_children: Vec<_> = doc.element_children(body_id).collect();
    assert_eq!(body_children.len(), 1);
    let p_id = body_children[0];
    assert_eq!(doc.get(p_id).unwrap().as_element().unwrap().tag_name(), "p");
    assert_eq!(doc.text_content(p_id), "Hello");
}

#[test]
fn test_paragraph_auto_closing_by_nested_paragraph() {
    let html = "<p>First<p>Second<p>Third";
    let doc = parse_html(html);

    let paragraphs = doc.get_elements_by_tag_name("p");
    assert_eq!(paragraphs.len(), 3);

    // All paragraphs must be siblings under <body>, NOT nested inside each other!
    let p1 = paragraphs[0];
    let p2 = paragraphs[1];
    let p3 = paragraphs[2];

    assert_eq!(doc.next_element_sibling(p1), Some(p2));
    assert_eq!(doc.next_element_sibling(p2), Some(p3));
    assert_eq!(doc.text_content(p1), "First");
    assert_eq!(doc.text_content(p2), "Second");
    assert_eq!(doc.text_content(p3), "Third");
}

#[test]
fn test_paragraph_auto_closing_by_block_elements() {
    let html = "<p>Text before div<div>Inside div</div><p>Text before ul<ul><li>Item</li></ul>";
    let doc = parse_html(html);

    let body_id = doc.get_elements_by_tag_name("body")[0];
    let body_elements: Vec<_> = doc.element_children(body_id).collect();

    // Elements should be: <p>, <div>, <p>, <ul>
    assert_eq!(body_elements.len(), 4);

    let tag_names: Vec<&str> = body_elements
        .iter()
        .map(|&id| doc.get(id).unwrap().as_element().unwrap().tag_name())
        .collect();
    assert_eq!(tag_names, vec!["p", "div", "p", "ul"]);
}

#[test]
fn test_list_item_auto_closing() {
    let html = "<ul><li>Alpha<li>Beta<li>Gamma</ul>";
    let doc = parse_html(html);

    let ul_id = doc.get_elements_by_tag_name("ul")[0];
    let list_items: Vec<_> = doc.element_children(ul_id).collect();

    assert_eq!(list_items.len(), 3, "<li> elements must be siblings");
    assert_eq!(doc.text_content(list_items[0]), "Alpha");
    assert_eq!(doc.text_content(list_items[1]), "Beta");
    assert_eq!(doc.text_content(list_items[2]), "Gamma");
}

#[test]
fn test_definition_list_auto_closing() {
    let html = "<dl><dt>Term 1<dd>Def 1<dt>Term 2<dd>Def 2</dl>";
    let doc = parse_html(html);

    let dl_id = doc.get_elements_by_tag_name("dl")[0];
    let dl_children: Vec<_> = doc.element_children(dl_id).collect();

    assert_eq!(dl_children.len(), 4);
    let tags: Vec<&str> = dl_children
        .iter()
        .map(|&id| doc.get(id).unwrap().as_element().unwrap().tag_name())
        .collect();
    assert_eq!(tags, vec!["dt", "dd", "dt", "dd"]);
}

#[test]
fn test_heading_auto_closing() {
    let html = "<h1>Heading 1<h2>Heading 2<h3>Heading 3<p>Paragraph</p>";
    let doc = parse_html(html);

    let body_id = doc.get_elements_by_tag_name("body")[0];
    let body_children: Vec<_> = doc.element_children(body_id).collect();

    assert_eq!(body_children.len(), 4);
    let tags: Vec<&str> = body_children
        .iter()
        .map(|&id| doc.get(id).unwrap().as_element().unwrap().tag_name())
        .collect();
    assert_eq!(tags, vec!["h1", "h2", "h3", "p"]);
}

#[test]
fn test_table_parsing_auto_tbody() {
    let html = "<table><tr><td>Cell 1</td><td>Cell 2</td></tr></table>";
    let doc = parse_html(html);

    let table_id = doc.get_elements_by_tag_name("table")[0];
    let table_children: Vec<_> = doc.element_children(table_id).collect();

    // <tbody> must be automatically inserted!
    assert_eq!(table_children.len(), 1);
    let tbody_id = table_children[0];
    assert_eq!(
        doc.get(tbody_id).unwrap().as_element().unwrap().tag_name(),
        "tbody"
    );

    let tr_id = doc.element_children(tbody_id).next().unwrap();
    assert_eq!(
        doc.get(tr_id).unwrap().as_element().unwrap().tag_name(),
        "tr"
    );

    let cells: Vec<_> = doc.element_children(tr_id).collect();
    assert_eq!(cells.len(), 2);
    assert_eq!(doc.text_content(cells[0]), "Cell 1");
    assert_eq!(doc.text_content(cells[1]), "Cell 2");
}

#[test]
fn test_foster_parenting_text_in_table() {
    let html = "<body><table>Misnested Text<tr><td>Cell</td></tr></table></body>";
    let doc = parse_html(html);

    let body_id = doc.get_elements_by_tag_name("body")[0];
    let table_id = doc.get_elements_by_tag_name("table")[0];

    // The text "Misnested Text" must be foster-parented into <body> BEFORE the <table>!
    let prev = doc
        .previous_sibling(table_id)
        .expect("Must have sibling before table");
    let prev_node = doc.get(prev).expect("Node exists");
    assert!(prev_node.is_text());
    assert!(
        prev_node
            .as_text()
            .unwrap()
            .content()
            .contains("Misnested Text")
    );
    assert_eq!(doc.parent(prev), Some(body_id));
}

#[test]
fn test_foster_parenting_element_in_table() {
    let html = "<table><b>Foster Bold</b><tr><td>Cell</td></tr></table>";
    let doc = parse_html(html);

    let body_id = doc.get_elements_by_tag_name("body")[0];
    let table_id = doc.get_elements_by_tag_name("table")[0];
    let b_id = doc.get_elements_by_tag_name("b")[0];

    // The <b> element must be foster-parented before the <table> in <body>
    assert_eq!(doc.parent(b_id), Some(body_id));
    assert_eq!(doc.next_element_sibling(b_id), Some(table_id));
    assert_eq!(doc.text_content(b_id), "Foster Bold");
}

#[test]
fn test_table_caption_and_colgroup() {
    let html = "<table><caption>Table Title</caption><colgroup><col><col></colgroup><tbody><tr><td>Data</td></tr></tbody></table>";
    let doc = parse_html(html);

    let table_id = doc.get_elements_by_tag_name("table")[0];
    let table_elements: Vec<_> = doc.element_children(table_id).collect();

    assert_eq!(table_elements.len(), 3);
    assert_eq!(
        doc.get(table_elements[0])
            .unwrap()
            .as_element()
            .unwrap()
            .tag_name(),
        "caption"
    );
    assert_eq!(
        doc.get(table_elements[1])
            .unwrap()
            .as_element()
            .unwrap()
            .tag_name(),
        "colgroup"
    );
    assert_eq!(
        doc.get(table_elements[2])
            .unwrap()
            .as_element()
            .unwrap()
            .tag_name(),
        "tbody"
    );

    // Inside colgroup, two <col> void elements
    let cols: Vec<_> = doc.element_children(table_elements[1]).collect();
    assert_eq!(cols.len(), 2);
}

#[test]
fn test_rcdata_title_element() {
    let html = "<title>Astra &amp; <b>Browser</b></title>";
    let doc = parse_html(html);

    let title_id = doc.get_elements_by_tag_name("title")[0];
    // In RCDATA, <b> is not a tag, it's text, and &amp; is decoded
    assert_eq!(doc.text_content(title_id), "Astra & <b>Browser</b>");

    // There should be NO <b> element in the document
    let b_elements = doc.get_elements_by_tag_name("b");
    assert_eq!(
        b_elements.len(),
        0,
        "No <b> element should be created from RCDATA"
    );
}

#[test]
fn test_rawtext_style_element() {
    let html = "<style>div > p { content: '&amp;'; }</style>";
    let doc = parse_html(html);

    let style_id = doc.get_elements_by_tag_name("style")[0];
    // In RAWTEXT, entities are NOT decoded
    assert_eq!(doc.text_content(style_id), "div > p { content: '&amp;'; }");

    // There should be NO <p> element created from inside <style>
    let p_elements = doc.get_elements_by_tag_name("p");
    assert_eq!(p_elements.len(), 0);
}

#[test]
fn test_rawtext_script_element() {
    let html = "<script>if (a < b && c > d) { console.log('ok'); }</script>";
    let doc = parse_html(html);

    let script_id = doc.get_elements_by_tag_name("script")[0];
    assert_eq!(
        doc.text_content(script_id),
        "if (a < b && c > d) { console.log('ok'); }"
    );
}

#[test]
fn test_rcdata_textarea_element() {
    let html = "<textarea>Sample &lt;value&gt; with <tag></textarea>";
    let doc = parse_html(html);

    let textarea_id = doc.get_elements_by_tag_name("textarea")[0];
    assert_eq!(doc.text_content(textarea_id), "Sample <value> with <tag>");
    assert_eq!(doc.get_elements_by_tag_name("tag").len(), 0);
}

#[test]
fn test_quirks_mode_detection() {
    // 1. Standards mode
    let doc1 = parse_html("<!DOCTYPE html><html><body></body></html>");
    assert_eq!(doc1.quirks_mode(), QuirksMode::NoQuirks);

    // 2. Quirks mode from missing doctype
    let doc2 = parse_html("<html><body>No Doctype</body></html>");
    assert_eq!(doc2.quirks_mode(), QuirksMode::Quirks);

    // 3. Quirks mode from legacy HTML 4.0 transitional
    let doc3 =
        parse_html("<!DOCTYPE HTML PUBLIC \"-//W3C//DTD HTML 4.0 Transitional//EN\"><html></html>");
    assert_eq!(doc3.quirks_mode(), QuirksMode::Quirks);

    // 4. Limited quirks mode
    let doc4 = parse_html(
        r#"<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd"><html></html>"#,
    );
    assert_eq!(doc4.quirks_mode(), QuirksMode::LimitedQuirks);
}

#[test]
fn test_unclosed_elements_at_eof() {
    let html = "<div><p><span>Unclosed text";
    let doc = parse_html(html);

    let div = doc.get_elements_by_tag_name("div")[0];
    let p = doc.get_elements_by_tag_name("p")[0];
    let span = doc.get_elements_by_tag_name("span")[0];

    assert_eq!(doc.parent(span), Some(p));
    assert_eq!(doc.parent(p), Some(div));
    assert_eq!(doc.text_content(span), "Unclosed text");
}

#[test]
fn test_formatting_element_reconstruction() {
    let html = "<p><b>Bold <i>Italic</b> Still Italic</i></p>";
    let doc = parse_html(html);

    let p = doc.get_elements_by_tag_name("p")[0];
    assert_eq!(doc.text_content(p), "Bold Italic Still Italic");
}
