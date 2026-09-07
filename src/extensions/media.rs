//! Media file auto-detection and HTML generation
//!
//! This module provides functionality to detect media files by extension
//! and generate appropriate HTML5 media tags (video, audio, picture).

use std::path::Path;

/// Media type detected from file extension
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaType {
    Video,
    Audio,
    Image,
    Downloadable,
}

fn media_type_from_extension(ext: &str) -> Option<MediaType> {
    match ext {
        // Video extensions
        "mp4" | "webm" | "ogv" | "mov" | "avi" | "mkv" | "m4v" => Some(MediaType::Video),
        // Audio extensions
        "mp3" | "wav" | "ogg" | "oga" | "m4a" | "aac" | "flac" | "opus" | "weba" => {
            Some(MediaType::Audio)
        }
        // Image extensions
        "jpg" | "jpeg" | "png" | "gif" | "svg" | "webp" | "avif" | "bmp" | "ico" | "jxl"
        | "tif" | "tiff" => Some(MediaType::Image),
        // Downloadable file extensions
        // Archive formats
        "zip" | "tar" | "gz" | "7z" | "rar" | "bz2" | "xz" => Some(MediaType::Downloadable),
        // Document formats
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "odp" => {
            Some(MediaType::Downloadable)
        }
        // Text formats
        "txt" | "md" | "csv" | "json" | "xml" | "yaml" | "yml" | "toml" => {
            Some(MediaType::Downloadable)
        }
        // Executable formats
        "exe" | "dmg" | "deb" | "rpm" | "app" | "apk" | "msi" => Some(MediaType::Downloadable),
        _ => None,
    }
}

fn resolve_extension(url: &str, allow_fragment_extension_hint: bool) -> Option<String> {
    // 1) Prefer path extension from the actual URL path.
    let path = url.split('?').next()?.split('#').next()?;
    if let Some(ext) = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
    {
        return Some(ext);
    }

    // 2) Optionally allow fragment hint extension (e.g. `#.png`) for extension-less URLs.
    if !allow_fragment_extension_hint {
        return None;
    }

    let fragment = url.split('#').nth(1)?;
    let hinted = fragment.strip_prefix('.')?;
    if hinted.is_empty() || !hinted.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }

    Some(hinted.to_lowercase())
}

/// Detect media type from URL
///
/// # Arguments
///
/// * `url` - The URL to analyze (may include query parameters)
///
/// # Returns
///
/// `Some(MediaType)` if a known extension is found, `None` otherwise
///
/// # Examples
///
/// ```
/// use umd::extensions::media::detect_media_type;
/// use umd::extensions::media::MediaType;
///
/// assert_eq!(detect_media_type("video.mp4"), Some(MediaType::Video));
/// assert_eq!(detect_media_type("audio.mp3"), Some(MediaType::Audio));
/// assert_eq!(detect_media_type("image.png"), Some(MediaType::Image));
/// assert_eq!(detect_media_type("file.unknown"), None);
/// ```
pub fn detect_media_type(url: &str) -> Option<MediaType> {
    detect_media_type_with_hint(url, false)
}

/// Detect media type from URL with optional fragment extension hint support.
pub fn detect_media_type_with_hint(
    url: &str,
    allow_fragment_extension_hint: bool,
) -> Option<MediaType> {
    let ext = resolve_extension(url, allow_fragment_extension_hint)?;
    media_type_from_extension(ext.as_str())
}

