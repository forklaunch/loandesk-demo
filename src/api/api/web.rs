//! The front end, compiled into the binary from `web-dist` (built from
//! `frontend/` by `npm run build:embed` and committed, because the image is
//! built by a Rust toolchain with no Node).
//!
//! Where there is no web server in front of the API (the hosted demo), the API
//! answers every `GET` outside its own paths from here; a path that is not a
//! file gets `index.html`. The Compose stack keeps its nginx container, which
//! serves the same files and never forwards these paths.
use axum::{
    http::{HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use include_dir::{Dir, include_dir};

static WEB: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/web-dist");

/// What the page may load: its own origin only, as nginx sets it for Compose.
pub const PAGE_CONTENT_SECURITY_POLICY: &str =
    "default-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'";

/// Paths that belong to the API (contract routes, the OpenAPI UI, MCP) and are
/// never answered with a page.
fn is_api_path(path: &str, docs_path: &str) -> bool {
    let under = |prefix: &str| {
        path == prefix || path.strip_prefix(prefix).is_some_and(|rest| rest.starts_with('/'))
    };
    under("/api") || under("/health") || under("/mcp") || under(docs_path)
}

/// Whether this request is for the front end rather than the API.
pub fn serves(method: &Method, path: &str, docs_path: &str) -> bool {
    matches!(*method, Method::GET | Method::HEAD) && !is_api_path(path, docs_path)
}

fn content_type(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or_default() {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "map" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// The file at `path`, or `index.html`. Files under `assets/` carry a content
/// hash in their name and may be cached for good; the page itself is always
/// revalidated, so a new release is picked up on the next load.
pub fn respond(path: &str) -> Response {
    let path = path.trim_start_matches('/');
    let (name, file) = match WEB.get_file(path).filter(|_| !path.is_empty()) {
        Some(file) => (path, file),
        // A missing asset is a 404, not a page: a script tag must never be
        // answered with HTML.
        None if path.starts_with("assets/") => return StatusCode::NOT_FOUND.into_response(),
        None => match WEB.get_file("index.html") {
            Some(file) => ("index.html", file),
            None => return StatusCode::NOT_FOUND.into_response(),
        },
    };
    let cache = if name.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    let mut response = (
        [
            (header::CONTENT_TYPE, HeaderValue::from_static(content_type(name))),
            (header::CACHE_CONTROL, HeaderValue::from_static(cache)),
        ],
        file.contents(),
    )
        .into_response();
    let headers = response.headers_mut();
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(header::REFERRER_POLICY, HeaderValue::from_static("no-referrer"));
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(PAGE_CONTENT_SECURITY_POLICY),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header<'a>(response: &'a Response, name: &str) -> &'a str {
        response.headers().get(name).and_then(|value| value.to_str().ok()).unwrap_or("")
    }

    #[test]
    fn the_built_front_end_is_in_the_binary() {
        let index = WEB.get_file("index.html").expect("web-dist/index.html (npm run build:embed)");
        let html = index.contents_utf8().expect("utf-8");
        // The page loads one script and one stylesheet, both files in the bundle.
        for reference in html.split('"').filter(|part| part.starts_with("/assets/")) {
            assert!(
                WEB.get_file(reference.trim_start_matches('/')).is_some(),
                "index.html refers to {reference}, which is not in web-dist"
            );
        }
        assert!(html.contains("/assets/"), "index.html loads the bundle");
        // The page policy allows no inline script, so the page must not need one.
        assert!(!html.contains("<script>"), "no inline script");
    }

    #[test]
    fn api_paths_are_never_answered_with_a_page() {
        for path in
            ["/api/auth/me", "/api", "/api/nothing-here", "/health", "/mcp", "/docs", "/docs/x"]
        {
            assert!(!serves(&Method::GET, path, "/docs"), "{path}");
        }
        for path in ["/", "/index.html", "/assets/index.js", "/favicon.ico", "/apiary", "/healthy"]
        {
            assert!(serves(&Method::GET, path, "/docs"), "{path}");
        }
        // Only reads: a POST to a page path goes on to the API and its checks.
        assert!(!serves(&Method::POST, "/", "/docs"));
        assert!(serves(&Method::HEAD, "/", "/docs"));
    }

    #[test]
    fn pages_carry_the_security_headers_and_unknown_paths_get_the_app() {
        for path in ["/", "/index.html", "/some/client/route"] {
            let response = respond(path);
            assert_eq!(response.status(), StatusCode::OK, "{path}");
            assert_eq!(header(&response, "content-type"), "text/html; charset=utf-8");
            assert_eq!(header(&response, "content-security-policy"), PAGE_CONTENT_SECURITY_POLICY);
            assert_eq!(header(&response, "x-content-type-options"), "nosniff");
            assert_eq!(header(&response, "x-frame-options"), "DENY");
            assert_eq!(header(&response, "referrer-policy"), "no-referrer");
            assert_eq!(header(&response, "cache-control"), "no-cache");
        }
        let script = WEB
            .get_dir("assets")
            .and_then(|assets| {
                assets.files().find(|file| file.path().extension().is_some_and(|e| e == "js"))
            })
            .expect("a script in web-dist/assets");
        let response = respond(&format!("/{}", script.path().display()));
        assert_eq!(header(&response, "content-type"), "text/javascript; charset=utf-8");
        assert_eq!(header(&response, "cache-control"), "public, max-age=31536000, immutable");
        assert_eq!(header(&response, "x-content-type-options"), "nosniff");
        assert_eq!(respond("/assets/missing.js").status(), StatusCode::NOT_FOUND);
    }
}
