use crate::errors::error::IsError;
use crate::errors::error::IsError::Scrape;
use once_cell::sync::Lazy;
use reqwest::blocking::Client;
use std::process::Command;

static REQWEST_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .http1_only()
        .build()
        .expect("Failed to build reqwest client")
});

pub fn scrape(url: &str) -> Result<String, IsError> {
    reqwest_scrape(url)
        .or_else(|_| curl_scrape(url))
        .map(|html| sanitize(&html))
}

fn reqwest_scrape(url: &str) -> Result<String, IsError> {
    REQWEST_CLIENT
        .get(url)
        .header("User-Agent", "Mozilla/5.0")
        .header("Accept", "text/html,application/xhtml+xml")
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .and_then(|resp| resp.error_for_status()) // Ensure HTTP errors are caught
        .and_then(|resp| resp.text())
        .map_err(|e| Scrape(format!("Request failed for {}: {}", url, e)))
}

// Some sites seem to be more comfortable serving curl rather than reqwest
fn curl_scrape(url: &str) -> Result<String, IsError> {
    let output = Command::new("curl")
        .args([
            "-A",
            "Mozilla/5.0 (compatible; MSIE 7.01; Windows NT 5.0)",
            url,
        ])
        .output()
        .map_err(|e| IsError::Scrape(e.to_string()))?;

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn sanitize(html: &str) -> String {
    html.replace("\t", "    ")
        .replace("\r", "")
        .replace('\u{feff}', "")
}
