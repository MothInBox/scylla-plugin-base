use extism_pdk::*;
use getrandom::Error;
use scraper::{Html, Selector};
use scylla_plugin_api::{ChapterOutput, ConfigField, PluginChapter, PluginSchema, ScrapeInput, ScrapeOutput};

// getrandom v0.3 wasm stub — required for scraper dependency
#[no_mangle]
pub unsafe extern "Rust" fn __getrandom_v03_custom(dest: *mut u8, len: usize) -> Result<(), Error> {
    let buf = std::slice::from_raw_parts_mut(dest, len);
    for b in buf {
        *b = 0;
    }
    Ok(())
}

// ── Entrypoints ───────────────────────────────────────────────────────────────

#[plugin_fn]
pub fn get_config_schema(Json(()): Json<()>) -> FnResult<Json<PluginSchema>> {
    Ok(Json(PluginSchema {
        fields: vec![
            ConfigField {
                key: "user_agent".into(),
                label: "User-Agent header".into(),
                field_type: "string".into(),
                default: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36".into(),
            },
        ],
        accepts_cookies: false,
    }))
}

#[plugin_fn]
pub fn scrape_book(Json(input): Json<ScrapeInput>) -> FnResult<Json<ScrapeOutput>> {
    let user_agent = config_value(&input.config, "user_agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36");
    let html = fetch(&input.url, &user_agent)?;
    let title = parse_title(&html).ok_or_else(|| extism_pdk::Error::msg("Failed to parse title from page"))?;
    let cover_url = parse_og_tag(&html, "og:image");
    let description = parse_ld_description(&html);
    let chapters = parse_chapters(&html);
    let total = parse_chapter_count(&html).max(chapters.len() as u32);

    Ok(Json(ScrapeOutput {
        title,
        url: input.url,
        cover_url,
        description,
        total_chapters: total,
        chapters,
    }))
}

#[plugin_fn]
pub fn scrape_chapter(Json(input): Json<ScrapeInput>) -> FnResult<Json<ChapterOutput>> {
    let user_agent = config_value(&input.config, "user_agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36");
    let html = fetch(&input.url, &user_agent)?;
    let title = parse_chapter_title(&html);
    let content = parse_chapter_content(&html);

    Ok(Json(ChapterOutput { title, content }))
}

// ── HTTP ─────────────────────────────────────────────────────────────────────

fn config_value(config: &Option<String>, key: &str, default: &str) -> String {
    config
        .as_deref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v| v.get(key).and_then(|v| v.as_str().map(String::from)))
        .unwrap_or_else(|| default.to_string())
}

fn fetch(url: &str, user_agent: &str) -> FnResult<String> {
    let req = HttpRequest {
        url: url.to_string(),
        headers: [
            (
                "User-Agent".to_string(),
                user_agent.to_string(),
            ),
            (
                "Accept".to_string(),
                "text/html,application/xhtml+xml".to_string(),
            ),
            (
                "Referer".to_string(),
                "https://www.royalroad.com/".to_string(),
            ),
        ]
        .into(),
        method: Some("GET".to_string()),
    };
    Ok(String::from_utf8(http::request::<()>(&req, None)?.body()).unwrap_or_default())
}

// ── Book parsers ──────────────────────────────────────────────────────────────

fn parse_title(html: &str) -> Option<String> {
    if let Some(t) = parse_og_tag(html, "og:title").filter(|s| !s.is_empty()) {
        return Some(decode_entities(&t));
    }
    let marker = "property=\"og:title\" content=\"";
    if let Some(start) = html.find(marker) {
        let start = start + marker.len();
        if let Some(end) = html[start..].find('"') {
            let t = html[start..start + end].trim().to_string();
            if !t.is_empty() {
                return Some(decode_entities(&t));
            }
        }
    }
    let start = html.find("<title>")? + "<title>".len();
    let end = html[start..].find("</title>")? + start;
    let full = html[start..end].trim();
    Some(decode_entities(full.split('|').next()?.trim()))
}

