/*!
HTML scraping library focused on easy to use.

In this library, matching patterns are described as HTML DOM trees.
You can write patterns intuitive and extract desired contents easily.

# Example

```rust
use easy_scraper::Pattern;

let doc = r#"
<!DOCTYPE html>
<html lang="en">
    <body>
        <ul>
            <li>1</li>
            <li>2</li>
            <li>3</li>
        </ul>
    </body>
</html>
"#;

let pat = Pattern::new(r#"
<ul>
    <li>{{foo}}</li>
</ul>
"#).unwrap();

let ms = pat.matches(doc);

assert_eq!(ms.len(), 3);
assert_eq!(ms[0]["foo"], "1");
assert_eq!(ms[1]["foo"], "2");
assert_eq!(ms[2]["foo"], "3");
```

# Syntax

## DOM Tree

DOM trees are valid pattern. You can write placeholders in DOM trees.

```html
<ul>
    <li>{{foo}}</li>
</ul>
```

Patterns are matched if the pattern is subset of document.

If the document is:

```html
<ul>
    <li>1</li>
    <li>2</li>
    <li>3</li>
</ul>
```

there trees are subset of this.

```html
<ul>
    <li>1</li>
</ul>
```

```html
<ul>
    <li>2</li>
</ul>
```

```html
<ul>
    <li>3</li>
</ul>
```

So, match result is

```json
[
    { "foo": "1" },
    { "foo": "2" },
    { "foo": "3" },
]
```

## Child

Child nodes are matched to any descendants
because of subset rule.

For example, this pattern

```html
<div>
    <li>{{id}}</li>
</div>
```

matches against this document.

```html
<div>
    <ul>
        <li>1</li>
    </ul>
</div>
```

## Siblings

To avoid useless matches,
siblings are restricted to match
only consective children of the same parent.

For example, this pattern

```html
<ul>
    <li>{{foo}}</li>
    <li>{{bar}}</li>
</ul>
```

does not match to this document.

```html
<ul>
    <li>123</li>
    <div>
        <li>456</li>
    </div>
</ul>
```

And for this document,

```html
<ul>
    <li>1</li>
    <li>2</li>
    <li>3</li>
</ul>
```

match results are:

```json
[
    { "foo": "1", "bar": "2" },
    { "foo": "2", "bar": "3" },
]
```

`{ "foo": 1, "bar": 3 }` is not contained, because there are not consective children.

You can specify allow nodes between siblings by writing `...` in the pattern.

```html
<ul>
    <li>{{foo}}</li>
    ...
    <li>{{bar}}</li>
</ul>
```

Match result for this pattern is:

```json
[
    { "foo": "1", "bar": "2" },
    { "foo": "1", "bar": "3" },
    { "foo": "2", "bar": "3" },
]
``````

If you want to match siblings as subsequence instead of consective substring,
you can use the `subseq` pattern.

```html
<table>
    <tr><th>AAA</th><td>aaa</td></tr>
    <tr><th>BBB</th><td>bbb</td></tr>
    <tr><th>CCC</th><td>ccc</td></tr>
    <tr><th>DDD</th><td>ddd</td></tr>
    <tr><th>EEE</th><td>eee</td></tr>
</table>
```

For this document,

```html
<table subseq>
    <tr><th>AAA</th><td>{{a}}</td></tr>
    <tr><th>BBB</th><td>{{b}}</td></tr>
    <tr><th>DDD</th><td>{{d}}</td></tr>
</table>
```

this pattern matches.

```json
[
    {
        "a": "aaa",
        "b": "bbb",
        "d": "ddd"
    }
]
```


## Attribute

You can specify attributes in patterns.
Attribute patterns match when pattern's attributes are subset of document's attributes.

This pattern

```html
<div class="attr1">
    {{foo}}
</div>
```

matches to this document.

```html
<div class="attr1 attr2">
    Hello
</div>
```

You can also write placeholders in attributes.

```html
<a href="{{url}}">{{title}}</a>
```

Match result for

```html
<a href="https://www.google.com">Google</a>
<a href="https://www.yahoo.com">Yahoo</a>
```

this document is:

```json
[
    { "url": "https://www.google.com", "title": "Google" },
    { "url": "https://www.yahoo.com", "title": "Yahoo" },
]
```

## Partial text-node pattern

You can write placeholders arbitrary positions in text-node.

```html
<ul>
    <li>A: {{a}}, B: {{b}}</li>
</ul>
```

Match result for

```html
<ul>
    <li>A: 1, B: 2</li>
    <li>A: 3, B: 4</li>
    <li>A: 5, B: 6</li>
</ul>
```

this document is:

```json
[
    { "a": "1",  "b": "2" },
    { "a": "3",  "b": "4" },
    { "a": "5",  "b": "6" },
]
```

You can also write placeholders in atteibutes position.

```html
<ul>
    <a href="/users/{{userid}}">{{username}}</a>
</ul>
```

Match result for

```html
<ul>
    <a href="/users/foo">Foo</a>
    <a href="/users/bar">Bar</a>
    <a href="/users/baz">Baz</a>
</ul>
```

this document is:

```json
[
    { "userid": "foo",  "username": "Foo" },
    { "userid": "bar",  "username": "Bar" },
    { "userid": "baz",  "username": "Baz" },
]
```

## Whole subtree pattern

The pattern `{{var:*}}` matches to whole sub-tree as string.

```html
<div>{{body:*}}</div>
```

Match result for

```html
<body>
    Hello
    <span>hoge</span>
    World
</body>
```

this document is:

```json
[
    { "body": "Hello<span>hoge</span>World" }
]
```

## White-space

White-space are ignored almost all positions.

# Restrictions

* Whole sub-tree patterns must be the only one element of the parent node.

This is valid:

```html
<div>
    {{foo:*}}
</div>
```

There are invalid:

```html
<div>
    hoge {{foo:*}}
</div>
```

```html
<ul>
    <li></li>
    {{foo:*}}
    <li></li>
<ul>
```
*/

