use std::path::Path;
use tokio::time::{timeout, Duration};
use tokio::io::AsyncWriteExt;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug)]
pub struct Downloader {
    pub concurrency: usize,
    pub timeout_seconds: u64,
}

impl Default for Downloader {
    fn default() -> Self {
        Self {
            concurrency: 20,
            timeout_seconds: 30,
        }
    }
}

impl Downloader {
    pub fn new(concurrency: usize, timeout_seconds: u64) -> Self {
        Self {
            concurrency,
            timeout_seconds,
        }
    }

    pub async fn download_all(&self, segments: &[crate::m3u8::TsSegment], output_dir: &Path) -> Vec<(usize, Result<String, Box<dyn std::error::Error>>)> {
        tokio::fs::create_dir_all(output_dir).await.unwrap_or_default();

        let total = segments.len();
        let pb = ProgressBar::new(total as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("=> "));

        let results = stream::iter(segments.iter().enumerate())
            .map(|(idx, segment)| {
                let output_dir = output_dir.to_path_buf();
                let timeout_seconds = self.timeout_seconds;
                async move {
                    let result = self.download_segment(segment, &output_dir, timeout_seconds).await;
                    (idx, result)
                }
            })
            .buffer_unordered(self.concurrency)
            .inspect(|_| pb.inc(1))
            .collect::<Vec<_>>()
            .await;

        pb.finish_with_message("Download completed");
        results
    }

    async fn download_segment(&self, segment: &crate::m3u8::TsSegment, output_dir: &Path, timeout_seconds: u64) -> Result<String, Box<dyn std::error::Error>> {
        let filename = format!("segment_{:06}.ts", segment.index);
        let output_path = output_dir.join(filename);
        let output_path_str = output_path.to_str().unwrap().to_string();

        let client = reqwest::Client::new();

        let download_task = async move {
            let response = client.get(&segment.url).send().await?;
            let mut file = tokio::fs::File::create(&output_path).await?;
            let content = response.bytes().await?;
            file.write_all(&content).await?;
            Ok(output_path_str)
        };

        match timeout(Duration::from_secs(timeout_seconds), download_task).await {
            Ok(Ok(path)) => Ok(path),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::TimedOut, "Download timeout"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_downloader() {
        let downloader = Downloader::default();
        let segments = vec![
            crate::m3u8::TsSegment {
                url: "https://example.com/test1.ts".to_string(),
                index: 0,
            },
            crate::m3u8::TsSegment {
                url: "https://example.com/test2.ts".to_string(),
                index: 1,
            },
        ];
        let output_dir = Path::new("./test_output");
        
        let results = downloader.download_all(&segments, output_dir).await;
        assert_eq!(results.len(), 2);
    }
}
