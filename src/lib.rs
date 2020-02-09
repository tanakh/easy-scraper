use html5ever::tendril::TendrilSink;
use kuchiki::NodeRef;
use std::collections::BTreeMap;

pub fn matches(doc: &NodeRef, pattern: &NodeRef) -> Vec<BTreeMap<String, String>> {
    todo!()
}

#[test]
fn test() {
    use kuchiki::parse_html;

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
    <li><variable name="hoge"></variable></li>
</ul>
"#,
    );

    dbg!(&doc.to_string());
    dbg!(&pat.to_string());
}
