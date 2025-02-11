pub use html_sanitize_derive::HtmlSanitize;

pub trait HtmlSanitize {
    fn sanitize(&self) -> Self;
}

pub fn sanitize_html_string(html_string: &str) -> String {
    ammonia::clean(html_string)
}

pub fn sanitize_html_option_string(html_string: Option<&String>) -> Option<String> {
    html_string.map(|s| sanitize_html_string(s))
}