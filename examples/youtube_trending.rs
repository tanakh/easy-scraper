fn main() {
    let pat = easy_scraper::Pattern::new(
        r##"
<li>
    <div class="yt-lockup-content">
        <h3 class="yt-lockup-title">
            <a href="{{url}}">{{title}}</a>
        </h3>
        <div class="yt-lockup-byline">
            <a href="{{channel-url}}">{{channel}}</a>
        </div>
        <div class="yt-lockup-meta">
            <ul class="yt-lockup-meta-info">
                <li>{{date}}</li>
                <li>{{view}}</li>
            </ul>
        </div>
    </div>
</li>
<li>
    最近急上昇
</li>
"##,
    )
    .unwrap();

    let doc = reqwest::blocking::get("https://www.youtube.com/feed/trending")
        .unwrap()
        .text()
        .unwrap();
    let ms = pat.matches(&doc);
    println!("{:#?}", ms);
    println!("{} results", ms.len());
}
