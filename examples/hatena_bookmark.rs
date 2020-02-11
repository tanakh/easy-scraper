use easy_scraper::Pattern;

fn main() {
    let pat = Pattern::new(
        r#"
<div class="entrylist-contents-main">
    <h3 class="entrylist-contents-title">
        <a href="{{url}}" title="{{title}}"></a>
    </h3>
    <span class="entrylist-contents-users">
        <a><span>{{users}}</span> users</a>
    </span>
    <div class="entrylist-contents-body">
        <a>
            <p>{{snippet}}</p>
        </a>
    </div>
    <div class="entrylist-contents-detail">
        <ul class="entrylist-contents-meta">
            <li class="entrylist-contents-category">
                <a>{{category}}</a>
            </li>
            <li class="entrylist-contents-date">{{date}}</li>
        </ul>
    </div>
</div>
"#,
    )
    .unwrap();

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/69.0.3497.100")
        .build()
        .unwrap();
    let doc = client
        .get("https://b.hatena.ne.jp/hotentry/it")
        .send()
        .unwrap()
        .text()
        .unwrap();

    let ms = pat.matches(&doc);
    println!("{:#?}", ms);
}