/// Get MIME type for a file extension
///
/// # Arguments
///
/// * `url` - The URL to analyze
/// * `media_type` - The detected media type
///
/// # Returns
///
/// MIME type string
fn get_mime_type_with_hint(
    url: &str,
    media_type: &MediaType,
    allow_fragment_extension_hint: bool,
) -> String {
    let ext = resolve_extension(url, allow_fragment_extension_hint).unwrap_or_default();

    match media_type {
        MediaType::Video => match ext.as_str() {
            "mp4" => "video/mp4",
            "webm" => "video/webm",
            "ogv" | "ogg" => "video/ogg",
            "mov" => "video/quicktime",
            "avi" => "video/x-msvideo",
            "mkv" => "video/x-matroska",
            "m4v" => "video/x-m4v",
            _ => "video/mp4",
        },
        MediaType::Audio => match ext.as_str() {
            "mp3" => "audio/mpeg",
            "wav" => "audio/wav",
            "ogg" | "oga" => "audio/ogg",
            "m4a" => "audio/mp4",
            "aac" => "audio/aac",
            "flac" => "audio/flac",
            "opus" => "audio/opus",
            "weba" => "audio/webm",
            _ => "audio/mpeg",
        },
        MediaType::Image => match ext.as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "webp" => "image/webp",
            "avif" => "image/avif",
            "bmp" => "image/bmp",
            "ico" => "image/x-icon",
            "jxl" => "image/jxl",
            _ => "image/png",
        },
        MediaType::Downloadable => match ext.as_str() {
            "pdf" => "application/pdf",
            "zip" => "application/zip",
            "tar" => "application/x-tar",
            "gz" => "application/gzip",
            "json" => "application/json",
            "xml" => "application/xml",
            _ => "application/octet-stream",
        },
    }
    .to_string()
}

#[derive(Debug, Default)]
struct MediaAttributes {
    size: Option<MediaSize>,
    integrity: Vec<String>,
}

#[derive(Debug)]
enum MediaSize {
    Width(String),
    Dimensions(String, String),
}

fn parse_media_attributes(parameters: Option<&str>) -> MediaAttributes {
    let mut attributes = MediaAttributes::default();

    for token in parameters.unwrap_or_default().split(',').map(str::trim) {
        if token.is_empty() {
            continue;
        }

        if token
            .strip_prefix("sha256-")
            .or_else(|| token.strip_prefix("sha384-"))
            .or_else(|| token.strip_prefix("sha512-"))
            .is_some_and(is_base64)
        {
            attributes.integrity.push(token.to_string());
            continue;
        }

        let size = if let Some(width) = token.strip_suffix('%') {
            width
                .chars()
                .all(|c| c.is_ascii_digit())
                .then(|| MediaSize::Width(format!("{}%", width)))
        } else if let Some((width, height)) = token.split_once('x') {
            (width.chars().all(|c| c.is_ascii_digit())
                && height.chars().all(|c| c.is_ascii_digit()))
            .then(|| MediaSize::Dimensions(width.to_string(), height.to_string()))
        } else {
            token
                .chars()
                .all(|c| c.is_ascii_digit())
                .then(|| MediaSize::Width(token.to_string()))
        };

        if attributes.size.is_none() {
            attributes.size = size;
        }
    }

    attributes
}

fn is_base64(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '='))
}

impl MediaAttributes {
    fn size_attributes(&self) -> String {
        match &self.size {
            Some(MediaSize::Width(width)) if width.ends_with('%') => {
                format!(" style=\"width: {}\"", escape_html(width))
            }
            Some(MediaSize::Width(width)) => format!(" width=\"{}\"", escape_html(width)),
            Some(MediaSize::Dimensions(width, height)) => format!(
                " width=\"{}\" height=\"{}\"",
                escape_html(width),
                escape_html(height)
            ),
            None => String::new(),
        }
    }

    fn integrity_attribute(&self) -> String {
        if self.integrity.is_empty() {
            String::new()
        } else {
            format!(" integrity=\"{}\"", escape_html(&self.integrity.join(" ")))
        }
    }
}

/// Generate HTML for media element
///
/// # Arguments
///
/// * `url` - The media URL
/// * `alt` - Alt text (used for track label in video, ignored in audio)
/// * `title` - Optional title attribute
/// * `media_type` - The type of media
///
/// # Returns
///
/// HTML string for the media element
///
/// # Examples
///
/// ```
/// use umd::extensions::media::{generate_media_html, MediaType};
/// use umd::parser::Icons;
///
/// let html = generate_media_html("video.mp4", "Demo", Some("Product demo"), &MediaType::Video, &Icons::default());
/// assert!(html.contains("<video"));
/// assert!(html.contains("controls"));
/// ```
pub fn generate_media_html(
    url: &str,
    alt: &str,
    title: Option<&str>,
    media_type: &MediaType,
    icons: &crate::parser::Icons,
) -> String {
    generate_media_html_with_hint(url, alt, title, media_type, icons, false, None)
}

