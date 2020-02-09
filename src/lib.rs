use kuchiki::parse_html;
use kuchiki::traits::*;
use kuchiki::NodeRef;
use std::collections::BTreeMap;
use std::iter::FromIterator;

pub struct Pattern(NodeRef);

impl Pattern {
    pub fn new(s: &str) -> Pattern {
        let doc = filter_whitespace(parse_html().one(s)).unwrap();
        Pattern(doc)
    }

    pub fn matches(&self, s: &str) -> Vec<BTreeMap<String, String>> {
        let doc = filter_whitespace(parse_html().one(s)).unwrap();
        match_subtree(&doc, &self.0, false)
    }
}

pub fn match_subtree(
    doc: &NodeRef,
    pattern: &NodeRef,
    exact: bool,
) -> Vec<BTreeMap<String, String>> {
    let mut ret = vec![];

    if let (Some(_), Some(_)) = (doc.as_doctype(), pattern.as_doctype()) {
        let doc_cs = doc.children().collect::<Vec<_>>();
        let pat_cs = pattern.children().collect::<Vec<_>>();
        ret.append(&mut match_siblings(&doc_cs, &pat_cs));
    }

    if let (Some(_), Some(_)) = (doc.as_document(), pattern.as_document()) {
        let doc_cs = doc.children().collect::<Vec<_>>();
        let pat_cs = pattern.children().collect::<Vec<_>>();
        ret.append(&mut match_siblings(&doc_cs, &pat_cs));
    }

    if let (Some(e1), Some(e2)) = (doc.as_element(), pattern.as_element()) {
        // TODO: check attribute
        if e1.name == e2.name {
            let doc_cs = doc.children().collect::<Vec<_>>();
            let pat_cs = pattern.children().collect::<Vec<_>>();
            ret.append(&mut match_siblings(&doc_cs, &pat_cs));
        }
    }

    if let (Some(doc_text), Some(pat_text)) = (doc.as_text(), pattern.as_text()) {
        if let Some(var) = is_var(pat_text.borrow().as_ref()) {
            // dbg!(&var, doc_text.borrow().trim());
            ret.push(BTreeMap::from_iter(vec![(
                var,
                doc_text.borrow().trim().to_owned(),
            )]));
        } else if doc_text.borrow().trim() == pat_text.borrow().trim() {
            ret.push(BTreeMap::new());
        }
    }

    // Do not search recursive text pattern.
    if let Some(_) = pattern.as_text() {
        return ret;
    }

    if !exact {
        for doc_child in doc.children() {
            ret.append(&mut match_subtree(&doc_child, pattern, false));
        }
    }

    ret
}

fn match_siblings(doc: &[NodeRef], pattern: &[NodeRef]) -> Vec<BTreeMap<String, String>> {
    if pattern.is_empty() {
        return vec![BTreeMap::new()];
    }

    if doc.is_empty() {
        return vec![];
    }

    let mut ret = vec![];

    // 1. all pattern is contained in one doc node

    for d in doc.iter() {
        ret.append(&mut match_descendants(d, pattern));
    }

    // 2. patterns maches consective element of doc

    for window in doc.windows(pattern.len()) {
        let mut t = vec![BTreeMap::<String, String>::new()];
        for (d, p) in window.iter().zip(pattern.iter()) {
            t = map_product(t, match_subtree(d, p, true));
        }
        ret.append(&mut t);
    }

    ret
}

fn match_descendants(doc: &NodeRef, pattern: &[NodeRef]) -> Vec<BTreeMap<String, String>> {
    if pattern.is_empty() {
        return vec![BTreeMap::new()];
    }

    let mut ret = vec![];
    let cs = doc.children().collect::<Vec<_>>();
    ret.append(&mut match_siblings(&cs, pattern));
    ret
}

fn map_product(
    a: Vec<BTreeMap<String, String>>,
    b: Vec<BTreeMap<String, String>>,
) -> Vec<BTreeMap<String, String>> {
    let mut ret = vec![];
    for a in a {
        for b in b.iter() {
            let mut a = a.clone();
            a.append(&mut b.clone());
            ret.push(a);
        }
    }
    ret
}

fn is_var(s: &str) -> Option<String> {
    let s = s.trim();
    if s.starts_with("{{") && s.ends_with("}}") {
        Some(s[2..s.len() - 2].to_owned())
    } else {
        None
    }
}

fn filter_whitespace(node: NodeRef) -> Option<NodeRef> {
    if let Some(dt) = node.as_doctype() {
        assert!(node.first_child().is_none());

        Some(NodeRef::new_doctype(&dt.name, &dt.public_id, &dt.system_id))
    } else if let Some(_) = node.as_document() {
        let ret = NodeRef::new_document();
        for child in node.children() {
            if let Some(child) = filter_whitespace(child) {
                ret.append(child);
            }
        }
        Some(ret)
    } else if let Some(element) = node.as_element() {
        let ret = NodeRef::new_element(
            element.name.clone(),
            element.attributes.borrow().map.clone(),
        );

        for child in node.children() {
            if let Some(child) = filter_whitespace(child) {
                ret.append(child);
            }
        }

        Some(ret)
    } else if let Some(text) = node.as_text() {
        assert!(node.first_child().is_none());

        let text = text.borrow();
        let text = text.trim();

        if text == "" {
            None
        } else {
            Some(NodeRef::new_text(text.to_owned()))
        }
    } else if let Some(_) = node.as_comment() {
        assert!(node.first_child().is_none());
        None
    } else {
        unreachable!()
    }
}

#[test]
fn test() {
    let doc = parse_html().one(
        r#"
<!DOCTYPE html>
<html lang="en">
    <head>
    </head>
    <body>
        <ul>
            <li>1</li>
            <li>2</li>
            <li>3</li>
        </ul>
    </body>
</html>
"#,
    );

    let pat = parse_html().one(
        r#"
<ul>
    <li>{{hoge}}</li>
</ul>
"#,
    );

    dbg!(&doc.to_string());
    dbg!(&pat);

    let pat = filter_whitespace(pat).unwrap();
    dbg!(pat.to_string());

    let ms = match_subtree(&doc, &pat, false);
    dbg!(&ms);
}

#[test]
fn test_hoge() {
    let doc = include_str!("../hoge.html");

    let pat = Pattern::new(
        r#"
<div>
    <section>
        <h3>{{input-name}}</h3>
        <pre>{{input}}</pre>
    </section>
</div>
<div>
    <section>
        <h3>{{output-name}}</h3>
        <pre>{{output}}</pre>
    </section>
</div>
"#,
    );

    // let pat = filter_whitespace(pat).unwrap();
    // dbg!(pat.to_string());
    // dbg!(matches(&doc, &pat));

    dbg!(pat.matches(doc));
}
