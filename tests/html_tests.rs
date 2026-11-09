use astra_html::{ParserOptions, Token, Tokenizer, parse_html, parse_html_with_options};

#[test]
fn test_tokenizer_tokens() {
    let html = r#"<!DOCTYPE html><div id="main" class='hero' data-flag disabled><!-- notice -->Hello &amp; welcome<br/></div>"#;
    let mut tokenizer = Tokenizer::new(html);
    let tokens = tokenizer.tokenize_all();

    assert!(matches!(&tokens[0], Token::Doctype(d) if d.name == "html"));
    assert!(matches!(&tokens[1], Token::StartTag(s) if s.name == "div" && s.attributes.len() == 4));
    assert!(matches!(&tokens[2], Token::Comment(c) if c == " notice "));
    assert!(matches!(&tokens[3], Token::Text(t) if t == "Hello & welcome"));
    assert!(matches!(&tokens[4], Token::StartTag(s) if s.name == "br" && s.self_closing));
    assert!(matches!(&tokens[5], Token::EndTag(e) if e.name == "div"));
    assert!(matches!(&tokens[6], Token::Eof));
}

#[test]
fn test_simple_html_document() {
    let html = "<html><body><div>Hello</div></body></html>";
    let doc = parse_html(html);

    let divs = doc.get_elements_by_tag_name("div");
    assert_eq!(divs.len(), 1);
    assert_eq!(doc.text_content(divs[0]), "Hello");
}

#[test]
fn test_nested_elements() {
    let html = "<div id=\"l1\"><section id=\"l2\"><article id=\"l3\"><span>Deep</span></article></section></div>";
    let doc = parse_html(html);

    let l1 = doc.get_element_by_id("l1").expect("l1 exists");
    let l2 = doc.get_element_by_id("l2").expect("l2 exists");
    let l3 = doc.get_element_by_id("l3").expect("l3 exists");
    let spans = doc.get_elements_by_tag_name("span");

    assert_eq!(doc.parent(l3), Some(l2));
    assert_eq!(doc.parent(l2), Some(l1));
    assert_eq!(spans.len(), 1);
    assert_eq!(doc.parent(spans[0]), Some(l3));
    assert_eq!(doc.text_content(l1), "Deep");
}

#[test]
fn test_multiple_attributes_styles() {
    let html =
        r#"<button type="submit" class='btn primary' data-action=save disabled required></button>"#;
    let doc = parse_html(html);

    let buttons = doc.get_elements_by_tag_name("button");
    assert_eq!(buttons.len(), 1);

    let btn = doc.get(buttons[0]).unwrap().as_element().unwrap();
    assert_eq!(btn.get_attribute("type"), Some("submit"));
    assert_eq!(btn.get_attribute("class"), Some("btn primary"));
    assert_eq!(btn.get_attribute("data-action"), Some("save"));
    assert_eq!(btn.get_attribute("disabled"), Some(""));
    assert_eq!(btn.get_attribute("required"), Some(""));
    assert!(btn.has_attribute("disabled"));
    assert!(btn.has_attribute("required"));
    assert!(!btn.has_attribute("nonexistent"));
}

#[test]
fn test_text_nodes_and_entity_decoding() {
    let html = "<p>5 &lt; 10 &amp; 10 &gt; 5 &quot;quoted&quot; &apos;single&apos; &#65;</p>";
    let doc = parse_html(html);

    let p = doc.get_elements_by_tag_name("p")[0];
    assert_eq!(doc.text_content(p), "5 < 10 & 10 > 5 \"quoted\" 'single' A");
}

#[test]
fn test_comments() {
    let html = "<div><!-- This is a comment --><span>Content</span><!-- Another comment --></div>";
    let doc = parse_html(html);

    let div = doc.get_elements_by_tag_name("div")[0];
    let children: Vec<_> = doc.children(div).collect();
    assert_eq!(children.len(), 3);

    let comment1 = doc.get(children[0]).unwrap().as_comment().unwrap();
    assert_eq!(comment1.content(), " This is a comment ");

    let span = doc.get(children[1]).unwrap().as_element().unwrap();
    assert_eq!(span.tag_name(), "span");

    let comment2 = doc.get(children[2]).unwrap().as_comment().unwrap();
    assert_eq!(comment2.content(), " Another comment ");
}

#[test]
fn test_doctype() {
    let html = "<!DOCTYPE html SYSTEM \"about:legacy-compat\"><html></html>";
    let doc = parse_html(html);

    let first = doc.first_child(doc.root()).unwrap();
    let node = doc.get(first).unwrap();
    assert!(node.is_doctype());
    let dt = node.as_doctype().unwrap();
    assert_eq!(dt.name(), "html");
    assert_eq!(dt.system_id(), Some("about:legacy-compat"));
}

#[test]
fn test_void_elements_do_not_nest() {
    let html = "<div><img src=\"img.png\"><br><input type=\"text\"><hr><p>Next paragraph</p></div>";
    let doc = parse_html(html);

    let div = doc.get_elements_by_tag_name("div")[0];
    let children: Vec<_> = doc.element_children(div).collect();

    // The void elements should be direct siblings under <div>, not nested inside each other!
    assert_eq!(children.len(), 5);

    let tag_names: Vec<&str> = children
        .iter()
        .map(|&id| doc.get(id).unwrap().as_element().unwrap().tag_name())
        .collect();
    assert_eq!(tag_names, vec!["img", "br", "input", "hr", "p"]);
}

#[test]
fn test_malformed_html_recovery_unclosed_tags() {
    let html = "<div><p>Paragraph without closing tag<span>Inline text</div>";
    let doc = parse_html(html);

    let div = doc.get_elements_by_tag_name("div")[0];
    let p = doc.get_elements_by_tag_name("p")[0];
    let span = doc.get_elements_by_tag_name("span")[0];

    // <div> should properly contain <p>, and <p> should contain <span>
    assert_eq!(doc.parent(p), Some(div));
    assert_eq!(doc.parent(span), Some(p));
}

#[test]
fn test_malformed_html_recovery_stray_closing_tags() {
    let html = "<div></span></p><article>Content</article></div>";
    let doc = parse_html(html);

    let div = doc.get_elements_by_tag_name("div")[0];
    let article = doc.get_elements_by_tag_name("article")[0];

    // Stray </span> and </p> should be ignored, <article> should be a child of <div>
    assert_eq!(doc.parent(article), Some(div));
    assert_eq!(doc.text_content(article), "Content");
}

#[test]
fn test_whitespace_filtering_option() {
    let html = "<div>\n    <span>Hello</span>\n</div>";
    let doc_clean = parse_html_with_options(
        html,
        ParserOptions {
            preserve_whitespace_only_text: false,
        },
    );

    let div = doc_clean.get_elements_by_tag_name("div")[0];
    // Under div, there should be ONLY the <span> child, no newline text nodes!
    let children: Vec<_> = doc_clean.children(div).collect();
    assert_eq!(children.len(), 1);
    assert_eq!(
        doc_clean
            .get(children[0])
            .unwrap()
            .as_element()
            .unwrap()
            .tag_name(),
        "span"
    );
}
