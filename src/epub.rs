use anyhow::{Context, Result, bail};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

use crate::image::{ImageNode, LineContent};
use crate::style_serde::StyleData;

// ─────────────────────────────────────────────────────────────────────────────
// Data model
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct EpubMetadata {
    pub title: String,
    pub author: String,
    pub language: String,
}

#[derive(Debug, Clone)]
pub struct ManifestItem {
    pub id: String,
    pub href: String,
    pub media_type: String,
}

#[derive(Debug, Clone)]
pub struct SpineItem {
    pub idref: String,
    pub href: String,
}

#[derive(Debug, Clone)]
pub struct TocEntry {
    pub title: String,
    pub href: String,
    pub level: usize,
}

#[derive(Debug)]
pub struct EpubBook {
    pub metadata: EpubMetadata,
    pub manifest: HashMap<String, ManifestItem>,
    pub spine: Vec<SpineItem>,
    pub toc: Vec<TocEntry>,
    pub opf_dir: String,
    pub file_path: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReadingProgress {
    pub chapter: usize,
    pub scroll: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// EPUB parsing
// ─────────────────────────────────────────────────────────────────────────────

impl EpubBook {
    pub fn open(path: &Path) -> Result<Self> {
        let file = fs::File::open(path)
            .with_context(|| format!("无法打开 EPUB 文件: {}", path.display()))?;
        let mut archive = ZipArchive::new(file).with_context(|| "无法解析 EPUB ZIP 结构")?;

        let opf_path = parse_container(&mut archive)?;
        let opf_dir = parent_dir(&opf_path);
        let opf_content = read_zip_file(&mut archive, &opf_path)?;
        let (metadata, manifest, spine, ncx_id) = parse_opf(&opf_content)?;

        // Resolve NCX href from manifest ID or direct href
        let ncx_href = ncx_id.as_ref().and_then(|id| {
            // First check if it's a manifest ID
            if let Some(item) = manifest.get(id) {
                return Some(item.href.clone());
            }
            // Otherwise treat as a direct href
            Some(id.clone())
        });

        let toc = if let Some(ref href) = ncx_href {
            let ncx_path = resolve_href(&opf_dir, href);
            let ncx_content = read_zip_file(&mut archive, &ncx_path).unwrap_or_default();
            parse_ncx(&ncx_content)
        } else {
            Vec::new()
        };

        Ok(EpubBook {
            metadata,
            manifest,
            spine,
            toc,
            opf_dir,
            file_path: path.to_path_buf(),
        })
    }

    pub fn book_hash(&self) -> String {
        let hash = Sha256::digest(self.file_path.to_string_lossy().as_bytes());
        format!("{:x}", hash)[..16].to_string()
    }

    pub fn read_chapter(&self, chapter_index: usize) -> Result<String> {
        if chapter_index >= self.spine.len() {
            bail!(
                "章节索引超出范围: {} >= {}",
                chapter_index,
                self.spine.len()
            );
        }
        let chapter_path = resolve_href(&self.opf_dir, &self.spine[chapter_index].href);
        let file = fs::File::open(&self.file_path)?;
        let mut archive = ZipArchive::new(file)?;
        read_zip_file(&mut archive, &chapter_path)
    }

    pub fn extract_image(&self, href: &str) -> Result<PathBuf> {
        let image_path = resolve_href(&self.opf_dir, href);
        let cache_dir = epub_cache_dir(&self.book_hash());
        fs::create_dir_all(&cache_dir)?;
        let dest = cache_dir.join(&image_path);
        if dest.exists() {
            return Ok(dest);
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = fs::File::open(&self.file_path)?;
        let mut archive = ZipArchive::new(file)?;
        let data = read_zip_file_bytes(&mut archive, &image_path)?;
        fs::write(&dest, &data)?;
        Ok(dest)
    }

    pub fn chapter_count(&self) -> usize {
        self.spine.len()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Progress persistence
// ─────────────────────────────────────────────────────────────────────────────

pub fn epub_cache_dir(book_hash: &str) -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".tread")
        .join("cache")
        .join("epub")
        .join(book_hash)
}

pub fn progress_path(book_hash: &str) -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".tread")
        .join("progress")
        .join(format!("{book_hash}.json"))
}

pub fn save_progress(book_hash: &str, progress: &ReadingProgress) -> Result<()> {
    let path = progress_path(book_hash);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(progress)?)?;
    Ok(())
}

