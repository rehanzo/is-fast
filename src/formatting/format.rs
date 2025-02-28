use crate::config::load::Config;
use crate::formatting::syntax_highlight::highlight_code;
use once_cell::sync::Lazy;
use ratatui::style::{Style, Styled};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;
use scraper::{ElementRef, Html, Node, Selector};
use std::collections::{HashMap, HashSet};

static IGNORED_TAGS: Lazy<&HashSet<String>> = Lazy::new(|| Config::get_ignored_tags());
static BLOCK_ELEMENTS: Lazy<&HashSet<String>> = Lazy::new(|| Config::get_block_elements());
static TAG_STYLES: Lazy<&HashMap<String, Style>> = Lazy::new(Config::get_styles);

pub fn to_display(url: &String, res: &String) -> Result<Paragraph<'static>, String> {
    let selection_tag = Config::get_selectors(url).unwrap_or_else(|| "body".to_string());
    let selector =
        Selector::parse(&selection_tag).map_err(|_| "Error: Could not parse selector")?;
    let mut lines = Html::parse_document(&res)
        .select(&selector)
        .flat_map(|e| to_lines(e, e.value().name() == "pre"))
        .map(|line| standardize_empty(line))
        .collect::<Vec<Line>>();
    lines.dedup();
    if let Some(first) = lines.first() {
        if first.spans.iter().all(|span| span.content.trim().is_empty()) {
            lines.remove(0);
        }
    }
    if lines.is_empty() {
        return Err("No content found".to_string());
    }
    Ok(Paragraph::new(Text::from(lines)))
}

fn to_lines(element: ElementRef, pre_formatted: bool) -> Vec<Line<'static>> {
    if is_hidden(&element) {
        return vec![];
    }
    let tag_name = element.value().name();

    if IGNORED_TAGS.contains(tag_name) {
        return vec![];
    }

    if tag_name == "br" {
        return vec![
            // Must return 2 lines - the first will be merged back into the previous line,
            // and the second will be the start of the next line.
            // Must be treated differently to block elements as it requires no empty line.
            Line::default(),
            Line::from(Span::from("")),
        ];
    }

    let style = TAG_STYLES.get(tag_name);

    if tag_name == "img" { // Show there is an image without rendering the image.
        return vec![create_optionally_styled_line("IMAGE", style)];
    }

    let mut lines = Vec::new();

    element.children().for_each(|node| match node.value() {
        Node::Text(text) => {
            let mut lines1 = &mut lines;
            if pre_formatted || tag_name == "pre" || !text.trim().is_empty() {
                let mut current_lines = text
                    .split_inclusive('\n')
                    .map(|line| create_optionally_styled_line(&*line.to_string(), style))
                    .collect::<Vec<Line>>();
                merge_with_previous_line(&mut lines1, &mut current_lines);
            }
        }
        Node::Element(_) => ElementRef::wrap(node).iter().for_each(|element| {
            let mut element_lines = to_lines(*element, pre_formatted || tag_name == "pre");
            if element_lines.is_empty() {
                return;
            }
            if BLOCK_ELEMENTS.contains(element.value().name()) {
                lines.extend(element_lines);
                return;
            }
            merge_with_previous_line(&mut lines, &mut element_lines);
        }),
        _ => {}
    });
    if lines.is_empty() {
        return vec![];
    }
    if let Some(styled) = style {
        lines = lines
            .into_iter()
            .map(|line| line.set_style(*styled))
            .collect();
    }
    if BLOCK_ELEMENTS.contains(tag_name) && !lines.is_empty() {
        lines.insert(0, Line::default());
        lines.push(Line::default());
    }
    if tag_name == "code" {
        let language_type = extract_language_type(element);
        let code_text = lines
            .iter()
            .map(|line| line.spans.iter().map(|span| span.content.clone()).collect())
            .collect::<Vec<String>>()
            .join("");
        return highlight_code(&code_text, &language_type);
    }
    lines
}

fn extract_language_type(element: ElementRef) -> String {
    element.value().attr("class")
        .into_iter()
        .flat_map(|class_attr| class_attr.split_whitespace())
        .filter(|class_name| class_name.starts_with("language-") || class_name.starts_with("lang-"))
        .map(|class_name| class_name.replace("language-", "").replace("lang-", ""))
        .next()
        .unwrap_or_else(|| "not-found".to_string())
}

fn is_hidden(element: &ElementRef) -> bool {
    if element.value().attr("hidden") == Some("true") {
        return true;
    }
    if let Some(style) = element.value().attr("style") {
        if style.contains("display: none") || style.contains("visibility: hidden") {
            return true;
        }
    }
    if element.value().attr("aria-hidden") == Some("true") {
        return true;
    }
    false
}

fn standardize_empty(line: Line) -> Line {
    if line.spans.is_empty() || line.spans.iter().all(|span| span.content.trim().is_empty()) {
        Line::default()
    } else {
        line
    }
}

fn create_optionally_styled_line(content: &str, style: Option<&Style>) -> Line<'static> {
    if let Some(style) = style {
        Line::from(Span::styled(content.to_string(), *style))
    } else {
        Line::from(Span::from(content.to_string()))
    }
}

fn merge_with_previous_line(lines: &mut Vec<Line<'static>>, new_lines: &mut Vec<Line<'static>>) {
    if new_lines.is_empty() {
        return;
    }
    if let Some(last_line) = lines.pop() {
        let mut spans = last_line.spans.clone();
        spans.extend(new_lines.first().unwrap().spans.clone());
        lines.push(Line::from(spans));
        lines.extend(new_lines.drain(1..));
    } else {
        lines.extend(new_lines.drain(..));
    }
}
