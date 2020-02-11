use easy_scraper::Pattern;

fn main() {
    let pat = Pattern::new(
        r#"
<li class="topicsListItem">
    <a href="{{url}}">{{title}}</a>
</li>
"#,
    )
    .unwrap();
    let doc = reqwest::blocking::get("https://news.yahoo.co.jp/")
        .unwrap()
        .text()
        .unwrap();
    let ms = pat.matches(&doc);
    println!("{:#?}", ms);
}