use kuchiki::{parse_html, parse_html_with_options, Attributes, NodeRef, ParseOpts};
use kuchiki::{traits::*, ExpandedName};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Deref;
use std::rc::Rc;

/// Pattern for matching HTML document
///
/// # Example
///
/// ```
/// use easy_scraper::Pattern;
///
/// let pat = Pattern::new(r#"
/// <ul>
///     <li>{{hoge}}</li>
/// </ul>
/// "#).unwrap();
///
/// let ms = pat.matches(r#"
/// <!DOCTYPE html>
/// <html lang="en">
///     <body>
///         <ul>
///             <li>1</li>
///             <li>2</li>
///             <li>3</li>
///         </ul>
///     </body>
/// </html>
/// "#);
///
/// assert_eq!(ms.len(), 3);
/// assert_eq!(ms[0]["hoge"], "1");
/// assert_eq!(ms[1]["hoge"], "2");
/// assert_eq!(ms[2]["hoge"], "3");
/// ```
///
pub struct Pattern(NodeRef);

impl Pattern {
    /// Build pattern
    pub fn new(pattern_str: &str) -> Result<Pattern, String> {
        let doc = filter_whitespace(parse_html_strict(pattern_str)?).unwrap();
        Ok(Pattern(doc))
    }

    /// Match HTML document to pattern
    /// Returns all matches.
    pub fn matches(&self, html: &str) -> Vec<BTreeMap<String, String>> {
        let doc = filter_whitespace(parse_html().one(html)).unwrap();
        match_subtree(&doc, &self.0, false)
    }
}

fn parse_html_strict(s: &str) -> Result<NodeRef, String> {
    let errs = Rc::new(RefCell::new(vec![]));
    let ret = {
        let mut opts = ParseOpts::default();
        let errs = Rc::clone(&errs);
        opts.on_parse_error = Some(Box::new(move |err| {
            // eprintln!("HTML parse error: {:?}", err);
            if err != "Unexpected token" {
                errs.borrow_mut().push(err.to_string())
            }
        }));
        parse_html_with_options(opts).one(s)
    };
    let errs = errs.deref().borrow().clone();
    let mut m = BTreeSet::new();
    let mut errs_uniq = vec![];
    for err in errs {
        if !m.contains(&err) {
            m.insert(err.clone());
            errs_uniq.push(err);
        }
    }

    if errs_uniq.is_empty() {
        Ok(ret)
    } else {
        Err(errs_uniq.join(", "))
    }
}

