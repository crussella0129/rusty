//! A small Markdown→egui renderer for lesson prose. We own this (rather than depend
//! on `egui_commonmark`) to avoid the egui-version coupling that made `egui_term`
//! unusable. It supports the subset lesson prose uses: headings, paragraphs, inline
//! code, bold/italic, fenced code blocks, and bullet lists. Anything else degrades
//! to plain text.
//!
//! The parse step ([`to_blocks`]) is pure and unit-tested; the egui drawing
//! ([`render_markdown`]) is verified by the manual smoke check.

use pulldown_cmark::{Event, Parser, Tag, TagEnd};

/// An inline run within a paragraph or list item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MdSpan {
    Text(String),
    Bold(String),
    Italic(String),
    Code(String),
}

/// A block-level piece of rendered Markdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MdBlock {
    Heading(u8, String),
    Paragraph(Vec<MdSpan>),
    CodeBlock(String),
    Bullet(Vec<MdSpan>),
}

/// Parse Markdown into a flat list of block-level elements (pure; testable).
pub fn to_blocks(md: &str) -> Vec<MdBlock> {
    let mut blocks: Vec<MdBlock> = Vec::new();
    let mut spans: Vec<MdSpan> = Vec::new();
    let mut heading: Option<u8> = None;
    let mut heading_text = String::new();
    let mut code: Option<String> = None;
    let mut strong = false;
    let mut em = false;
    let mut in_item = false;

    let push_text = |spans: &mut Vec<MdSpan>, strong: bool, em: bool, t: &str| {
        let s = t.to_string();
        if strong {
            spans.push(MdSpan::Bold(s));
        } else if em {
            spans.push(MdSpan::Italic(s));
        } else {
            spans.push(MdSpan::Text(s));
        }
    };

    for event in Parser::new(md) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                heading = Some(level as u8);
                heading_text.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(level) = heading.take() {
                    blocks.push(MdBlock::Heading(level, heading_text.trim().to_string()));
                }
            }
            Event::Start(Tag::Paragraph) => {
                spans.clear();
            }
            Event::End(TagEnd::Paragraph) => {
                let taken = std::mem::take(&mut spans);
                if in_item {
                    blocks.push(MdBlock::Bullet(taken));
                } else {
                    blocks.push(MdBlock::Paragraph(taken));
                }
            }
            Event::Start(Tag::Item) => {
                in_item = true;
                spans.clear();
            }
            Event::End(TagEnd::Item) => {
                // Tight lists put text directly in the item (no inner paragraph).
                if !spans.is_empty() {
                    blocks.push(MdBlock::Bullet(std::mem::take(&mut spans)));
                }
                in_item = false;
            }
            Event::Start(Tag::CodeBlock(_)) => code = Some(String::new()),
            Event::End(TagEnd::CodeBlock) => {
                if let Some(src) = code.take() {
                    blocks.push(MdBlock::CodeBlock(src.trim_end().to_string()));
                }
            }
            Event::Start(Tag::Strong) => strong = true,
            Event::End(TagEnd::Strong) => strong = false,
            Event::Start(Tag::Emphasis) => em = true,
            Event::End(TagEnd::Emphasis) => em = false,
            Event::Text(t) => {
                if let Some(buf) = code.as_mut() {
                    buf.push_str(&t);
                } else if heading.is_some() {
                    heading_text.push_str(&t);
                } else {
                    push_text(&mut spans, strong, em, &t);
                }
            }
            Event::Code(t) => {
                if heading.is_some() {
                    heading_text.push_str(&t);
                } else {
                    spans.push(MdSpan::Code(t.to_string()));
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if heading.is_none() && code.is_none() {
                    spans.push(MdSpan::Text(" ".to_string()));
                }
            }
            _ => {}
        }
    }
    blocks
}

/// Render Markdown into the current egui `Ui`.
pub fn render_markdown(ui: &mut egui::Ui, md: &str) {
    for block in to_blocks(md) {
        match block {
            MdBlock::Heading(level, text) => {
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new(text)
                        .size(crate::theme::heading_size(level))
                        .strong(),
                );
            }
            MdBlock::Paragraph(spans) => {
                paragraph(ui, &spans);
            }
            MdBlock::Bullet(spans) => {
                ui.horizontal_wrapped(|ui| {
                    ui.label("• ");
                    paragraph(ui, &spans);
                });
            }
            MdBlock::CodeBlock(src) => {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.label(egui::RichText::new(src).monospace());
                });
            }
        }
        ui.add_space(4.0);
    }
}

fn paragraph(ui: &mut egui::Ui, spans: &[MdSpan]) {
    let mut job = egui::text::LayoutJob::default();
    let body = egui::FontId::proportional(crate::theme::BODY);
    let mono = egui::FontId::monospace(13.0);
    let normal = ui.visuals().text_color();
    let strong = ui.visuals().strong_text_color();
    for span in spans {
        let (text, font, color) = match span {
            MdSpan::Text(t) => (t, body.clone(), normal),
            MdSpan::Bold(t) => (t, body.clone(), strong),
            MdSpan::Italic(t) => (t, body.clone(), normal),
            MdSpan::Code(t) => (t, mono.clone(), strong),
        };
        job.append(
            text,
            0.0,
            egui::text::TextFormat {
                font_id: font,
                color,
                ..Default::default()
            },
        );
    }
    ui.label(job);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_blocks_heading_bold_code() {
        let blocks = to_blocks("# Title\n\n**bold** and `code` here");
        // A heading block.
        assert!(blocks
            .iter()
            .any(|b| matches!(b, MdBlock::Heading(1, t) if t == "Title")));
        // A paragraph containing a Bold span and a Code span.
        let para = blocks
            .iter()
            .find_map(|b| match b {
                MdBlock::Paragraph(spans) => Some(spans),
                _ => None,
            })
            .expect("a paragraph");
        assert!(para
            .iter()
            .any(|s| matches!(s, MdSpan::Bold(t) if t == "bold")));
        assert!(para
            .iter()
            .any(|s| matches!(s, MdSpan::Code(t) if t == "code")));
    }

    #[test]
    fn test_to_blocks_code_fence() {
        let blocks = to_blocks("```\nfn main() {}\n```");
        assert!(blocks
            .iter()
            .any(|b| matches!(b, MdBlock::CodeBlock(s) if s.contains("fn main"))));
    }

    #[test]
    fn test_to_blocks_plain_paragraph() {
        let blocks = to_blocks("just text");
        assert_eq!(blocks.len(), 1);
        assert!(matches!(&blocks[0], MdBlock::Paragraph(_)));
    }
}
