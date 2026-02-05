use regex::Regex;
use std::path::Path;

#[derive(Debug)]
pub struct M3U8Parser {
    pub base_url: String,
}

#[derive(Debug)]
pub struct TsSegment {
    pub url: String,
    pub index: usize,
}

impl M3U8Parser {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
        }
    }

    pub async fn parse_from_url(&self, url: &str) -> Result<Vec<TsSegment>, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;
        let content = response.text().await?;
        self.parse(&content)
    }

    pub fn parse_from_file(&self, path: &Path) -> Result<Vec<TsSegment>, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        self.parse(&content)
    }

    pub fn parse(&self, content: &str) -> Result<Vec<TsSegment>, Box<dyn std::error::Error>> {
        let mut segments = Vec::new();
        let ts_regex = Regex::new(r#"(?:")?([^"]+\.ts)(?:")?"#)?;
        
        for (index, line) in content.lines().enumerate() {
            if let Some(captures) = ts_regex.captures(line) {
                let ts_url = captures.get(1).unwrap().as_str();
                let full_url = self.make_full_url(ts_url);
                segments.push(TsSegment {
                    url: full_url,
                    index,
                });
            }
        }
        
        Ok(segments)
    }

    fn make_full_url(&self, ts_url: &str) -> String {
        if ts_url.starts_with("http://") || ts_url.starts_with("https://") {
            ts_url.to_string()
        } else if ts_url.starts_with("/") {
            if let Some(base) = self.base_url.split("://").nth(1) {
                if let Some(domain) = base.split("/").nth(0) {
                    format!("{}://{}{}", self.base_url.split("://").nth(0).unwrap_or("https"), domain, ts_url)
                } else {
                    format!("{}{}", self.base_url, ts_url)
                }
            } else {
                format!("{}{}", self.base_url, ts_url)
            }
        } else {
            if self.base_url.ends_with("/") {
                format!("{}{}", self.base_url, ts_url)
            } else {
                format!("{}/{}", self.base_url, ts_url)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_m3u8_content() {
        let content = r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:10
#EXT-X-MEDIA-SEQUENCE:0
#EXTINF:10.0,
segment0.ts
#EXTINF:10.0,
segment1.ts
#EXTINF:10.0,
segment2.ts
#EXT-X-ENDLIST"#;
        
        let parser = M3U8Parser::new("https://example.com/video/");
        let segments = parser.parse(content).unwrap();
        
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].url, "https://example.com/video/segment0.ts");
        assert_eq!(segments[1].url, "https://example.com/video/segment1.ts");
        assert_eq!(segments[2].url, "https://example.com/video/segment2.ts");
    }
}
