use crate::formatting::format::to_display;
use crate::links::link::Link;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use ratatui::prelude::Text;
use ratatui::widgets::Paragraph;
use std::thread;

static CACHE: Lazy<DashMap<String, Paragraph>> = Lazy::new(DashMap::new);

pub fn get_content(link: &Link) -> Paragraph<'static> {
    CACHE
        .get(&link.url)
        .map(|reference| reference.value().clone())
        .unwrap_or_else(|| {
            (link.convert_to_html)()
                .and_then(|html| to_display(&link.url, &html))
                .map(|result| {
                    let paragraph = Paragraph::new(result);
                    CACHE.insert(link.clone().url, paragraph.clone());
                    paragraph
                })
                .unwrap_or_else(|e| Paragraph::new(Text::from(e.to_string())))
        })
}

pub fn preload(link: &Link) {
    let clone = link.clone();
    thread::spawn(move || {
        get_content(&clone);
    });
}
