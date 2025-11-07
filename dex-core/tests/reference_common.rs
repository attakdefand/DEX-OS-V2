use std::path::{Path, PathBuf};

pub fn reference_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("dex-core must have a parent directory")
        .join(".reference")
}

pub fn slugify(input: &str) -> String {
    let normalized = input
        .replace(" & ", " and ")
        .replace('&', " and ")
        .to_lowercase();

    let mut slug = String::with_capacity(normalized.len());
    for ch in normalized.chars() {
        match ch {
            'a'..='z' | '0'..='9' => slug.push(ch),
            '_' => slug.push('_'),
            _ => slug.push('_'),
        }
    }

    slug.trim_matches('_').to_string()
}

#[allow(dead_code)]
pub fn relaxed_slug(input: &str) -> String {
    let base = slugify(input);
    let mut compact = String::with_capacity(base.len());
    let mut prev_was_underscore = false;

    for ch in base.chars() {
        if ch == '_' {
            if !prev_was_underscore {
                compact.push(ch);
                prev_was_underscore = true;
            }
        } else {
            prev_was_underscore = false;
            compact.push(ch);
        }
    }

    compact
}