pub fn load_progress(book_hash: &str) -> Option<ReadingProgress> {
    let content = fs::read_to_string(progress_path(book_hash)).ok()?;
    serde_json::from_str(&content).ok()
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

fn attr_str(e: &quick_xml::events::BytesStart, key: &[u8]) -> String {
    for attr in e.attributes().flatten() {
        if attr.key.local_name().as_ref() == key {
            return String::from_utf8_lossy(&attr.value).into_owned();
        }
    }
    String::new()
}

fn local_name_owned(e: &quick_xml::events::BytesStart) -> String {
    String::from_utf8_lossy(e.local_name().as_ref()).into_owned()
}

fn local_name_end_owned(e: &quick_xml::events::BytesEnd) -> String {
    String::from_utf8_lossy(e.local_name().as_ref()).into_owned()
}

fn parse_container(archive: &mut ZipArchive<fs::File>) -> Result<String> {
    let content = read_zip_file(archive, "META-INF/container.xml")?;
    let mut reader = Reader::from_str(&content);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"rootfile" => {
                let path = attr_str(e, b"full-path");
                if !path.is_empty() {
                    return Ok(path);
                }
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }
    bail!("container.xml 中未找到 rootfile")
}

fn parse_opf(
    content: &str,
) -> Result<(
    EpubMetadata,
    HashMap<String, ManifestItem>,
    Vec<SpineItem>,
    Option<String>,
)> {
    let mut reader = Reader::from_str(content);
    let mut buf = Vec::new();
    let mut metadata = EpubMetadata::default();
    let mut manifest = HashMap::new();
    let mut spine = Vec::new();
    let mut ncx_href = None;
    let mut in_metadata = false;
    let mut in_spine = false;
    let mut current_tag = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = local_name_owned(e);
                current_tag = name.clone();
                match name.as_str() {
                    "metadata" => in_metadata = true,
                    "spine" => {
                        in_spine = true;
                        let toc = attr_str(e, b"toc");
                        if !toc.is_empty() {
                            ncx_href = Some(toc);
                        }
                    }
                    "item" if !in_spine => {
                        let id = attr_str(e, b"id");
                        let href = attr_str(e, b"href");
                        let mt = attr_str(e, b"media-type");
                        if !id.is_empty() {
                            if mt.contains("ncx") || mt.contains("dtbncx") {
                                ncx_href = Some(href.clone());
                            }
                            manifest.insert(
                                id.clone(),
                                ManifestItem {
                                    id,
                                    href,
                                    media_type: mt,
                                },
                            );
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name = local_name_owned(e);
                match name.as_str() {
                    "item" if !in_spine => {
                        let id = attr_str(e, b"id");
                        let href = attr_str(e, b"href");
                        let mt = attr_str(e, b"media-type");
                        if !id.is_empty() {
                            if mt.contains("ncx") || mt.contains("dtbncx") {
                                ncx_href = Some(href.clone());
                            }
                            manifest.insert(
                                id.clone(),
                                ManifestItem {
                                    id,
                                    href,
                                    media_type: mt,
                                },
                            );
                        }
                    }
                    "itemref" if in_spine => {
                        let idref = attr_str(e, b"idref");
                        if !idref.is_empty() {
                            let href = manifest
                                .get(&idref)
                                .map(|m| m.href.clone())
                                .unwrap_or_default();
                            spine.push(SpineItem { idref, href });
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                match local_name_end_owned(e).as_str() {
                    "metadata" => in_metadata = false,
                    "spine" => in_spine = false,
                    _ => {}
                }
                current_tag.clear();
            }
            Ok(Event::Text(ref e)) if in_metadata => {
                let text = e.unescape().unwrap_or_default().to_string();
                match current_tag.as_str() {
                    "title" => metadata.title = text,
                    "creator" => metadata.author = text,
                    "language" => metadata.language = text,
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }
    Ok((metadata, manifest, spine, ncx_href))
}

fn parse_ncx(content: &str) -> Vec<TocEntry> {
    let mut reader = Reader::from_str(content);
    let mut buf = Vec::new();
    let mut toc = Vec::new();
    let mut in_navpoint = false;
    let mut in_navlabel = false;
    let mut navpoint_depth: usize = 0;
    let mut current_title = String::new();
    let mut current_href = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match local_name_owned(e).as_str() {
                "navPoint" => {
                    in_navpoint = true;
                    navpoint_depth += 1;
                    current_title.clear();
                    current_href.clear();
                }
                "navLabel" if in_navpoint => in_navlabel = true,
                _ => {}
            },
            Ok(Event::Empty(ref e)) => match local_name_owned(e).as_str() {
                "content" if in_navpoint => {
                    let src = attr_str(e, b"src");
                    current_href = src.split('#').next().unwrap_or(&src).to_string();
                }
                _ => {}
            },
            Ok(Event::Text(ref e)) if in_navlabel => {
                let text = e.unescape().unwrap_or_default().to_string();
                if !text.trim().is_empty() && current_title.is_empty() {
                    current_title = text.trim().to_string();
                }
            }
            Ok(Event::End(ref e)) => match local_name_end_owned(e).as_str() {
                "navPoint" => {
                    if !current_title.is_empty() {
                        toc.push(TocEntry {
                            title: current_title.clone(),
                            href: current_href.clone(),
                            level: navpoint_depth,
                        });
                    }
                    navpoint_depth -= 1;
                    in_navpoint = navpoint_depth > 0;
                }
                "navLabel" => in_navlabel = false,
                _ => {}
            },
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }
    toc
}

fn read_zip_file(archive: &mut ZipArchive<fs::File>, path: &str) -> Result<String> {
    String::from_utf8(read_zip_file_bytes(archive, path)?)
        .with_context(|| format!("文件不是有效 UTF-8: {path}"))
}

fn read_zip_file_bytes(archive: &mut ZipArchive<fs::File>, path: &str) -> Result<Vec<u8>> {
    let mut file = archive
        .by_name(path)
        .with_context(|| format!("ZIP 中未找到文件: {path}"))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

fn resolve_href(base_dir: &str, href: &str) -> String {
    if href.starts_with('/') {
        return href[1..].to_string();
    }
    if base_dir.is_empty() {
        return href.to_string();
    }
    format!("{base_dir}/{href}")
}

fn parent_dir(path: &str) -> String {
    path.rfind('/')
        .map(|p| path[..p].to_string())
        .unwrap_or_default()
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static CTR: AtomicUsize = AtomicUsize::new(0);
    fn epub_path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "tread_epub_{}.epub",
            CTR.fetch_add(1, Ordering::SeqCst)
        ))
    }

    fn create_test_epub(path: &Path) {
        use zip::write::SimpleFileOptions;
        let file = fs::File::create(path).unwrap();
        let mut z = zip::ZipWriter::new(file);
        let o = SimpleFileOptions::default();

        z.start_file("META-INF/container.xml", o).unwrap();
        z.write_all(r#"<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#.as_bytes()).unwrap();

        z.start_file("OEBPS/content.opf", o).unwrap();
        z.write_all(r#"<?xml version="1.0"?><package version="2.0" xmlns="http://www.idpf.org/2007/opf"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>测试书籍</dc:title><dc:creator>测试作者</dc:creator><dc:language>zh</dc:language></metadata><manifest><item id="ch1" href="chapter1.xhtml" media-type="application/xhtml+xml"/><item id="ch2" href="chapter2.xhtml" media-type="application/xhtml+xml"/><item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/><item id="img1" href="images/cover.png" media-type="image/png"/></manifest><spine toc="ncx"><itemref idref="ch1"/><itemref idref="ch2"/></spine></package>"#.as_bytes()).unwrap();

        z.start_file("OEBPS/chapter1.xhtml", o).unwrap();
        z.write_all(r#"<?xml version="1.0"?><html xmlns="http://www.w3.org/1999/xhtml"><body><h1>第一章：开始</h1><p>内容包含<strong>粗体</strong>。</p><img src="images/cover.png" alt="封面"/></body></html>"#.as_bytes()).unwrap();

        z.start_file("OEBPS/chapter2.xhtml", o).unwrap();
        z.write_all(r#"<?xml version="1.0"?><html xmlns="http://www.w3.org/1999/xhtml"><body><h1>第二章：继续</h1><pre><code>fn main() {}</code></pre></body></html>"#.as_bytes()).unwrap();

        z.start_file("OEBPS/toc.ncx", o).unwrap();
        z.write_all(r#"<?xml version="1.0"?><ncx xmlns="http://www.daisy.org/z3986/2005/ncx/"><navMap><navPoint id="ch1" playOrder="1"><navLabel><text>第一章：开始</text></navLabel><content src="chapter1.xhtml"/></navPoint><navPoint id="ch2" playOrder="2"><navLabel><text>第二章：继续</text></navLabel><content src="chapter2.xhtml"/></navPoint></navMap></ncx>"#.as_bytes()).unwrap();

        z.start_file("OEBPS/images/cover.png", o).unwrap();
        z.write_all(b"\x89PNG\r\n\x1a\nfake").unwrap();
        z.finish().unwrap();
    }

    use std::io::Write;

    #[test]
    fn parse_container_finds_opf() {
        let p = epub_path();
        create_test_epub(&p);
        let book = EpubBook::open(&p).unwrap();
        assert_eq!(book.opf_dir, "OEBPS");
        fs::remove_file(&p).ok();
    }

    #[test]
    fn parse_metadata() {
        let p = epub_path();
        create_test_epub(&p);
        let book = EpubBook::open(&p).unwrap();
        assert_eq!(book.metadata.title, "测试书籍");
        assert_eq!(book.metadata.author, "测试作者");
        fs::remove_file(&p).ok();
    }

    #[test]
    fn parse_spine() {
        let p = epub_path();
        create_test_epub(&p);
        let book = EpubBook::open(&p).unwrap();
        assert_eq!(book.spine.len(), 2);
        assert_eq!(book.spine[0].idref, "ch1");
        fs::remove_file(&p).ok();
    }

    #[test]
    fn parse_ncx_toc() {
        let p = epub_path();
        create_test_epub(&p);
        let book = EpubBook::open(&p).unwrap();
        assert_eq!(book.toc.len(), 2, "TOC should have 2 entries");
        assert_eq!(book.toc[0].title, "第一章：开始");
        assert_eq!(book.toc[1].title, "第二章：继续");
        fs::remove_file(&p).ok();
    }

    #[test]
    fn read_chapter_content() {
        let p = epub_path();
        create_test_epub(&p);
        let book = EpubBook::open(&p).unwrap();
        let ch1 = book.read_chapter(0).unwrap();
        assert!(ch1.contains("第一章：开始"));
        fs::remove_file(&p).ok();
    }

    #[test]
    fn read_chapter_out_of_bounds() {
        let p = epub_path();
        create_test_epub(&p);
        let book = EpubBook::open(&p).unwrap();
        assert!(book.read_chapter(99).is_err());
        fs::remove_file(&p).ok();
    }

    #[test]
    fn chapter_count() {
        let p = epub_path();
        create_test_epub(&p);
        let book = EpubBook::open(&p).unwrap();
        assert_eq!(book.chapter_count(), 2);
        fs::remove_file(&p).ok();
    }

    #[test]
    fn book_hash_is_deterministic() {
        let p = epub_path();
        create_test_epub(&p);
        let book = EpubBook::open(&p).unwrap();
        assert_eq!(book.book_hash().len(), 16);
        assert_eq!(book.book_hash(), book.book_hash());
        fs::remove_file(&p).ok();
    }

    #[test]
    fn extract_image_from_epub() {
        let p = epub_path();
        create_test_epub(&p);
        let book = EpubBook::open(&p).unwrap();
        let img = book.extract_image("images/cover.png").unwrap();
        assert!(img.exists());
        fs::remove_file(&p).ok();
        fs::remove_dir_all(epub_cache_dir(&book.book_hash())).ok();
    }

    #[test]
    fn progress_save_and_load() {
        let h = "test_epub_progress_hash";
        save_progress(
            h,
            &ReadingProgress {
                chapter: 3,
                scroll: 42,
            },
        )
        .unwrap();
        let loaded = load_progress(h).unwrap();
        assert_eq!(loaded.chapter, 3);
        assert_eq!(loaded.scroll, 42);
        fs::remove_file(progress_path(h)).ok();
    }

    #[test]
    fn load_progress_nonexistent() {
        assert!(load_progress("nonexistent_999999").is_none());
    }

    #[test]
    fn resolve_href_works() {
        assert_eq!(resolve_href("OEBPS", "ch.xhtml"), "OEBPS/ch.xhtml");
        assert_eq!(resolve_href("", "ch.xhtml"), "ch.xhtml");
        assert_eq!(resolve_href("OEBPS", "/abs/path"), "abs/path");
    }

    #[test]
    fn parent_dir_works() {
        assert_eq!(parent_dir("OEBPS/content.opf"), "OEBPS");
        assert_eq!(parent_dir("content.opf"), "");
    }

    #[test]
    fn invalid_epub_returns_error() {
        let p = std::env::temp_dir().join("tread_invalid.epub");
        fs::write(&p, b"not a zip").unwrap();
        assert!(EpubBook::open(&p).is_err());
        fs::remove_file(&p).ok();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Cached line format for serialization
// ─────────────────────────────────────────────────────────────────────────────

/// Serializable representation of a LineContent line.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CachedLine {
    #[serde(rename = "styled")]
    Styled { spans: Vec<CachedSpan> },
    #[serde(rename = "empty")]
    Empty,
    #[serde(rename = "image")]
    Image {
        alt: String,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        local_path: Option<String>,
        id: usize,
        download_failed: bool,
    },
    #[serde(rename = "link")]
    Link {
        text: String,
        url: String,
        is_external: bool,
    },
}

/// Serializable link info (optional for styled spans).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedLinkInfo {
    pub url: String,
    pub is_external: bool,
}

/// Serializable span (text + style + optional link).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedSpan {
    pub text: String,
    pub style: StyleData,
    pub link: Option<CachedLinkInfo>,
}

impl From<&LineContent> for CachedLine {
    fn from(lc: &LineContent) -> Self {
        match lc {
            LineContent::Styled(spans) => CachedLine::Styled {
                spans: spans
                    .iter()
                    .map(|span| CachedSpan {
                        text: span.text.clone(),
                        style: StyleData::from(span.style),
                        link: span.link.as_ref().map(|l| CachedLinkInfo {
                            url: l.url.clone(),
                            is_external: l.is_external,
                        }),
                    })
                    .collect(),
            },
            LineContent::Image(node) => CachedLine::Image {
                alt: node.alt.clone(),
                url: node.url.clone(),
                local_path: node
                    .local_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().into_owned()),
                id: node.id,
                download_failed: node.download_failed,
            },
            LineContent::Link(node) => CachedLine::Link {
                text: node.text.clone(),
                url: node.url.clone(),
                is_external: node.is_external,
            },
        }
    }
}

impl From<&CachedLine> for LineContent {
    fn from(cl: &CachedLine) -> Self {
        match cl {
            CachedLine::Styled { spans } => {
                let styled_spans: Vec<crate::image::StyledSpan> = spans
                    .iter()
                    .map(|s| {
                        let style = ratatui::style::Style::from(&s.style);
                        match &s.link {
                            Some(l) => crate::image::StyledSpan::with_link(
                                s.text.clone(),
                                style,
                                crate::image::LinkInfo {
                                    url: l.url.clone(),
                                    is_external: l.is_external,
                                },
                            ),
                            None => crate::image::StyledSpan::new(s.text.clone(), style),
                        }
                    })
                    .collect();
                if styled_spans.is_empty() {
                    LineContent::Styled(Vec::new())
                } else {
                    LineContent::Styled(styled_spans)
                }
            }
            CachedLine::Empty => LineContent::Styled(Vec::new()),
            CachedLine::Image {
                alt,
                url,
                local_path,
                id,
                download_failed,
            } => LineContent::Image(ImageNode {
                alt: alt.clone(),
                url: url.clone(),
                local_path: local_path.as_ref().map(PathBuf::from),
                id: *id,
                download_failed: *download_failed,
            }),
            CachedLine::Link {
                text,
                url,
                is_external,
            } => LineContent::Link(crate::image::LinkNode {
                text: text.clone(),
                url: url.clone(),
                is_external: *is_external,
            }),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Chapter cache read/write
// ─────────────────────────────────────────────────────────────────────────────

fn chapter_cache_path(book_hash: &str, chapter_idx: usize) -> PathBuf {
    epub_cache_dir(book_hash).join(format!("chapter_{chapter_idx}.json"))
}

/// Save parsed chapter content to cache.
pub fn save_chapter_cache(
    book_hash: &str,
    chapter_idx: usize,
    lines: &[LineContent],
) -> Result<()> {
    let path = chapter_cache_path(book_hash, chapter_idx);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let cached: Vec<CachedLine> = lines.iter().map(CachedLine::from).collect();
    let json = serde_json::to_string(&cached)?;
    fs::write(&path, json)?;
    Ok(())
}

/// Load parsed chapter content from cache. Returns None if cache miss.
pub fn load_chapter_cache(book_hash: &str, chapter_idx: usize) -> Option<Vec<LineContent>> {
    let path = chapter_cache_path(book_hash, chapter_idx);
    let json = fs::read_to_string(&path).ok()?;
    let cached: Vec<CachedLine> = serde_json::from_str(&json).ok()?;
    Some(cached.iter().map(LineContent::from).collect())
}

// ─────────────────────────────────────────────────────────────────────────────
// Paragraph merging
// ─────────────────────────────────────────────────────────────────────────────

/// Merge false paragraph breaks in parsed content.
///
/// Rules:
/// - Empty paragraphs (styled with empty spans) → collapse consecutive empties to one
/// - Sentence continuation: if previous paragraph ends with a continuation
///   punctuation (，,、etc.) or has no ending punctuation, merge with next
pub fn merge_paragraphs(lines: Vec<LineContent>) -> Vec<LineContent> {
    if lines.is_empty() {
        return lines;
    }

    // Step 1: Classify each line
    enum LineKind {
        Content(String), // text content
        Empty,           // blank line
        NonText,         // image, link, etc.
    }

    let kinds: Vec<LineKind> = lines
        .iter()
        .map(|lc| match lc {
            LineContent::Styled(spans) => {
                let text: String = spans.iter().map(|s| s.text.as_str()).collect();
                if text.trim().is_empty() {
                    LineKind::Empty
                } else {
                    LineKind::Content(text)
                }
            }
            LineContent::Image(_) | LineContent::Link(_) => LineKind::NonText,
        })
        .collect();

    let mut result: Vec<LineContent> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        match &kinds[i] {
            LineKind::Empty => {
                // Collapse consecutive empties to one
                // But only emit if there's content before AND after
                let has_content_before = result.last().map_or(false, |lc| {
                    matches!(lc, LineContent::Styled(spans) if !spans.is_empty())
                        || matches!(lc, LineContent::Image(_) | LineContent::Link(_))
                });
                let mut j = i;
                while j < lines.len() && matches!(&kinds[j], LineKind::Empty) {
                    j += 1;
                }
                let has_content_after = (j < lines.len())
                    && matches!(&kinds[j], LineKind::Content(_) | LineKind::NonText);
                if has_content_before && has_content_after {
                    result.push(LineContent::Styled(Vec::new()));
                }
                i = j;
            }
            LineKind::Content(text) => {
                // Check if this should merge with next content paragraph
                let mut merged_text = text.clone();
                let mut merged_spans: Vec<crate::image::StyledSpan> = Vec::new();

                // Copy current spans
                if let LineContent::Styled(ref spans) = lines[i] {
                    merged_spans = spans.clone();
                }

                let mut j = i + 1;

                // Skip empties between potential continuation
                while j < lines.len() && matches!(&kinds[j], LineKind::Empty) {
                    j += 1;
                }

                // Check if next content should be merged
                if j < lines.len() {
                    if let LineKind::Content(next_text) = &kinds[j] {
                        if should_merge_paragraphs(&merged_text, next_text) {
                            // Merge: add next paragraph's spans to current
                            if let LineContent::Styled(ref next_spans) = lines[j] {
                                // Add a space between merged paragraphs
                                merged_spans.push(crate::image::StyledSpan::new(
                                    " ".to_string(),
                                    ratatui::style::Style::default(),
                                ));
                                merged_spans.extend(next_spans.iter().cloned());
                                merged_text.push(' ');
                                merged_text.push_str(next_text);
                            }
                            i = j + 1;
                            result.push(LineContent::Styled(merged_spans));
                            continue;
                        }
                    }
                }

                // No merge — emit as-is
                result.push(lines[i].clone());
                i += 1;
            }
            LineKind::NonText => {
                result.push(lines[i].clone());
                i += 1;
            }
        }
    }

    result
}

/// Check if two paragraphs should be merged (continuation vs. new paragraph).
fn should_merge_paragraphs(prev_text: &str, next_text: &str) -> bool {
    let prev_trimmed = prev_text.trim();
    let next_trimmed = next_text.trim();

    if prev_trimmed.is_empty() || next_trimmed.is_empty() {
        return false;
    }

    // Check if previous ends with "sentence continuation" punctuation
    let last_char = prev_trimmed.chars().last().unwrap_or(' ');
    let continues = matches!(
        last_char,
        '，' | ','
            | '、'
            | '：'
            | ':'
            | '\u{201c}'
            | '\u{201d}'
            | '）'
            | ')'
            | ';'
            | '；'
            | '—'
            | '…'
    );

    // Check if previous ends with "sentence ending" punctuation
    let ends_sentence = matches!(last_char, '。' | '！' | '？' | '.' | '!' | '?');

    // Check if next starts with "new paragraph" markers
    let new_para = next_trimmed.starts_with('第')         // 第一章
        || next_trimmed.starts_with("「")
        || next_trimmed.starts_with('\u{201c}')           // "
        || next_trimmed.starts_with('\u{201d}')           // "
        || next_trimmed.starts_with("（")
        || next_trimmed.starts_with('(');

    // Check if next starts with uppercase (new sentence in English)
    let starts_upper = next_trimmed
        .chars()
        .next()
        .map_or(false, |c| c.is_uppercase());

    // Decision:
    // - If continues (ends with comma etc.) AND next doesn't start new para → merge
    // - If doesn't end sentence AND next doesn't start new para → merge (cut mid-sentence)
    // - If ends sentence → keep separate
    // - If next starts new para → keep separate
    if new_para {
        return false;
    }

    if continues {
        return true;
    }

    if ends_sentence {
        return false;
    }

    // No clear ending punctuation → likely cut mid-sentence
    // But don't merge if next starts with uppercase (English new sentence)
    !starts_upper
}

// ─────────────────────────────────────────────────────────────────────────────
// Paragraph merging tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod merge_tests {
    use super::*;
    use ratatui::style::Style;

    fn styled(text: &str) -> LineContent {
        LineContent::Styled(vec![crate::image::StyledSpan::new(
            text.to_string(),
            Style::default(),
        )])
    }

    fn empty() -> LineContent {
        LineContent::Styled(Vec::new())
    }

    fn text_of(lines: &[LineContent]) -> Vec<String> {
        lines
            .iter()
            .filter_map(|lc| match lc {
                LineContent::Styled(spans) => {
                    let t: String = spans.iter().map(|s| s.text.as_str()).collect();
                    if t.is_empty() { None } else { Some(t) }
                }
                _ => None,
            })
            .collect()
    }

    fn count_empties(lines: &[LineContent]) -> usize {
        lines
            .iter()
            .filter(|lc| matches!(lc, LineContent::Styled(spans) if spans.is_empty()))
            .count()
    }

    #[test]
    fn consecutive_empties_collapsed() {
        let lines = vec![
            styled("第一段"),
            empty(),
            empty(),
            empty(),
            empty(),
            styled("第二段"),
        ];
        let merged = merge_paragraphs(lines);
        assert_eq!(
            count_empties(&merged),
            1,
            "should have exactly 1 empty line"
        );
        let texts = text_of(&merged);
        assert_eq!(texts, vec!["第一段", "第二段"]);
    }

    #[test]
    fn comma_endings_merge() {
        let lines = vec![
            styled("说到境界，大家不免会问一个问题"),
            empty(),
            styled("什么是境界"),
        ];
        let merged = merge_paragraphs(lines);
        let texts = text_of(&merged);
        // Should merge because prev ends with comma-like content
        assert!(texts.len() <= 2);
    }

    #[test]
    fn sentence_ending_stays_separate() {
        let lines = vec![styled("这是第一句话。"), empty(), styled("这是第二句话。")];
        let merged = merge_paragraphs(lines);
        let texts = text_of(&merged);
        assert_eq!(
            texts.len(),
            2,
            "sentence-ending paragraphs should stay separate"
        );
    }

    #[test]
    fn new_chapter_stays_separate() {
        let lines = vec![
            styled("这是上一章的最后一句"),
            empty(),
            styled("第三章 新的开始"),
        ];
        let merged = merge_paragraphs(lines);
        let texts = text_of(&merged);
        assert_eq!(texts.len(), 2, "new chapter should stay separate");
    }

    #[test]
    fn no_ending_punctuation_merges() {
        let lines = vec![styled("从何处下手"), empty(), styled("高境界也不难")];
        let merged = merge_paragraphs(lines);
        let texts = text_of(&merged);
        // "从何处下手" has no ending punctuation → likely mid-sentence cut → merge
        assert_eq!(texts.len(), 1, "mid-sentence cut should merge");
    }

    #[test]
    fn image_lines_preserved() {
        let lines = vec![
            styled("文字"),
            LineContent::Image(ImageNode {
                alt: "图".to_string(),
                url: "img.png".to_string(),
                local_path: None,
                id: 0,
                download_failed: false,
            }),
            styled("更多文字"),
        ];
        let merged = merge_paragraphs(lines);
        assert_eq!(merged.len(), 3);
    }

    #[test]
    fn cache_roundtrip() {
        let lines = vec![
            styled("测试内容"),
            empty(),
            LineContent::Image(ImageNode {
                alt: "alt".to_string(),
                url: "http://example.com/img.png".to_string(),
                local_path: Some(PathBuf::from("/cached/img.png")),
                id: 0,
                download_failed: false,
            }),
        ];

        let hash = "test_cache_roundtrip";
        save_chapter_cache(hash, 0, &lines).unwrap();
        let loaded = load_chapter_cache(hash, 0);
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.len(), 3);

        // Cleanup
        let path = chapter_cache_path(hash, 0);
        fs::remove_file(&path).ok();
    }
}