fn match_subtree(doc: &NodeRef, pattern: &NodeRef, exact: bool) -> Vec<BTreeMap<String, String>> {
    let mut ret = vec![];

    if let (Some(_), Some(_)) = (doc.as_doctype(), pattern.as_doctype()) {
        let doc_cs = doc.children().collect::<Vec<_>>();
        let pat_cs = pattern.children().collect::<Vec<_>>();
        ret.append(&mut match_siblings(&doc_cs, &pat_cs, false));
    }

    if let (Some(_), Some(_)) = (doc.as_document(), pattern.as_document()) {
        let doc_cs = doc.children().collect::<Vec<_>>();
        let pat_cs = pattern.children().collect::<Vec<_>>();
        ret.append(&mut match_siblings(&doc_cs, &pat_cs, false));
    }

    if let (Some(e1), Some(e2)) = (doc.as_element(), pattern.as_element()) {
        if e1.name == e2.name {
            if let Some(m1) = match_attributes(
                e1.attributes.borrow().deref(),
                e2.attributes.borrow().deref(),
            ) {
                let subseq = e2
                    .attributes
                    .borrow()
                    .deref()
                    .map
                    .keys()
                    .any(|k| k.local.as_ref() == "subseq");

                let doc_cs = doc.children().collect::<Vec<_>>();
                let pat_cs = pattern.children().collect::<Vec<_>>();

                // FIXME: this is hack for auto completion of <tbody> tag.
                let pat_cs = if e2.name.local.as_ref() == "table"
                    && pat_cs.len() == 1
                    && pat_cs[0].as_element().map(|r| r.name.local.as_ref()) == Some("tbody")
                {
                    pat_cs[0].children().collect::<Vec<_>>()
                } else {
                    pat_cs
                };

                let m2 = match_siblings(&doc_cs, &pat_cs, subseq);

                ret.append(&mut map_product(vec![m1], m2));
            }
        }
    }

    if let Some(pat_text) = pattern.as_text() {
        if let Some(var) = is_var(pat_text.borrow().as_ref()) {
            assert!(!var.whole);

            if let Some(doc_text) = doc.as_text() {
                return vec![singleton(var.name, doc_text.borrow().trim().to_owned())];
            }

            return vec![];
        }

        if let Some(doc_text) = doc.as_text() {
            if let Some(m) = match_text(doc_text.borrow().trim(), pat_text.borrow().trim()) {
                return vec![m];
            }
        }

        // Do not search recursive text pattern.
        return vec![];
    }

    if !exact {
        for doc_child in doc.children() {
            ret.append(&mut match_subtree(&doc_child, pattern, false));
        }
    }

    ret
}

fn match_siblings(
    doc: &[NodeRef],
    pattern: &[NodeRef],
    subseq: bool,
) -> Vec<BTreeMap<String, String>> {
    if pattern.is_empty() {
        return vec![BTreeMap::new()];
    }

    if doc.is_empty() {
        return vec![];
    }

    // special case: if `pattern` is whole variable, all `doc` nodes matches
    if pattern.len() == 1 {
        if let Some(pat_text) = pattern[0].as_text() {
            if let Some(var) = is_var(pat_text.borrow().as_ref()) {
                if var.whole {
                    let texts = doc.iter().map(|r| r.to_string()).collect::<Vec<_>>();
                    return vec![singleton(var.name, texts.concat())];
                }
            }
        }
    }

    let mut ret = vec![];

    // 1. `pattern` nodes match consective element of `doc`
    if subseq {
        ret.append(&mut match_siblings_direct(&doc[..], pattern, subseq));
    } else {
        for i in 0..doc.len() {
            ret.append(&mut match_siblings_direct(&doc[i..], pattern, subseq));
        }
    }

    // 2. all `pattern` nodes are contained in the one `doc` node
    for d in doc.iter() {
        ret.append(&mut match_descendants(d, pattern, subseq));
    }

    ret
}

// Matches two siblings.
// * `subseq` - If true, check if `pattern` is subsequence of `doc`.
// Otherwise, check if `pattern` is substring of `doc`.
fn match_siblings_direct(
    doc: &[NodeRef],
    pattern: &[NodeRef],
    subseq: bool,
) -> Vec<BTreeMap<String, String>> {
    let non_skip_len = pattern
        .iter()
        .filter(|r| {
            if let Some(text) = r.as_text() {
                !is_skip(text.borrow().as_ref())
            } else {
                true
            }
        })
        .count();

    if non_skip_len == 0 {
        return vec![BTreeMap::new()];
    }

    if non_skip_len > doc.len() {
        return vec![];
    }

    if let Some(text) = pattern[0].as_text() {
        if is_skip(text.borrow().as_ref()) {
            let mut ret = vec![];
            for i in 0..doc.len() {
                ret.append(&mut match_siblings_direct(&doc[i..], &pattern[1..], subseq));
            }
            return ret;
        }
    }

    let a = match_subtree(&doc[0], &pattern[0], true);

    let mut ret = if !a.is_empty() {
        map_product(a, match_siblings_direct(&doc[1..], &pattern[1..], subseq))
    } else {
        vec![]
    };

    if subseq {
        ret.append(&mut match_siblings_direct(&doc[1..], pattern, subseq));
    }

    ret
}

