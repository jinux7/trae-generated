mod m3u8;
mod downloader;
mod ffmpeg;

use std::path::Path;

#[tokio::main]
async fn main() {
    use std::io::{self, BufRead};
    
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    
    println!("=== M3U8 Downloader ===");
    println!("请输入m3u8文件地址:");
    let mut m3u8_url = String::new();
    reader.read_line(&mut m3u8_url).expect("无法读取输入");
    let m3u8_url = m3u8_url.trim();
    
    if m3u8_url.is_empty() {
        println!("错误: m3u8地址不能为空");
        return;
    }
    
    println!("请输入输出文件名 (请包含文件扩展名，如: output.mp4):");
    let mut output_file_str = String::new();
    reader.read_line(&mut output_file_str).expect("无法读取输入");
    let output_file_str = output_file_str.trim();
    
    if output_file_str.is_empty() {
        println!("错误: 输出文件名不能为空");
        return;
    }
    
    if !output_file_str.contains('.') {
        println!("错误: 输出文件名必须包含文件扩展名，如: output.mp4");
        return;
    }
    
    let output_file = Path::new(output_file_str);
    let output_dir = Path::new("./downloads");

    println!("Starting m3u8 downloader...");
    println!("M3U8 URL: {}", m3u8_url);
    println!("Output file: {}", output_file.to_str().unwrap());
    println!("Download directory: {}", output_dir.to_str().unwrap());

    // Step 1: Parse m3u8 file
    println!("\nStep 1: Parsing m3u8 file...");
    let base_url = if m3u8_url.contains('/') {
        if let Some(last_slash) = m3u8_url.rfind('/') {
            m3u8_url[..last_slash].to_string()
        } else {
            m3u8_url.to_string()
        }
    } else {
        m3u8_url.to_string()
    };
    let parser = m3u8::M3U8Parser::new(&base_url);
    
    let segments = match parser.parse_from_url(m3u8_url).await {
        Ok(segments) => {
            println!("Found {} ts segments", segments.len());
            segments
        }
        Err(e) => {
            println!("Error parsing m3u8 file: {}", e);
            return;
        }
    };

    if segments.is_empty() {
        println!("No ts segments found in m3u8 file");
        return;
    }

    // Step 2: Download ts files
    println!("\nStep 2: Downloading ts segments...");
    let downloader = downloader::Downloader::default();
    let results = downloader.download_all(&segments, output_dir).await;

    // Collect successful downloads
    let mut successful_downloads = Vec::new();
    let mut failed_count = 0;

    for (idx, result) in results {
        match result {
            Ok(path) => {
                successful_downloads.push(path);
            }
            Err(e) => {
                println!("Failed to download segment {}: {}", idx, e);
                failed_count += 1;
            }
        }
    }

    println!("\nDownload completed:");
    println!("Successful: {}", successful_downloads.len());
    println!("Failed: {}", failed_count);

    if successful_downloads.is_empty() {
        println!("No segments downloaded successfully");
        return;
    }

    // Step 3: Merge ts files using FFmpeg
    println!("\nStep 3: Merging ts files with FFmpeg...");
    let merger = ffmpeg::FFmpegMerger::default();

    if !merger.check_ffmpeg() {
        println!("Error: FFmpeg not found. Please install FFmpeg and add it to PATH.");
        println!("You can download FFmpeg from https://ffmpeg.org/download.html");
        return;
    }

    match merger.merge_ts_files(&successful_downloads, output_file) {
        Ok(_) => {
            println!("Successfully merged ts files into {}", output_file.to_str().unwrap());
            
            // Clean up ts cache files
            println!("\n正在清理ts缓存文件...");
            let mut deleted_count = 0;
            let mut failed_count = 0;
            
            for ts_file in &successful_downloads {
                if let Err(e) = std::fs::remove_file(ts_file) {
                    println!("删除文件 {} 失败: {}", ts_file, e);
                    failed_count += 1;
                } else {
                    deleted_count += 1;
                }
            }
            
            println!("清理完成: 成功删除 {} 个文件, 失败 {} 个文件", deleted_count, failed_count);
            println!("\nDownload and merge completed successfully!");
        }
        Err(e) => {
            println!("Error merging ts files: {}", e);
            return;
        }
    }

    // Clean up filelist.txt
    let filelist_path = Path::new("filelist.txt");
    if filelist_path.exists() {
        std::fs::remove_file(filelist_path).unwrap_or_default();
    }

    println!("\nDone!");
}

