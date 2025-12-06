//! Output Encoding Module for Protection Layer 3 - Output Encoding
//!
//! Implements output encoding from DEX-OS-V2.csv line 247:
//! - Security,Protection Layer,Protection Layer 3,Output Encoding,Content Security,High
//!
//! Features:
//! - HTML entity encoding
//! - JavaScript encoding
//! - URL encoding
//! - CSS encoding
//! - JSON encoding
//! - XML encoding
//! - SQL encoding
//! - Context-aware encoding

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Output encoding errors
#[derive(Debug, Error, Clone, PartialEq)]
pub enum EncodingError {
    #[error("Invalid input for encoding: {0}")]
    InvalidInput(String),
    #[error("Unsupported encoding context: {0}")]
    UnsupportedContext(String),
    #[error("Encoding failed: {0}")]
    EncodingFailed(String),
}

/// Encoding context - determines which encoding to apply
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncodingContext {
    /// HTML content (body, attributes)
    Html,
    /// HTML attributes specifically
    HtmlAttribute,
    /// JavaScript/JSON context
    JavaScript,
    /// URL parameters or path
    Url,
    /// CSS content
    Css,
    /// XML content
    Xml,
    /// SQL string literals
    Sql,
    /// JSON values
    Json,
}

/// Output encoding result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncodedOutput {
    /// Original input
    pub original: String,
    /// Encoded output
    pub encoded: String,
    /// Context used for encoding
    pub context: EncodingContext,
    /// Whether encoding was applied
    pub was_encoded: bool,
}

/// Output encoder manager
#[derive(Debug, Clone)]
pub struct OutputEncoder {
    /// HTML entity map
    html_entities: HashMap<char, String>,
    /// JavaScript escape sequences
    js_escapes: HashMap<char, String>,
}

impl OutputEncoder {
    /// Create a new output encoder
    pub fn new() -> Self {
        Self {
            html_entities: Self::build_html_entities(),
            js_escapes: Self::build_js_escapes(),
        }
    }

    /// Build HTML entity mapping
    fn build_html_entities() -> HashMap<char, String> {
        let mut entities = HashMap::new();
        entities.insert('&', "&amp;".to_string());
        entities.insert('<', "&lt;".to_string());
        entities.insert('>', "&gt;".to_string());
        entities.insert('"', "&quot;".to_string());
        entities.insert('\'', "&#x27;".to_string());
        entities.insert('/', "&#x2F;".to_string());
        entities
    }

    /// Build JavaScript escape sequences
    fn build_js_escapes() -> HashMap<char, String> {
        let mut escapes = HashMap::new();
        escapes.insert('"', "\\\"".to_string());
        escapes.insert('\'', "\\'".to_string());
        escapes.insert('\\', "\\\\".to_string());
        escapes.insert('\n', "\\n".to_string());
        escapes.insert('\r', "\\r".to_string());
        escapes.insert('\t', "\\t".to_string());
        escapes.insert('\x08', "\\b".to_string());
        escapes.insert('\x0C', "\\f".to_string());
        escapes
    }

    /// Encode for HTML context
    pub fn encode_html(&self, input: &str) -> String {
        let mut encoded = String::with_capacity(input.len());

        for c in input.chars() {
            if let Some(mapped) = self.html_entities.get(&c) {
                encoded.push_str(mapped);
                continue;
            }

            if c.is_control() && c != '\n' && c != '\r' && c != '\t' {
                encoded.push_str(&format!("&#{};", c as u32));
                continue;
            }

            if (c as u32) > 127 {
                encoded.push_str(&format!("&#{};", c as u32));
                continue;
            }

            encoded.push(c);
        }

        encoded
    }

    /// Encode for HTML attribute context
    pub fn encode_html_attribute(&self, input: &str) -> String {
        // More restrictive - encode more characters
        input.chars()
            .map(|c| {
                match c {
                    '&' => "&amp;".to_string(),
                    '<' => "&lt;".to_string(),
                    '>' => "&gt;".to_string(),
                    '"' => "&quot;".to_string(),
                    '\'' => "&#x27;".to_string(),
                    '=' => "&#x3D;".to_string(),
                    '`' => "&#x60;".to_string(),
                    _ => {
                        if c.is_control() || (c as u32) > 127 {
                            format!("&#{};", c as u32)
                        } else {
                            c.to_string()
                        }
                    }
                }
            })
            .collect::<Vec<String>>()
            .join("")
    }

    /// Encode for JavaScript context
    pub fn encode_javascript(&self, input: &str) -> String {
        let mut encoded = String::with_capacity(input.len());

        for c in input.chars() {
            if let Some(mapped) = self.js_escapes.get(&c) {
                encoded.push_str(mapped);
                continue;
            }

            if c.is_control() || (c as u32) > 127 {
                encoded.push_str(&format!("\\u{:04x}", c as u32));
                continue;
            }

            match c {
                '<' => encoded.push_str("\\u003c"),
                '>' => encoded.push_str("\\u003e"),
                '&' => encoded.push_str("\\u0026"),
                _ => encoded.push(c),
            }
        }

        encoded
    }