fn match_descendants(
    doc: &NodeRef,
    pattern: &[NodeRef],
    subseq: bool,
) -> Vec<BTreeMap<String, String>> {
    if pattern.is_empty() {
        return vec![BTreeMap::new()];
    }

    let mut ret = vec![];
    let cs = doc.children().collect::<Vec<_>>();
    ret.append(&mut match_siblings(&cs, pattern, subseq));
    ret
}

fn match_text(doc: &str, pat: &str) -> Option<BTreeMap<String, String>> {
    if pat.find("{{").is_some() && pat.find("}}").is_some() {
        // FIXME: cache regex
        let mut re_str = String::new();

        re_str += "^";

        let mut vars = vec![];

        let mut cur = pat;

        while let Some(ix) = cur.find("{{") {
            re_str += &cur[0..ix];
            cur = &cur[ix + 2..];
            let close = cur.find("}}");
            assert!(close.is_some(), "Invalid text pattern: \"{}\"", pat);
            let close = close.unwrap();
            vars.push(&cur[0..close]);
            re_str += "(.*)";
            cur = &cur[close + 2..];
        }

        re_str += cur;
        re_str += "$";

        let re = regex::Regex::new(&re_str).unwrap();

        if let Some(caps) = re.captures(doc) {
            let mut ret = BTreeMap::new();
            for i in 0..vars.len() {
                ret.insert(vars[i].to_owned(), caps[i + 1].to_string());
            }
            Some(ret)
        } else {
            None
        }
    } else {
        if doc == pat {
            Some(BTreeMap::new())
        } else {
            None
        }
    }
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

struct Variable {
    name: String,
    whole: bool,
}

fn is_var(s: &str) -> Option<Variable> {
    let s = s.trim();
    if s.starts_with("{{") && s.ends_with("}}") {
        let var = &s[2..s.len() - 2];
        let mut it = var.split(':');
        let var = it.next()?;

        if let Some(qual) = it.next() {
            if qual == "*" {
                Some(Variable {
                    name: var.to_owned(),
                    whole: true,
                })
            } else {
                None
            }
        } else {
            Some(Variable {
                name: var.to_owned(),
                whole: false,
            })
        }
    } else {
        None
    }
}

fn is_skip(s: &str) -> bool {
    s.trim() == "..."
}

fn is_special_attr(n: &ExpandedName) -> bool {
    let s = n.local.as_ref();
    s == "subseq"
}

fn singleton(key: String, val: String) -> BTreeMap<String, String> {
    let mut ret = BTreeMap::new();
    ret.insert(key, val);
    ret
}

fn match_attributes(a1: &Attributes, a2: &Attributes) -> Option<BTreeMap<String, String>> {
    let a1 = &a1.map;
    let a2 = &a2.map;

    let mut ret = BTreeMap::new();

    for (k2, v2) in a2.iter() {
        if is_special_attr(k2) {
            continue;
        }

        if let Some(v1) = a1.get(k2) {
            if let Some(var) = is_var(&v2.value) {
                // Simple variable
                assert!(!var.whole);
                ret.insert(var.name, v1.value.trim().to_owned());
            } else if {
                let x = v2.value.find("{{");
                let y = v2.value.find("}}");
                x.is_some() && y.is_some() && x < y
            } {
                // Complex pattern
                let t = match_text(&v1.value, &v2.value);
                if t.is_none() {
                    return None;
                }
                ret.append(&mut t.unwrap())
            } else if !is_subset(&v1.value, &v2.value) {
                // Set of attribute
                return None;
            }
        } else {
            return None;
        }
    }

    Some(ret)
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
    )
    .unwrap();

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
    )
    .unwrap();

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

    let pat = Pattern::new(r#"<div>{{foo}}</div>"#).unwrap();
    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 1);
    assert_eq!(ms[0]["foo"], "hello");

    let pat = Pattern::new(r#"<div class="">{{foo}}</div>"#).unwrap();
    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 1);
    assert_eq!(ms[0]["foo"], "hello");

    let pat = Pattern::new(r#"<div class="foo">{{foo}}</div>"#).unwrap();
    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 1);
    assert_eq!(ms[0]["foo"], "hello");

    let pat = Pattern::new(r#"<div class="foo bar">{{foo}}</div>"#).unwrap();
    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 1);
    assert_eq!(ms[0]["foo"], "hello");

    let pat = Pattern::new(r#"<div class="foo bar baz">{{foo}}</div>"#).unwrap();
    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 1);
    assert_eq!(ms[0]["foo"], "hello");

    let pat = Pattern::new(r#"<div class="baz foo bar">{{foo}}</div>"#).unwrap();
    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 1);
    assert_eq!(ms[0]["foo"], "hello");

    let pat = Pattern::new(r#"<div class="hoge">{{foo}}</div>"#).unwrap();
    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 0);

    let pat = Pattern::new(r#"<div id="">{{foo}}</div>"#).unwrap();
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

    let pat = Pattern::new(r#"<a href="{{url}}">{{link}}</a>"#).unwrap();
    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 2);
    assert_eq!(ms[0]["url"], "https://www.google.com");
    assert_eq!(ms[0]["link"], "Google");
    assert_eq!(ms[1]["url"], "https://github.com");
    assert_eq!(ms[1]["link"], "GitHub");
}