fn generate_media_html_with_hint(
    url: &str,
    alt: &str,
    title: Option<&str>,
    media_type: &MediaType,
    icons: &crate::parser::Icons,
    allow_fragment_extension_hint: bool,
    parameters: Option<&str>,
) -> String {
    let mime_type = get_mime_type_with_hint(url, media_type, allow_fragment_extension_hint);
    let media_attributes = parse_media_attributes(parameters);
    let title_attr = title
        .map(|t| format!(" title=\"{}\"", escape_html(t)))
        .unwrap_or_default();

    match media_type {
        MediaType::Video => {
            let track_label = escape_html(alt);
            let display_text = if alt.is_empty() { url } else { alt };
            format!(
                "<video controls{}>\n  <source src=\"{}\" type=\"{}\"{} />\n  <track kind=\"captions\" label=\"{}\" />\n  <a href=\"{}\" download class=\"download-link video-fallback\"{}>{} {}</a>\n</video>",
                format!("{}{}", title_attr, media_attributes.size_attributes()),
                escape_html(url),
                mime_type,
                media_attributes.integrity_attribute(),
                track_label,
                escape_html(url),
                media_attributes.integrity_attribute(),
                icons.video,
                escape_html(display_text)
            )
        }
        MediaType::Audio => {
            let display_text = if alt.is_empty() { url } else { alt };
            format!(
                "<audio controls{}>\n  <source src=\"{}\" type=\"{}\"{} />\n  <a href=\"{}\" download class=\"download-link audio-fallback\"{}>{} {}</a>\n</audio>",
                format!("{}{}", title_attr, media_attributes.size_attributes()),
                escape_html(url),
                mime_type,
                media_attributes.integrity_attribute(),
                escape_html(url),
                media_attributes.integrity_attribute(),
                icons.audio,
                escape_html(display_text)
            )
        }
        MediaType::Image => {
            let img_title = title
                .map(|t| format!(" title=\"{}\"", escape_html(t)))
                .unwrap_or_default();
            format!(
                "<picture{}>\n  <source srcset=\"{}\" type=\"{}\"{} />\n  <img src=\"{}\" alt=\"{}\" loading=\"lazy\" class=\"img-fluid\"{} />\n</picture>",
                format!("{}{}", title_attr, media_attributes.size_attributes()),
                escape_html(url),
                mime_type,
                media_attributes.integrity_attribute(),
                escape_html(url),
                escape_html(alt),
                img_title
            )
        }
        MediaType::Downloadable => {
            let display_text = if alt.is_empty() { url } else { alt };
            format!(
                "<a href=\"{}\" download class=\"download-link\"{}{}{}>\n  {} {}\n</a>",
                escape_html(url),
                title_attr,
                media_attributes.size_attributes(),
                media_attributes.integrity_attribute(),
                icons.download,
                escape_html(display_text)
            )
        }
    }
}

