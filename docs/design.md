# Syntax

Based on HTML DOM tree.

```html
<ul>
    <li>{{foo}}</li>
</ul>
```

## siblings

Siblings are match only consective elements:

```html
<ul>
    <li>{{foo}}</li>
    <li>{{bar}}</li>
</ul>
```

This pattern matches against

```html
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
```

this document, results are:

```
{ "foo" => 1, "bar" => 2 }
{ "foo" => 2, "bar" => 3 }
```

## non-consective siblings

`...` indicates allowing any node in sublings.

```
<ul>
    <li>{{foo}}</li>
    ...
    <li>{{bar}}</li>
</ul>
```

This matches

```
{ "foo" => 1, "bar" => 2 }
{ "foo" => 1, "bar" => 3 }
{ "foo" => 2, "bar" => 3 }
```

## attribute

Attributes matches superset of words

```html
<div class="foo">{{foo}}</div>
```

## variable

```html
<a href="{{url}}">hoge hoge</a>
```

## text node

```html
<a href="{{url}}">aaa {{foo}} bbb {{bar}}</a>
```

```html
<a href="{{url}}">{{whole_sub_tree:*}}</a>
```