#[test]
fn test_skip() {
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
    ...
    <li>{{moge}}</li>
</ul>
"#,
    )
    .unwrap();

    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 3);
    assert_eq!(ms[0]["hoge"], "1");
    assert_eq!(ms[0]["moge"], "2");
    assert_eq!(ms[1]["hoge"], "1");
    assert_eq!(ms[1]["moge"], "3");
    assert_eq!(ms[2]["hoge"], "2");
    assert_eq!(ms[2]["moge"], "3");
}

#[test]
fn test_all_match() {
    let doc = r#"
<!DOCTYPE html>
<html lang="en">
    <head>
    </head>
    <body>
        Hello
        <span>hoge</span>
        World
    </body>
</html>
"#;

    let pat = Pattern::new(r#"<body>{{body:*}}</body>"#).unwrap();

    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 1);
    assert_eq!(ms[0]["body"], "Hello<span>hoge</span>World");
}

#[test]
fn test_partial() {
    let doc = r#"
<!DOCTYPE html>
<html lang="en">
    <head>
    </head>
    <body>
        <ul>
            <li>Test 1, 2</li>
            <li>Test 3, 4</li>
            <li>Test 5, 6</li>
        </ul>
    </body>
</html>
"#;

    let pat = Pattern::new(r#"<ul>Test {{foo}}, {{bar}}</ul>"#).unwrap();

    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 3);
    assert_eq!(ms[0]["foo"], "1");
    assert_eq!(ms[0]["bar"], "2");
    assert_eq!(ms[1]["foo"], "3");
    assert_eq!(ms[1]["bar"], "4");
    assert_eq!(ms[2]["foo"], "5");
    assert_eq!(ms[2]["bar"], "6");
}

#[test]
fn test_attr_partial() {
    let doc = r#"
<!DOCTYPE html>
<html lang="en">
    <head>
    </head>
    <body>
        <a href="/users/foo/info"></a>
        <a href="/users/bar/info"></a>
        <a href="/users/baz/info"></a>
    </body>
</html>
"#;

    let pat = Pattern::new(r#"<a href="/users/{{user}}/info"></a>"#).unwrap();

    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 3);
    assert_eq!(ms[0]["user"], "foo");
    assert_eq!(ms[1]["user"], "bar");
    assert_eq!(ms[2]["user"], "baz");
}

#[test]
fn test_table_skip() {
    let doc = r#"
<!DOCTYPE html>
<html lang="en">
    <head>
    </head>
    <body>
        <table>
            <tr><th>AAA</th><td>aaa</td></tr>
            <tr><th>BBB</th><td>bbb</td></tr>
            <tr><th>CCC</th><td>ccc</td></tr>
            <tr><th>DDD</th><td>ddd</td></tr>
            <tr><th>EEE</th><td>eee</td></tr>
        </table>
    </body>
</html>
"#;

    let pat = Pattern::new(
        r#"
<table subseq>
    <tr><th>AAA</th><td>{{a}}</td></tr>
    <tr><th>BBB</th><td>{{b}}</td></tr>
    <tr><th>DDD</th><td>{{d}}</td></tr>
</table>
"#,
    )
    .unwrap();

    let ms = pat.matches(doc);
    assert_eq!(ms.len(), 1);
    assert_eq!(ms[0]["a"], "aaa");
    assert_eq!(ms[0]["b"], "bbb");
    assert_eq!(ms[0]["d"], "ddd");
}