/// Escape HTML special characters
fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Transform image tags to media tags based on file extension
///
/// This function processes HTML and converts `<img>` tags to appropriate
/// media tags (`<video>`, `<audio>`, or `<picture>`) based on the file extension.
///
/// # Arguments
///
/// * `html` - The HTML string to transform
///
/// # Returns
///
/// Transformed HTML with media tags
///
/// # Examples
///
/// ```
/// use umd::extensions::media::transform_images_to_media;
/// use umd::parser::Icons;
///
/// let html = r#"<img src="video.mp4" alt="Demo" />"#;
/// let result = transform_images_to_media(html, &Icons::default(), false);
/// assert!(result.contains("<video"));
/// ```
pub fn transform_images_to_media(
    html: &str,
    icons: &crate::parser::Icons,
    allow_fragment_extension_hint: bool,
) -> String {
    use regex::Regex;

    // Pattern to match <img> tags with src and alt attributes
    let img_re = Regex::new(
        r#"<img\s+src="([^"]+)"(?:\s+alt="([^"]*)")?(?:\s+title="([^"]*)")?\s*/>(?:\{([^{}\n]*)\})?"#,
    )
    .unwrap();

    let transformed = img_re
        .replace_all(html, |caps: &regex::Captures| {
            let url = caps.get(1).map_or("", |m| m.as_str());
            let alt = caps.get(2).map_or("", |m| m.as_str());
            let title = caps.get(3).map(|m| m.as_str());

            // Detect media type and generate appropriate HTML
            if let Some(media_type) =
                detect_media_type_with_hint(url, allow_fragment_extension_hint)
            {
                generate_media_html_with_hint(
                    url,
                    alt,
                    title,
                    &media_type,
                    icons,
                    allow_fragment_extension_hint,
                    caps.get(4).map(|m| m.as_str()),
                )
            } else {
                // Not a recognized media file, wrap in <picture> tag anyway
                let media_attributes = parse_media_attributes(caps.get(4).map(|m| m.as_str()));
                let title_attr = title
                    .map(|t| format!(" title=\"{}\"", escape_html(t)))
                    .unwrap_or_default();
                let img_title = title
                    .map(|t| format!(" title=\"{}\"", escape_html(t)))
                    .unwrap_or_default();
                format!(
                    "<picture{}>\n  <img src=\"{}\" alt=\"{}\" loading=\"lazy\" class=\"img-fluid\"{} />\n</picture>",
                    format!("{}{}", title_attr, media_attributes.size_attributes()),
                    escape_html(url),
                    escape_html(alt),
                    img_title
                )
            }
        })
        .to_string();

    // Block media: if a paragraph consists only of a media element,
    // treat it as block-level output and wrap with <figure>.
    // Inline media inside text remains unchanged.
    let media_only_paragraph = Regex::new(
        r#"(?s)<p>\s*(<picture[\s\S]*?</picture>|<video[\s\S]*?</video>|<audio[\s\S]*?</audio>|<a href="[^"]+" download class="download-link[^"]*"[^>]*>[\s\S]*?</a>)\s*</p>"#,
    )
    .unwrap();

    media_only_paragraph
        .replace_all(&transformed, |caps: &regex::Captures| {
            format!("<figure class=\"w-100\">\n{}\n</figure>", &caps[1])
        })
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_video() {
        assert_eq!(detect_media_type("video.mp4"), Some(MediaType::Video));
        assert_eq!(detect_media_type("video.webm"), Some(MediaType::Video));
        assert_eq!(detect_media_type("video.ogv"), Some(MediaType::Video));
        assert_eq!(detect_media_type("video.mov"), Some(MediaType::Video));
        assert_eq!(detect_media_type("VIDEO.MP4"), Some(MediaType::Video)); // Case insensitive
    }

    #[test]
    fn test_detect_audio() {
        assert_eq!(detect_media_type("audio.mp3"), Some(MediaType::Audio));
        assert_eq!(detect_media_type("audio.wav"), Some(MediaType::Audio));
        assert_eq!(detect_media_type("audio.ogg"), Some(MediaType::Audio));
        assert_eq!(detect_media_type("AUDIO.MP3"), Some(MediaType::Audio)); // Case insensitive
    }

    #[test]
    fn test_detect_image() {
        assert_eq!(detect_media_type("image.png"), Some(MediaType::Image));
        assert_eq!(detect_media_type("image.jpg"), Some(MediaType::Image));
        assert_eq!(detect_media_type("image.jpeg"), Some(MediaType::Image));
        assert_eq!(detect_media_type("image.gif"), Some(MediaType::Image));
        assert_eq!(detect_media_type("image.webp"), Some(MediaType::Image));
        assert_eq!(detect_media_type("image.avif"), Some(MediaType::Image));
        assert_eq!(detect_media_type("image.jxl"), Some(MediaType::Image)); // JPEG XL
    }

    #[test]
    fn test_detect_with_query_params() {
        assert_eq!(detect_media_type("video.mp4?v=123"), Some(MediaType::Video));
        assert_eq!(
            detect_media_type("image.png?size=large#anchor"),
            Some(MediaType::Image)
        );
    }

    #[test]
    fn test_detect_fragment_extension_hint_opt_in() {
        assert_eq!(
            detect_media_type_with_hint("/assets/image#.png", false),
            None
        );
        assert_eq!(
            detect_media_type_with_hint("/assets/image#.png", true),
            Some(MediaType::Image)
        );
    }

    #[test]
    fn test_detect_fragment_extension_hint_reject_non_extension() {
        assert_eq!(
            detect_media_type_with_hint("/assets/media#t=10", true),
            None
        );
        assert_eq!(
            detect_media_type_with_hint("/assets/image#section", true),
            None
        );
    }

    #[test]
    fn test_detect_fragment_extension_hint_tiff() {
        assert_eq!(
            detect_media_type_with_hint("/assets/photo#.tiff", true),
            Some(MediaType::Image)
        );
        assert_eq!(
            detect_media_type_with_hint("/assets/photo#.tif", true),
            Some(MediaType::Image)
        );
    }

    #[test]
    fn test_detect_downloadable_archives() {
        assert_eq!(detect_media_type("file.zip"), Some(MediaType::Downloadable));
        assert_eq!(detect_media_type("file.tar"), Some(MediaType::Downloadable));
        assert_eq!(detect_media_type("file.gz"), Some(MediaType::Downloadable));
        assert_eq!(detect_media_type("file.7z"), Some(MediaType::Downloadable));
        assert_eq!(detect_media_type("file.rar"), Some(MediaType::Downloadable));
    }

    #[test]
    fn test_detect_downloadable_documents() {
        assert_eq!(
            detect_media_type("document.pdf"),
            Some(MediaType::Downloadable)
        );
        assert_eq!(
            detect_media_type("document.doc"),
            Some(MediaType::Downloadable)
        );
        assert_eq!(
            detect_media_type("document.docx"),
            Some(MediaType::Downloadable)
        );
        assert_eq!(
            detect_media_type("spreadsheet.xls"),
            Some(MediaType::Downloadable)
        );
        assert_eq!(
            detect_media_type("spreadsheet.xlsx"),
            Some(MediaType::Downloadable)
        );
        assert_eq!(
            detect_media_type("presentation.ppt"),
            Some(MediaType::Downloadable)
        );
        assert_eq!(
            detect_media_type("presentation.pptx"),
            Some(MediaType::Downloadable)
        );
    }

    #[test]
    fn test_detect_downloadable_text() {
        assert_eq!(detect_media_type("file.txt"), Some(MediaType::Downloadable));
        assert_eq!(detect_media_type("data.csv"), Some(MediaType::Downloadable));
        assert_eq!(
            detect_media_type("config.json"),
            Some(MediaType::Downloadable)
        );
        assert_eq!(detect_media_type("data.xml"), Some(MediaType::Downloadable));
        assert_eq!(
            detect_media_type("config.yaml"),
            Some(MediaType::Downloadable)
        );
    }

    #[test]
    fn test_detect_downloadable_executables() {
        assert_eq!(
            detect_media_type("installer.exe"),
            Some(MediaType::Downloadable)
        );
        assert_eq!(
            detect_media_type("installer.dmg"),
            Some(MediaType::Downloadable)
        );
        assert_eq!(
            detect_media_type("package.deb"),
            Some(MediaType::Downloadable)
        );
        assert_eq!(
            detect_media_type("package.rpm"),
            Some(MediaType::Downloadable)
        );
    }

    #[test]
    fn test_detect_unknown() {
        assert_eq!(detect_media_type("file.unknown"), None);
        assert_eq!(detect_media_type("noextension"), None);
    }

    #[test]
    fn test_generate_video_html() {
        let html = generate_media_html(
            "video.mp4",
            "Demo video",
            Some("Product demo"),
            &MediaType::Video,
            &crate::parser::Icons::default(),
        );
        assert!(html.contains("<video controls"));
        assert!(html.contains("title=\"Product demo\""));
        assert!(html.contains("src=\"video.mp4\""));
        assert!(html.contains("type=\"video/mp4\""));
        assert!(html.contains("<track kind=\"captions\" label=\"Demo video\""));
        // Check for download fallback
        assert!(
            html.contains("<a href=\"video.mp4\" download class=\"download-link video-fallback\">")
        );
        assert!(html.contains(
            r#"<iconify-icon icon="ic:baseline-videocam" aria-hidden="true"></iconify-icon> Demo video"#
        ));
    }

    #[test]
    fn test_generate_audio_html() {
        let html = generate_media_html(
            "audio.mp3",
            "Background music",
            Some("Theme song"),
            &MediaType::Audio,
            &crate::parser::Icons::default(),
        );
        assert!(html.contains("<audio controls"));
        assert!(html.contains("title=\"Theme song\""));
        assert!(html.contains("src=\"audio.mp3\""));
        assert!(html.contains("type=\"audio/mpeg\""));
        // Check for download fallback
        assert!(
            html.contains("<a href=\"audio.mp3\" download class=\"download-link audio-fallback\">")
        );
        assert!(html.contains(
            r#"<iconify-icon icon="ic:baseline-music-note" aria-hidden="true"></iconify-icon> Background music"#
        ));
    }

    #[test]
    fn test_generate_image_html() {
        let html = generate_media_html(
            "image.png",
            "Logo",
            Some("Company logo"),
            &MediaType::Image,
            &crate::parser::Icons::default(),
        );
        assert!(html.contains("<picture"));
        assert!(html.contains("title=\"Company logo\""));
        assert!(html.contains("srcset=\"image.png\""));
        assert!(html.contains("type=\"image/png\""));
        assert!(html.contains("alt=\"Logo\""));
        assert!(html.contains("loading=\"lazy\""));
    }

    #[test]
    fn test_generate_without_title() {
        let html = generate_media_html(
            "video.mp4",
            "Video",
            None,
            &MediaType::Video,
            &crate::parser::Icons::default(),
        );
        assert!(!html.contains("title="));
        assert!(html.contains("<video controls>"));
    }

    #[test]
    fn test_html_escape() {
        let html = generate_media_html(
            "video.mp4?foo=bar&baz=qux",
            "Test <script>",
            Some("Title with \"quotes\""),
            &MediaType::Video,
            &crate::parser::Icons::default(),
        );
        assert!(html.contains("&amp;"));
        assert!(html.contains("&lt;"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn test_generate_downloadable_html() {
        let html = generate_media_html(
            "document.pdf",
            "Research Report",
            Some("Annual Research"),
            &MediaType::Downloadable,
            &crate::parser::Icons::default(),
        );
        assert!(html.contains("<a href=\"document.pdf\" download class=\"download-link\""));
        assert!(html.contains("title=\"Annual Research\""));
        assert!(html.contains(
            r#"<iconify-icon icon="ic:baseline-file-download" aria-hidden="true"></iconify-icon> Research Report"#
        ));
    }

    #[test]
    fn test_downloadable_empty_alt() {
        let html = generate_media_html(
            "archive.zip",
            "",
            None,
            &MediaType::Downloadable,
            &crate::parser::Icons::default(),
        );
        assert!(html.contains("<a href=\"archive.zip\" download"));
        assert!(html.contains(
            r#"<iconify-icon icon="ic:baseline-file-download" aria-hidden="true"></iconify-icon> archive.zip"#
        )); // URL as fallback
    }

    #[test]
    fn test_video_empty_alt_fallback() {
        let html = generate_media_html(
            "video.mp4",
            "",
            None,
            &MediaType::Video,
            &crate::parser::Icons::default(),
        );
        assert!(html.contains(
            r#"<iconify-icon icon="ic:baseline-videocam" aria-hidden="true"></iconify-icon> video.mp4"#
        )); // URL as fallback in download link
    }

    #[test]
    fn test_audio_empty_alt_fallback() {
        let html = generate_media_html(
            "audio.mp3",
            "",
            None,
            &MediaType::Audio,
            &crate::parser::Icons::default(),
        );
        assert!(html.contains(
            r#"<iconify-icon icon="ic:baseline-music-note" aria-hidden="true"></iconify-icon> audio.mp3"#
        )); // URL as fallback in download link
    }

    #[test]
    fn test_downloadable_with_query_params() {
        let html = generate_media_html(
            "document.pdf?version=2",
            "User Guide",
            None,
            &MediaType::Downloadable,
            &crate::parser::Icons::default(),
        );
        assert!(html.contains("href=\"document.pdf?version=2\""));
        assert!(html.contains(
            r#"<iconify-icon icon="ic:baseline-file-download" aria-hidden="true"></iconify-icon> User Guide"#
        ));
    }

    #[test]
    fn test_custom_icons() {
        let icons = crate::parser::Icons {
            video: r#"<i class="bi bi-camera-video"></i>"#.to_string(),
            audio: r#"<i class="bi bi-music-note"></i>"#.to_string(),
            download: r#"<i class="bi bi-file-earmark-arrow-down"></i>"#.to_string(),
            ..crate::parser::Icons::default()
        };
        let html = generate_media_html("video.mp4", "Demo", None, &MediaType::Video, &icons);
        assert!(html.contains(r#"<i class="bi bi-camera-video"></i>"#));
        assert!(!html.contains("bi-camera-video-fill"));

        let html = generate_media_html("doc.pdf", "Doc", None, &MediaType::Downloadable, &icons);
        assert!(html.contains(r#"<i class="bi bi-file-earmark-arrow-down"></i>"#));
        assert!(!html.contains("bi-file-earmark-arrow-down-fill"));
    }

    #[test]
    fn test_transform_media_paragraph_to_figure() {
        let html = r#"<p><img src="image.png" alt="alt" title="Title" /></p>"#;
        let transformed = transform_images_to_media(html, &crate::parser::Icons::default(), false);
        assert!(transformed.contains(r#"<figure class="w-100">"#));
        assert!(transformed.contains("<picture"));
        assert!(transformed.contains("src=\"image.png\""));
    }

    #[test]
    fn test_transform_inline_media_remains_inline() {
        let html = r#"<p>before <img src="image.png" alt="alt" /> after</p>"#;
        let transformed = transform_images_to_media(html, &crate::parser::Icons::default(), false);
        assert!(!transformed.contains("<figure>"));
        assert!(transformed.contains("before"));
        assert!(transformed.contains("after"));
        assert!(transformed.contains("<picture"));
    }

    #[test]
    fn test_transform_fragment_extension_hint_opt_in() {
        let html = r#"<p><img src="/assets/image#.png" alt="alt" /></p>"#;
        let transformed = transform_images_to_media(html, &crate::parser::Icons::default(), true);
        assert!(transformed.contains("<picture"));
        assert!(transformed.contains("type=\"image/png\""));
    }

    #[test]
    fn test_transform_media_parameters() {
        let html = r#"<img src="movie.mp4" alt="Movie" />{75%, sha256-abc123==, sha384-def456==}"#;
        let transformed = transform_images_to_media(html, &crate::parser::Icons::default(), false);

        assert!(transformed.contains(r#"<video controls style="width: 75%">"#));
        assert!(
            transformed
                .contains(r#"type="video/mp4" integrity="sha256-abc123== sha384-def456==" />"#)
        );
        assert!(transformed.contains(
            r#"class="download-link video-fallback" integrity="sha256-abc123== sha384-def456==""#
        ));
        assert!(!transformed.contains("{75%"));
    }

    #[test]
    fn test_markdown_media_parameters_are_transformed() {
        let output = crate::parse("![Movie](movie.mp4){75%}");

        assert!(output.contains(r#"<video controls style="width: 75%">"#));
        assert!(!output.contains("{75%}"));
    }

    #[test]
    fn test_transform_image_dimensions_and_download_integrity() {
        let image = transform_images_to_media(
            r#"<img src="photo.png" alt="Photo" />{320x240}"#,
            &crate::parser::Icons::default(),
            false,
        );
        assert!(image.contains(r#"<picture width="320" height="240">"#));

        let download = transform_images_to_media(
            r#"<img src="manual.pdf" alt="Manual" />{sha512-ghi789==}"#,
            &crate::parser::Icons::default(),
            false,
        );
        assert!(download.contains(
            r#"<a href="manual.pdf" download class="download-link" integrity="sha512-ghi789==">"#
        ));
    }

    #[test]
    fn test_transform_media_parameters_ignores_invalid_and_duplicate_sizes() {
        let transformed = transform_images_to_media(
            r#"<img src="audio.mp3" alt="Audio" />{bad, 400, 800, sha256-not valid}"#,
            &crate::parser::Icons::default(),
            false,
        );

        assert!(transformed.contains(r#"<audio controls width="400">"#));
        assert!(!transformed.contains("800"));
        assert!(!transformed.contains("integrity="));
        assert!(!transformed.contains("{bad"));
    }
}