    /// Encode for URL context
    pub fn encode_url(&self, input: &str) -> String {
        input.chars()
            .map(|c| {
                match c {
                    'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                    _ => format!("%{:02X}", c as u8),
                }
            })
            .collect::<Vec<String>>()
            .join("")
    }

    /// Encode for CSS context
    pub fn encode_css(&self, input: &str) -> String {
        input.chars()
            .map(|c| {
                match c {
                    'A'..='Z' | 'a'..='z' | '0'..='9' => c.to_string(),
                    _ => format!("\\{:x} ", c as u32),
                }
            })
            .collect::<Vec<String>>()
            .join("")
    }

    /// Encode for XML context
    pub fn encode_xml(&self, input: &str) -> String {
        input.chars()
            .map(|c| match c {
                '&' => "&amp;".to_string(),
                '<' => "&lt;".to_string(),
                '>' => "&gt;".to_string(),
                '"' => "&quot;".to_string(),
                '\'' => "&apos;".to_string(),
                _ => {
                    if c.is_control() && c != '\n' && c != '\r' && c != '\t' {
                        format!("&#x{:x};", c as u32)
                    } else {
                        c.to_string()
                    }
                }
            })
            .collect::<Vec<String>>()
            .join("")
    }

    /// Encode for SQL context
    pub fn encode_sql(&self, input: &str) -> String {
        input.chars()
            .map(|c| match c {
                '\'' => "''".to_string(),
                '\\' => "\\\\".to_string(),
                '\0' => "".to_string(),
                '\n' => "\\n".to_string(),
                '\r' => "\\r".to_string(),
                _ => c.to_string(),
            })
            .collect::<Vec<String>>()
            .join("")
    }

    /// Encode for JSON context
    pub fn encode_json(&self, input: &str) -> String {
        input.chars()
            .map(|c| match c {
                '"' => "\\\"".to_string(),
                '\\' => "\\\\".to_string(),
                '\n' => "\\n".to_string(),
                '\r' => "\\r".to_string(),
                '\t' => "\\t".to_string(),
                '\x08' => "\\b".to_string(),
                '\x0C' => "\\f".to_string(),
                _ => {
                    if c.is_control() {
                        format!("\\u{:04x}", c as u32)
                    } else {
                        c.to_string()
                    }
                }
            })
            .collect::<Vec<String>>()
            .join("")
    }

    /// Encode based on context
    pub fn encode(&self, input: &str, context: EncodingContext) -> EncodedOutput {
        let original = input.to_string();
        
        let encoded = match context {
            EncodingContext::Html => self.encode_html(input),
            EncodingContext::HtmlAttribute => self.encode_html_attribute(input),
            EncodingContext::JavaScript => self.encode_javascript(input),
            EncodingContext::Url => self.encode_url(input),
            EncodingContext::Css => self.encode_css(input),
            EncodingContext::Xml => self.encode_xml(input),
            EncodingContext::Sql => self.encode_sql(input),
            EncodingContext::Json => self.encode_json(input),
        };

        EncodedOutput {
            original: original.clone(),
            encoded: encoded.clone(),
            context,
            was_encoded: original != encoded,
        }
    }

    /// Decode HTML entities (for testing/verification)
    pub fn decode_html(&self, input: &str) -> String {
        let mut result = input.to_string();
        
        result = result.replace("&amp;", "&");
        result = result.replace("&lt;", "<");
        result = result.replace("&gt;", ">");
        result = result.replace("&quot;", "\"");
        result = result.replace("&#x27;", "'");
        result = result.replace("&#x2F;", "/");
        
        // Handle numeric entities
        let re = regex::Regex::new(r"&#(\d+);").unwrap();
        result = re.replace_all(&result, |caps: &regex::Captures| {
            if let Some(num_str) = caps.get(1) {
                if let Ok(num) = num_str.as_str().parse::<u32>() {
                    if let Some(ch) = char::from_u32(num) {
                        return ch.to_string();
                    }
                }
            }
            caps[0].to_string()
        }).to_string();
        
        result
    }

    /// Build safe HTML with encoded values
    pub fn build_safe_html(&self, template: &str, values: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        
        for (key, value) in values {
            let placeholder = format!("{{{{{}}}}}", key);
            let encoded = self.encode_html(value);
            result = result.replace(&placeholder, &encoded);
        }
        
        result
    }

    /// Build safe JavaScript string
    pub fn build_safe_js_string(&self, value: &str) -> String {
        format!("\"{}\"", self.encode_javascript(value))
    }

