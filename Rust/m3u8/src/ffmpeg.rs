use std::path::Path;
use std::process::Command;

#[derive(Debug)]
pub struct FFmpegMerger {
    pub ffmpeg_path: String,
}

impl Default for FFmpegMerger {
    fn default() -> Self {
        Self {
            ffmpeg_path: "ffmpeg".to_string(),
        }
    }
}

impl FFmpegMerger {
    pub fn new(ffmpeg_path: &str) -> Self {
        Self {
            ffmpeg_path: ffmpeg_path.to_string(),
        }
    }

    pub fn merge_ts_files(&self, ts_files: &[String], output_file: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::new(&self.ffmpeg_path);
        
        cmd.arg("-y")
           .arg("-f")
           .arg("concat")
           .arg("-safe")
           .arg("0")
           .arg("-i")
           .arg(self.create_file_list(ts_files)?.as_str())
           .arg("-c")
           .arg("copy")
           .arg(output_file.to_str().unwrap());

        let output = cmd.output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("FFmpeg error: {}", stderr)
            )));
        }

        Ok(())
    }

    fn create_file_list(&self, ts_files: &[String]) -> Result<String, Box<dyn std::error::Error>> {
        let file_list_content = ts_files
            .iter()
            .map(|file| format!("file '{}'", file))
            .collect::<Vec<_>>()
            .join("\n");
        
        let file_list_path = "filelist.txt";
        std::fs::write(file_list_path, file_list_content)?;
        
        Ok(file_list_path.to_string())
    }

    pub fn check_ffmpeg(&self) -> bool {
        let output = Command::new(&self.ffmpeg_path)
            .arg("-version")
            .output();
        
        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_ffmpeg() {
        let merger = FFmpegMerger::default();
        let result = merger.check_ffmpeg();
        println!("FFmpeg available: {}", result);
    }
}
