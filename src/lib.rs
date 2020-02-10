use kuchiki::traits::*;
use kuchiki::{parse_html, Attributes, NodeRef};
use std::collections::BTreeMap;
use std::iter::FromIterator;
use std::ops::Deref;

pub struct Pattern(NodeRef);

impl Pattern {
    pub fn new(s: &str) -> Pattern {
        let doc = filter_whitespace(parse_html().one(s)).unwrap();
        // dbg!(doc.to_string());
        Pattern(doc)
    }

    pub fn matches(&self, s: &str) -> Vec<BTreeMap<String, String>> {
        let doc = filter_whitespace(parse_html().one(s)).unwrap();
        // dbg!(doc.to_string());
        match_subtree(&doc, &self.0, false)
    }
}

fn match_subtree(doc: &NodeRef, pattern: &NodeRef, exact: bool) -> Vec<BTreeMap<String, String>> {
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
        if e1.name == e2.name {
            if let Some(m1) = match_attributes(
                e1.attributes.borrow().deref(),
                e2.attributes.borrow().deref(),
            ) {
                let doc_cs = doc.children().collect::<Vec<_>>();
                let pat_cs = pattern.children().collect::<Vec<_>>();
                let m2 = match_siblings(&doc_cs, &pat_cs);
                ret.append(&mut map_product(vec![m1], m2));
            }
        }
    }

    if let (Some(doc_text), Some(pat_text)) = (doc.as_text(), pattern.as_text()) {
        if let Some(var) = is_var(pat_text.borrow().as_ref()) {
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

    // 1. `pattern` nodes match consective element of `doc`
    for window in doc.windows(pattern.len()) {
        let mut t = vec![BTreeMap::<String, String>::new()];
        for (d, p) in window.iter().zip(pattern.iter()) {
            t = map_product(t, match_subtree(d, p, true));
        }
        ret.append(&mut t);
    }

    // 2. all `pattern` nodes are contained in the one `doc` node
    for d in doc.iter() {
        ret.append(&mut match_descendants(d, pattern));
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

fn match_attributes(a1: &Attributes, a2: &Attributes) -> Option<BTreeMap<String, String>> {
    let a1 = &a1.map;
    let a2 = &a2.map;

    for (k2, v2) in a2.iter() {
        if let Some(v1) = a1.get(k2) {
            if let Some(var) = is_var(&v2.value) {
                return Some(singleton(var, v1.value.trim().to_owned()));
            } else if !is_subset(&v1.value, &v2.value) {
                return None;
            }
        } else {
            return None;
        }
    }

    Some(BTreeMap::new())
}

fn singleton(key: String, val: String) -> BTreeMap<String, String> {
    let mut ret = BTreeMap::new();
    ret.insert(key, val);
    ret
}

fn is_subset(s1: &str, s2: &str) -> bool {
    let ws1 = s1.split_whitespace().collect::<Vec<_>>();
    for w in s2.split_whitespace() {
        if !ws1.contains(&w) {
            return false;
        }
    }
    true
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
fn test_basic() {
    let doc = r#"
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
"#;

    let pat = Pattern::new(
        r#"
<ul>
    <li>{{hoge}}</li>
</ul>
"#,
    );

    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 3);
    assert_eq!(ms[0]["hoge"], "1");
    assert_eq!(ms[1]["hoge"], "2");
    assert_eq!(ms[2]["hoge"], "3");

    let pat = Pattern::new(
        r#"
<ul>
    <li>{{hoge}}</li>
    <li>{{moge}}</li>
</ul>
"#,
    );

    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 2);
    assert_eq!(ms[0]["hoge"], "1");
    assert_eq!(ms[0]["moge"], "2");
    assert_eq!(ms[1]["hoge"], "2");
    assert_eq!(ms[1]["moge"], "3");
}

#[test]
fn test_attribute() {
    let doc = r#"
<!DOCTYPE html>
<html lang="en">
    <head>
    </head>
    <body>
        <div class="foo bar baz">
            hello
        </div>
    </body>
</html>
"#;

    let pat = Pattern::new(r#"<div>{{foo}}</div>"#);
    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 1);
    assert_eq!(ms[0]["foo"], "hello");

    let pat = Pattern::new(r#"<div class="">{{foo}}</div>"#);
    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 1);
    assert_eq!(ms[0]["foo"], "hello");

    let pat = Pattern::new(r#"<div class="foo">{{foo}}</div>"#);
    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 1);
    assert_eq!(ms[0]["foo"], "hello");

    let pat = Pattern::new(r#"<div class="foo bar">{{foo}}</div>"#);
    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 1);
    assert_eq!(ms[0]["foo"], "hello");

    let pat = Pattern::new(r#"<div class="foo bar baz">{{foo}}</div>"#);
    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 1);
    assert_eq!(ms[0]["foo"], "hello");

    let pat = Pattern::new(r#"<div class="baz foo bar">{{foo}}</div>"#);
    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 1);
    assert_eq!(ms[0]["foo"], "hello");

    let pat = Pattern::new(r#"<div class="hoge">{{foo}}</div>"#);
    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 0);

    let pat = Pattern::new(r#"<div id="">{{foo}}</div>"#);
    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 0);
}

#[test]
fn test_attribute_pattern() {
    let doc = r#"
<!DOCTYPE html>
<html lang="en">
    <head>
    </head>
    <body>
        <a href="https://www.google.com">Google</a>
        <p>
            <a href="https://github.com">GitHub</a>
        </p>
    </body>
</html>
"#;

    let pat = Pattern::new(r#"<a href="{{url}}">{{link}}</a>"#);
    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 2);
    assert_eq!(ms[0]["url"], "https://www.google.com");
    assert_eq!(ms[0]["link"], "Google");
    assert_eq!(ms[1]["url"], "https://github.com");
    assert_eq!(ms[1]["link"], "GitHub");
}
