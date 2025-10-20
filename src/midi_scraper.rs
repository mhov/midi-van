use reqwest;
use scraper::{Html, Selector};
use anyhow::{Result, anyhow};
use std::collections::HashSet;

pub async fn fetch_midi_urls() -> Result<Vec<String>> {
    println!("Fetching MIDI file URLs from piano-midi.de...");

    let client = reqwest::Client::new();
    let response = client
        .get("http://piano-midi.de/midi_files.htm")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to fetch page: {}", response.status()));
    }

    let html = response.text().await?;
    let document = Html::parse_document(&html);

    let mut midi_urls = HashSet::new();

    let link_selector = Selector::parse("a[href$='.mid']").unwrap();
    for element in document.select(&link_selector) {
        if let Some(href) = element.value().attr("href") {
            let full_url = if href.starts_with("http") {
                href.to_string()
            } else if href.starts_with("/") {
                format!("http://piano-midi.de{}", href)
            } else {
                format!("http://piano-midi.de/{}", href)
            };
            midi_urls.insert(full_url);
        }
    }

    let link_selector2 = Selector::parse("a[href*='.mid']").unwrap();
    for element in document.select(&link_selector2) {
        if let Some(href) = element.value().attr("href") {
            if href.contains(".mid") {
                let full_url = if href.starts_with("http") {
                    href.to_string()
                } else if href.starts_with("/") {
                    format!("http://piano-midi.de{}", href)
                } else {
                    format!("http://piano-midi.de/{}", href)
                };
                midi_urls.insert(full_url);
            }
        }
    }

    if midi_urls.is_empty() {
        println!("No MIDI URLs found on main page, trying composer pages...");

        let composer_links = vec![
            "http://piano-midi.de/bach.htm",
            "http://piano-midi.de/mozart.htm",
            "http://piano-midi.de/beethoven.htm",
            "http://piano-midi.de/chopin.htm",
            "http://piano-midi.de/liszt.htm",
            "http://piano-midi.de/schumann.htm",
            "http://piano-midi.de/brahms.htm",
            "http://piano-midi.de/debussy.htm",
        ];

        for composer_url in composer_links {
            if let Ok(urls) = fetch_midi_urls_from_page(&client, composer_url).await {
                for url in urls {
                    midi_urls.insert(url);
                }
            }
        }
    }

    let urls: Vec<String> = midi_urls.into_iter().collect();
    println!("Found {} MIDI file URLs", urls.len());

    if urls.is_empty() {
        Err(anyhow!("No MIDI files found"))
    } else {
        Ok(urls)
    }
}

async fn fetch_midi_urls_from_page(client: &reqwest::Client, url: &str) -> Result<Vec<String>> {
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Ok(vec![]);
    }

    let html = response.text().await?;
    let document = Html::parse_document(&html);

    let mut urls = vec![];

    let link_selector = Selector::parse("a[href$='.mid'], a[href*='.mid']").unwrap();
    for element in document.select(&link_selector) {
        if let Some(href) = element.value().attr("href") {
            if href.contains(".mid") {
                let full_url = if href.starts_with("http") {
                    href.to_string()
                } else if href.starts_with("/") {
                    format!("http://piano-midi.de{}", href)
                } else {
                    format!("http://piano-midi.de/{}", href)
                };
                urls.push(full_url);
            }
        }
    }

    Ok(urls)
}

pub async fn download_midi_file(url: &str, cache_dir: &str) -> Result<String> {
    std::fs::create_dir_all(cache_dir)?;

    let file_name = url.split('/').last().unwrap_or("unknown.mid");
    let file_path = format!("{}/{}", cache_dir, file_name);

    if std::path::Path::new(&file_path).exists() {
        return Ok(file_path);
    }

    println!("Downloading: {}", url);

    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to download MIDI file: {}", response.status()));
    }

    let bytes = response.bytes().await?;
    std::fs::write(&file_path, bytes)?;

    Ok(file_path)
}