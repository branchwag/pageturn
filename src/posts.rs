use crate::post::BlogPost;
use pulldown_cmark::{html, Parser};

include!(concat!(env!("OUT_DIR"), "/generated_posts.rs"));

pub fn load_posts() -> Vec<BlogPost> {
    let mut posts = Vec::new();

    for (id, content) in POST_FILES.iter().enumerate() {
        if let Some(post) = parse_post(id as u32 + 1, content) {
            posts.push(post);
        }
    }

    posts
}

fn parse_post(id: u32, content: &str) -> Option<BlogPost> {
    let (frontmatter, markdown) = split_frontmatter(content)?;

    let title = extract_field(&frontmatter, "title")?;
    let author = extract_field(&frontmatter, "author")?;
    let date = extract_field(&frontmatter, "date")?;

    let parser = Parser::new(markdown);
    let mut html_content = String::new();
    html::push_html(&mut html_content, parser);

    // Extract plain text for display in egui (simplified approach)
    let plain_text = markdown.to_string();

    let summary = markdown.lines().find(|l| !l.trim().is_empty())?.to_string();

    Some(BlogPost {
        id,
        title,
        author,
        date,
        content: plain_text, // Use plain text for egui display
        summary,
    })
}

fn split_frontmatter(content: &str) -> Option<(String, &str)> {
    let content = content.trim();
    if !content.starts_with("---") {
        return None;
    }

    let rest = &content[3..];
    let end = rest.find("---")?;
    let frontmatter = rest[..end].to_string();
    let markdown = &rest[end + 3..].trim();

    Some((frontmatter, markdown))
}

fn extract_field(frontmatter: &str, field: &str) -> Option<String> {
    frontmatter
        .lines()
        .find(|line| line.starts_with(&format!("{}: ", field)))
        .map(|line| line[field.len() + 2..].trim().to_string())
}