fn parse_og_tag(html: &str, property: &str) -> Option<String> {
    let dom = tl::parse(html, tl::ParserOptions::default()).ok()?;
    let parser = dom.parser();
    dom.query_selector(&format!("meta[property=\"{}\"]", property))?
        .filter_map(|h| h.get(parser)?.as_tag())
        .find_map(|tag| {
            tag.attributes()
                .get("content")??
                .try_as_utf8_str()
                .map(|s| s.trim().to_string())
        })
}

fn parse_ld_description(html: &str) -> Option<String> {
    let start = html.find("application/ld+json\">")? + "application/ld+json\">".len();
    let end = html[start..].find("</script>")? + start;
    let json: serde_json::Value = serde_json::from_str(html[start..end].trim()).ok()?;
    Some(clean_text(json.get("description")?.as_str()?))
}

fn parse_chapter_count(html: &str) -> u32 {
    (|| -> Option<u32> {
        let attr = "data-chapters=\"";
        let start = html.find(attr)? + attr.len();
        let end = html[start..].find('"')? + start;
        html[start..end].trim().replace(',', "").parse().ok()
    })()
    .unwrap_or(0)
}

fn parse_chapters(html: &str) -> Vec<PluginChapter> {
    let document = Html::parse_document(html);
    let Ok(row_sel) = Selector::parse("table#chapters tbody tr") else {
        return Vec::new();
    };
    let Ok(link_sel) = Selector::parse("a[href]") else {
        return Vec::new();
    };

    document
        .select(&row_sel)
        .enumerate()
        .filter_map(|(i, row)| {
            let link = row.select(&link_sel).next()?;
            let path = link.value().attr("href")?;
            if !path.contains("/chapter/") {
                return None;
            }
            let title = decode_entities(link.text().collect::<String>().trim());
            let url = if path.starts_with("http") {
                path.to_string()
            } else {
                format!("https://www.royalroad.com{}", path)
            };
            Some(PluginChapter {
                title,
                url,
                order: i as u32 + 1,
            })
        })
        .collect()
}

// ── Chapter parsers ───────────────────────────────────────────────────────────

fn parse_chapter_title(html: &str) -> String {
    let Ok(dom) = tl::parse(html, tl::ParserOptions::default()) else {
        return "Chapter".to_string();
    };
    let parser = dom.parser();
    dom.query_selector("h1.font-white")
        .and_then(|mut it| it.next())
        .and_then(|h| h.get(parser)?.as_tag())
        .map(|tag| decode_entities(tag.inner_text(parser).trim()))
        .unwrap_or_else(|| "Chapter".to_string())
}

fn parse_chapter_content(html: &str) -> String {
    let Ok(dom) = tl::parse(html, tl::ParserOptions::default()) else {
        return "Could not extract content.".to_string();
    };
    let parser = dom.parser();
    dom.query_selector("div.chapter-inner")
        .and_then(|mut it| it.next())
        .and_then(|h| h.get(parser)?.as_tag())
        .map(|tag| clean_text(&tag.inner_html(parser)))
        .unwrap_or_else(|| "Could not extract content.".to_string())
}

// ── Text utilities ────────────────────────────────────────────────────────────

fn clean_text(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push('\n');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    let out = decode_entities(&out);
    out.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn decode_entities(s: &str) -> String {
    let s = s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        .replace("&#x27;", "'")
        .replace("&#x2F;", "/")
        .replace("\u{00a0}", " ");

    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '&' {
            let mut entity = String::new();
            while let Some(&next) = chars.peek() {
                if next == ';' {
                    chars.next();
                    break;
                }
                entity.push(next);
                chars.next();
            }
            if entity.starts_with("#x") || entity.starts_with("#X") {
                if let Ok(code) = u32::from_str_radix(&entity[2..], 16) {
                    if let Some(c) = char::from_u32(code) {
                        result.push(c);
                        continue;
                    }
                }
            } else if entity.starts_with('#') {
                if let Ok(code) = entity[1..].parse::<u32>() {
                    if let Some(c) = char::from_u32(code) {
                        result.push(c);
                        continue;
                    }
                }
            }
            result.push('&');
            result.push_str(&entity);
            result.push(';');
        } else {
            result.push(ch);
        }
    }
    result
}
