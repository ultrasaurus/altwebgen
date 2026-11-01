pub mod config;
pub mod devserve;
pub mod setup;

pub mod util;
pub mod web;
pub use web::words;

#[cfg(test)]
mod tests {

    use crate::{config::*, setup};
    use std::path::Path;
    // use test_log::test; // logging output for debugging tests

    #[test]
    // tests a sample app to verify that assets are copied into outdir when building
    fn test_sample_template() {
        let config = Config::new(
            "src/test/sample-template/.dist",
            "src/test/sample-template/source",
            "src/test/sample-template/template",
            "",
            Mode::Build,
            Transcript::Off,
        );

        let result = setup::init_and_build(&config);
        if let Err(e) = result {
            panic!("unexpected failure of setup::init_and_build: {:?}", e);
        }

        // test that static files were copied
        assert!(Path::new("src/test/sample-template/.dist/theme/theme.css").exists());
        assert!(
            Path::new("src/test/sample-template/.dist/theme/pixeltrue-web-design.png").exists()
        );
        assert!(
            Path::new("src/test/sample-template/.dist/theme/icons8-favicon-pulsar-color.ico")
                .exists()
        );
    }

    #[test]
    // builds samples/blog: pretty urls, drafts, prev/next within a category,
    // and archive pages listed by month
    fn test_sample_blog() {
        let config = Config::new(
            "samples/blog/.dist",
            "samples/blog/source",
            "samples/blog/template",
            "",
            Mode::Build,
            Transcript::Off,
        );
        // output from an earlier build whose source is gone
        let stale = Path::new("samples/blog/.dist/2009/04/deleted-post/index.html");
        std::fs::create_dir_all(stale.parent().unwrap()).unwrap();
        std::fs::write(stale, "old").unwrap();

        setup::init_and_build(&config).unwrap();
        let read = |p: &str| std::fs::read_to_string(format!("samples/blog/.dist/{p}")).unwrap();
        assert!(!Path::new("samples/blog/.dist/2009/04/deleted-post").exists());

        assert!(!Path::new("samples/blog/.dist/2009/05/unfinished").exists());
        assert!(!Path::new("samples/blog/.dist/2009/05/unfinished.html").exists());
        assert!(!Path::new("samples/blog/.dist/collection.yaml").exists());

        // general: hello-world -> later-that-day (same day, by time) -> spring,
        // skipping the code posts between
        let hello = read("2009/04/hello-world/index.html");
        assert!(hello.contains(r#"<a href="/2009/04/later-that-day/">Later that day &rarr;</a>"#));
        assert!(!hello.contains("&larr;"));
        // `comments` from the data file hello-world.comments.yaml, which
        // isn't published
        assert!(hello.contains(r#"<a href="https://example.com/first">First Reader</a>"#));
        assert!(hello.contains("<p>Looking forward to more.</p>"));
        assert!(!Path::new("samples/blog/.dist/2009/04/hello-world.comments.yaml").exists());
        assert!(!read("2009/04/later-that-day/index.html").contains("Comments"));
        let later = read("2009/04/later-that-day/index.html");
        assert!(later.contains(r#"<a href="/2009/04/hello-world/">&larr; Hello, world</a>"#));
        assert!(later.contains(r#"<a href="/2009/05/spring/">Spring &rarr;</a>"#));
        // code: first-code -> more-code
        let more_code = read("2009/04/more-code/index.html");
        assert!(more_code.contains(r#"<a href="/2009/04/first-code/">&larr; First code</a>"#));
        assert!(!more_code.contains("&rarr;"));
        // category from the collection defaults
        assert!(read("2009/05/spring/index.html").contains("May 2, 2009 &middot; general"));

        let archives = read("archives.html");
        let may = archives.find("<h2>May 2009</h2>").unwrap();
        let april = archives.find("<h2>April 2009</h2>").unwrap();
        assert!(may < april, "newest month first");
        let code = read("category-code.html");
        assert!(code.contains("/2009/04/more-code/"));
        assert!(!code.contains("/2009/04/hello-world/"));
        // home: two lists (`as`), the 3 newest with excerpts, then code posts
        let home = read("index.html");
        let recent = &home[home.find("Recent posts").unwrap()..home.find("From the code").unwrap()];
        assert_eq!(recent.matches("<article>").count(), 3, "limit: 3");
        assert!(recent.contains(r#"<a href="/2009/05/spring/">Spring</a>"#));
        assert!(recent.contains("<p>No category, so it is general by default.</p>"));
        assert!(!recent.contains("/2009/04/hello-world/"));
        assert!(home.contains(r#"<li><a href="/2009/04/first-code/">First code</a></li>"#));
        // the drafts page has `status: draft` too
        assert!(!Path::new("samples/blog/.dist/drafts.html").exists());
        assert!(!archives.contains("unfinished"));
        // sitemap from `site_url` in _site.yaml, which isn't published
        let sitemap = read("sitemap.xml");
        assert!(sitemap.contains(
            "<url><loc>https://blog.example.com/2009/04/hello-world/</loc><lastmod>2009-04-17</lastmod></url>"
        ));
        assert!(sitemap.contains("<loc>https://blog.example.com/archives.html</loc>"));
        assert!(!sitemap.contains("unfinished"));
        assert!(!Path::new("samples/blog/.dist/_site.yaml").exists());

        // with drafts (`dev --drafts`): built, labeled, listed, in prev/next;
        // the next plain build above removes them again
        let config = Config {
            drafts: true,
            ..config
        };
        setup::init_and_build(&config).unwrap();
        let unfinished = read("2009/05/unfinished/index.html");
        assert!(unfinished.contains(r#"<p class="draft">Draft</p>"#));
        assert!(unfinished.contains(r#"<a href="/2009/05/spring/">&larr; Spring</a>"#));
        assert!(!read("2009/05/spring/index.html").contains(r#"<p class="draft">"#));
        let drafts = read("drafts.html");
        assert!(drafts.contains("/2009/05/unfinished/"));
        assert!(!drafts.contains("/2009/05/spring/"));
        assert!(read("archives.html").contains(r#"<span class="draft">Draft</span>"#));
        assert!(!read("sitemap.xml").contains("unfinished"));
    }

    fn audio_sample_config(transcript_opt: Transcript) -> Config {
        Config::new(
            "src/test/sample-audio/.dist",
            "src/test/sample-audio/source",
            "src/test/sample-audio/template",
            "",
            Mode::Build,
            transcript_opt,
        )
    }

    #[test]
    fn test_sample_audio_no_transcript() {
        use crate::util::NodeRefExt;
        use kuchikiki::traits::TendrilSink;

        let config = audio_sample_config(Transcript::Off);

        let _ = setup::init_and_build(&config).unwrap();
        // test generated html
        let index_file_path = "src/test/sample-audio/.dist/index.html";
        assert!(Path::new(index_file_path).exists());
        let html = std::fs::read_to_string(index_file_path).unwrap();
        let document = kuchikiki::parse_html().one(html);
        // let stdout: std::io::Stdout = std::io::stdout();
        // let mut handle: std::io::StdoutLock<'_> = stdout.lock();
        // let _ = document.serialize(&mut handle);
        let audio_node = document.find_html_child_element("audio").unwrap();
        let maybe_paragraph = audio_node.as_node().next_sibling();
        assert!(maybe_paragraph.is_some());
        let paragraph = maybe_paragraph.unwrap();
        assert_eq!(paragraph.children().count(), 1);
        let text = paragraph.text_contents();
        assert_eq!(text.as_str(), "Let me introduce the word \"hypertext\" to mean a body of written or pictorial material interconnected in such a complex way that it could not conveniently be presented or represented on paper.");
    }
}