    /// Build safe URL with encoded parameters
    pub fn build_safe_url(&self, base: &str, params: &HashMap<String, String>) -> String {
        if params.is_empty() {
            return base.to_string();
        }

        let encoded_params: Vec<String> = params.iter()
            .map(|(k, v)| {
                format!("{}={}", 
                    self.encode_url(k),
                    self.encode_url(v)
                )
            })
            .collect();

        format!("{}?{}", base, encoded_params.join("&"))
    }
}

impl Default for OutputEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_encoding() {
        let encoder = OutputEncoder::new();
        
        let input = "<script>alert('XSS')</script>";
        let encoded = encoder.encode_html(input);
        
        assert!(!encoded.contains("<script>"));
        assert!(encoded.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_html_attribute_encoding() {
        let encoder = OutputEncoder::new();
        
        let input = "value\" onload=\"alert(1)";
        let encoded = encoder.encode_html_attribute(input);
        
        assert!(encoded.contains("&quot;"));
        assert!(!encoded.contains("\""));
    }

    #[test]
    fn test_javascript_encoding() {
        let encoder = OutputEncoder::new();
        
        let input = "'; alert('XSS'); //";
        let encoded = encoder.encode_javascript(input);
        
        assert!(encoded.contains("\\'"));
        assert!(!encoded.contains("';"));
    }

    #[test]
    fn test_url_encoding() {
        let encoder = OutputEncoder::new();
        
        let input = "hello world&foo=bar";
        let encoded = encoder.encode_url(input);
        
        assert!(encoded.contains("%20")); // space
        assert!(encoded.contains("%26")); // &
        assert!(encoded.contains("%3D")); // =
    }

    #[test]
    fn test_css_encoding() {
        let encoder = OutputEncoder::new();
        
        let input = "expression(alert(1))";
        let encoded = encoder.encode_css(input);
        
        // Special characters should be escaped
        assert!(encoded.contains("\\"));
    }

    #[test]
    fn test_xml_encoding() {
        let encoder = OutputEncoder::new();
        
        let input = "<tag attr='value'>&data</tag>";
        let encoded = encoder.encode_xml(input);
        
        assert!(encoded.contains("&lt;"));
        assert!(encoded.contains("&gt;"));
        assert!(encoded.contains("&apos;"));
        assert!(encoded.contains("&amp;"));
    }

    #[test]
    fn test_sql_encoding() {
        let encoder = OutputEncoder::new();
        
        let input = "O'Reilly";
        let encoded = encoder.encode_sql(input);
        
        assert_eq!(encoded, "O''Reilly");
    }

    #[test]
    fn test_json_encoding() {
        let encoder = OutputEncoder::new();
        
        let input = "Line 1\nLine 2\tTabbed";
        let encoded = encoder.encode_json(input);
        
        assert!(encoded.contains("\\n"));
        assert!(encoded.contains("\\t"));
    }

    #[test]
    fn test_context_aware_encoding() {
        let encoder = OutputEncoder::new();
        
        let input = "<script>alert('test')</script>";
        
        let html_result = encoder.encode(input, EncodingContext::Html);
        assert!(html_result.was_encoded);
        assert!(html_result.encoded.contains("&lt;"));
        
        let js_result = encoder.encode(input, EncodingContext::JavaScript);
        assert!(js_result.was_encoded);
        assert!(js_result.encoded.contains("\\u003c"));
    }

    #[test]
    fn test_safe_html_builder() {
        let encoder = OutputEncoder::new();
        
        let mut values = HashMap::new();
        values.insert("name".to_string(), "<script>alert(1)</script>".to_string());
        values.insert("message".to_string(), "Hello & goodbye".to_string());
        
        let template = "<div>{{name}}: {{message}}</div>";
        let safe_html = encoder.build_safe_html(template, &values);
        
        assert!(!safe_html.contains("<script>"));
        assert!(safe_html.contains("&lt;script&gt;"));
        assert!(safe_html.contains("&amp;"));
    }

    #[test]
    fn test_safe_url_builder() {
        let encoder = OutputEncoder::new();
        
        let mut params = HashMap::new();
        params.insert("q".to_string(), "hello world".to_string());
        params.insert("filter".to_string(), "type=user&active=true".to_string());
        
        let url = encoder.build_safe_url("https://example.com/search", &params);
        
        assert!(url.contains("%20")); // encoded space
        assert!(url.contains("%3D")); // encoded =
        assert!(url.contains("%26")); // encoded &
    }

    #[test]
    fn test_html_decode() {
        let encoder = OutputEncoder::new();
        
        let encoded = "&lt;script&gt;alert(&#x27;test&#x27;)&lt;/script&gt;";
        let decoded = encoder.decode_html(encoded);
        
        assert_eq!(decoded, "<script>alert('test')</script>");
    }
}
