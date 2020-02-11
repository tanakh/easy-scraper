# easy-scraper

HTML scraping library focused on easy to use.

In this library, matching patterns are described as HTML DOM trees.
You can write patterns intuitive and extract desired contents easily.

# Usage

Add this line to your `Cargo.toml`:

```toml
[dependencies]
easy-scraper = "0.1"
```

# Example

```rust
use easy_scraper::Pattern;

let pat = Pattern::new(r#"
<ul>
    <li>{{foo}}</li>
</ul>
"#).unwrap();

let ms = pat.matches(r#"
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
"#);

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

* Whole sub-tree patterns must be only one element of the parent node.

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
