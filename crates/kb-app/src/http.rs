//! The app's HTTP client, for the images the panel loads: bookmark favicons and the thumbnails
//! notes carry (DESIGN.md, Adaptations). GPUI gives native apps none, so every remote image
//! failed; this one runs macOS's `curl` on a thread per request, so kb needs no HTTP crate.
//! GET only, over HTTP(S) only.

use std::process::Command;
use std::thread;

use futures::FutureExt as _;
use futures::channel::oneshot;
use futures::future::BoxFuture;
use gpui_kit::http_client::http::HeaderValue;
use gpui_kit::http_client::{AsyncBody, HttpClient, Request, Response, Result, Url, anyhow};

/// Fetches with `/usr/bin/curl`.
#[derive(Debug, Default)]
pub struct CurlClient;

impl HttpClient for CurlClient {
    fn user_agent(&self) -> Option<&HeaderValue> {
        None
    }

    fn proxy(&self) -> Option<&Url> {
        None
    }

    fn send(&self, request: Request<AsyncBody>) -> BoxFuture<'static, Result<Response<AsyncBody>>> {
        let url = request.uri().to_string();
        let (sender, receiver) = oneshot::channel();
        thread::spawn(move || {
            // The receiver is gone when GPUI no longer wants the image; nothing to report then.
            let _ = sender.send(fetch(&url));
        });
        async move {
            receiver
                .await
                .map_err(|_| anyhow!("the request for an image was dropped"))?
        }
        .boxed()
    }
}

/// GETs `url`, following redirects, giving up after ten seconds.
fn fetch(url: &str) -> Result<Response<AsyncBody>> {
    let output = Command::new("/usr/bin/curl")
        .args(["--silent", "--show-error", "--fail", "--location", "--max-time", "10"])
        // Only the web: a note's image URL must not read local files (file://) or other schemes.
        .args(["--proto", "=http,https", "--proto-redir", "=http,https", "--"])
        .arg(url)
        .output()?;
    if !output.status.success() {
        return Err(anyhow!(
            "couldn't fetch {url}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(Response::builder().status(200).body(AsyncBody::from(output.stdout))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_anything_but_the_web() {
        let Err(error) = fetch("file:///etc/hosts") else {
            panic!("a file URL was fetched");
        };
        let error = error.to_string();
        assert!(error.contains("couldn't fetch file:///etc/hosts"), "{error}");
    }

    #[test]
    fn reports_a_failed_request() {
        assert!(fetch("https://kb.invalid/missing.png").is_err());
    }
}
