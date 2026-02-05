use tauri::Emitter;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Semaphore;

use reqwest::Client;
use tokio::task;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use futures::future::BoxFuture;

#[tauri::command]
async fn select_directory() -> Result<String, String> {
  use rfd::FileDialog;
  
  if let Some(path) = FileDialog::new().pick_folder() {
    Ok(path.to_str().unwrap_or("").to_string())
  } else {
    Err("No directory selected".to_string())
  }
}

#[derive(Debug)]
struct TsSegment {
  url: String,
  index: usize,
}

struct M3U8Parser {
  base_url: String,
}

impl M3U8Parser {
  fn new(base_url: &str) -> Self {
    Self {
      base_url: base_url.to_string(),
    }
  }

  fn parse<'a>(&'a self, content: &'a str) -> BoxFuture<'a, Result<Vec<TsSegment>, Box<dyn std::error::Error + Send + Sync>>> {
    let mut segments = Vec::new();
    
    Box::pin(async move {
      for (index, line) in content.lines().enumerate() {
        let trimmed_line = line.trim();
        // 跳过注释行
        if trimmed_line.starts_with("#") {
          continue;
        }
        // 检查是否是 TS 片段
        if trimmed_line.ends_with(".ts") {
          let ts_url = trimmed_line.trim_matches('"');
          let full_url = self.make_full_url(ts_url);
          segments.push(TsSegment {
            url: full_url,
            index,
          });
        }
        // 处理子 M3U8 文件
        else if trimmed_line.ends_with(".m3u8") {
          let sub_m3u8_url = self.make_full_url(trimmed_line);
          let client = Client::new();
          if let Ok(response) = client.get(&sub_m3u8_url).send().await {
            if let Ok(sub_content) = response.text().await {
              let sub_parser = M3U8Parser::new(&sub_m3u8_url);
              if let Ok(sub_segments) = sub_parser.parse(&sub_content).await {
                segments.extend(sub_segments);
              }
            }
          }
        }
      }
      
      Ok(segments)
    })
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

struct Downloader {
  concurrency: usize,
  timeout_seconds: u64,
}

impl Default for Downloader {
  fn default() -> Self {
    Self {
      concurrency: 20,
      timeout_seconds: 30,
    }
  }
}

// 增加一个构造函数，允许自定义并发数
impl Downloader {
  pub fn new(concurrency: usize) -> Self {
    Self {
      concurrency,
      timeout_seconds: 30,
    }
  }
}

impl Downloader {
  async fn download_all(&self, segments: Vec<TsSegment>, output_dir: &Path, window: &tauri::Window) -> Vec<String> {
    tokio::fs::create_dir_all(output_dir).await.unwrap_or_default();

    let total = segments.len();
    let semaphore = Arc::new(Semaphore::new(self.concurrency));
    let downloaded_count = Arc::new(Mutex::new(0u32));
    let ts_files = Arc::new(Mutex::new(vec![None; total]));

    println!("Starting download of {} TS segments...", total);

    let mut tasks = Vec::new();

    for (segment_index, segment) in segments.into_iter().enumerate() {
      let semaphore = semaphore.clone();
      let downloaded_count = downloaded_count.clone();
      let ts_files = ts_files.clone();
      let output_dir = output_dir.to_path_buf();
      let window = window.clone();
      let timeout_seconds = self.timeout_seconds;
      let total = total;

      let task = task::spawn(async move {
        let _permit = semaphore.acquire().await.unwrap();

        println!("Downloading TS segment {}: {}", segment_index + 1, segment.url);

        let filename = format!("segment_{:06}.ts", segment_index);
        let output_path = output_dir.join(filename);
        let output_path_str = output_path.to_str().unwrap().to_string();
        let output_path_str_clone = output_path_str.clone();

        let client = Client::new();

        let download_task = async move {
          let response = client.get(&segment.url).send().await?;
          let content = response.bytes().await?;
          fs::write(&output_path, content)?;
          Ok::<String, Box<dyn std::error::Error + Send + Sync>>(output_path_str)
        };

        match timeout(Duration::from_secs(timeout_seconds), download_task).await {
          Ok(Ok(path)) => {
            let mut count = downloaded_count.lock().await;
            *count += 1;
            
            let progress = ((*count as f64 / total as f64) * 80.0) as u32;
            window.emit("download_progress", progress).unwrap();

            let mut files = ts_files.lock().await;
            files[segment_index] = Some(path);

            println!("Downloaded TS segment {} to {}, progress: {}% ({} of {})", 
                     segment_index + 1, output_path_str_clone, progress, *count, total);
          },
          Ok(Err(e)) => {
            println!("Failed to download TS segment {}: {}", segment_index + 1, e);
          },
          Err(_) => {
            println!("Download timeout for TS segment {}", segment_index + 1);
          }
        }
      });

      tasks.push(task);
    }

    for task in tasks {
      let _ = task.await;
    }

    let files = ts_files.lock().await;
    let mut result = Vec::new();
    let mut success_count = 0;

    for (i, file) in files.iter().enumerate() {
      if let Some(file_str) = file {
        result.push(file_str.clone());
        success_count += 1;
      } else {
        println!("TS segment {} download failed", i + 1);
      }
    }

    println!("Download completed: {} of {} segments successful", success_count, total);
    result
  }
}

struct FFmpegMerger {
  ffmpeg_path: String,
}

impl Default for FFmpegMerger {
  fn default() -> Self {
    Self {
      ffmpeg_path: "ffmpeg".to_string(),
    }
  }
}

impl FFmpegMerger {
  fn merge_ts_files(&self, ts_files: &[String], output_file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file_list_path = output_file.parent().unwrap().join("filelist.txt");
    let file_list_content = ts_files
      .iter()
      .map(|file| format!("file '{}'", file))
      .collect::<Vec<_>>()
      .join("\n");

    fs::write(&file_list_path, file_list_content)?;

    println!("Merging {} TS segments...", ts_files.len());
    
    let output = Command::new(&self.ffmpeg_path)
      .arg("-y")
      .arg("-f")
      .arg("concat")
      .arg("-safe")
      .arg("0")
      .arg("-i")
      .arg(file_list_path.to_str().unwrap())
      .arg("-c")
      .arg("copy")
      .arg(output_file.to_str().unwrap())
      .output()?;

    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      return Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("FFmpeg error: {}", stderr)
      )));
    }

    // 清理文件列表
    let _ = fs::remove_file(file_list_path);

    println!("Merged TS segments to: {}", output_file.to_str().unwrap());
    Ok(())
  }
}

#[tauri::command]
async fn download_m3u8(url: String, output_path: String, file_name: String, window: tauri::Window) -> Result<String, String> {
  println!("Downloading M3U8 from: {}", url);
  println!("Output path: {}", output_path);
  println!("File name: {}", file_name);
  
  // 确保输出目录存在
  let output_dir = Path::new(&output_path);
  if !output_dir.exists() {
    match fs::create_dir_all(output_dir) {
      Ok(_) => println!("Created output directory: {}", output_path),
      Err(e) => return Err(format!("Failed to create output directory: {}", e)),
    }
  }
  
  // 下载 M3U8 文件
  let client = Client::new();
  let m3u8_content = match client.get(&url).send().await {
    Ok(response) => match response.text().await {
      Ok(content) => content,
      Err(e) => return Err(format!("Failed to read M3U8 content: {}", e)),
    },
    Err(e) => return Err(format!("Failed to download M3U8 file: {}", e)),
  };
  
  println!("M3U8 content length: {}", m3u8_content.len());
  
  // 解析 M3U8 文件，获取 TS 片段 URL
  let parser = M3U8Parser::new(&url);
  let segments: Vec<TsSegment> = match parser.parse(&m3u8_content).await {
    Ok(segments) => segments,
    Err(e) => return Err(format!("Failed to parse M3U8 file: {}", e)),
  };
  
  println!("Found {} TS segments", segments.len());
  
  if segments.is_empty() {
    return Err("No TS segments found in M3U8 file".to_string());
  }
  
  // 发送初始进度
  window.emit("download_progress", 0).unwrap();
  
  // 下载 TS 片段
  let downloader = Downloader::new(20);
  println!("Starting download with {} concurrent threads", downloader.concurrency);
  let ts_files = downloader.download_all(segments, output_dir, &window).await;
  println!("Downloaded {} TS segments", ts_files.len());
  
  if ts_files.is_empty() {
    return Err("Failed to download any TS segments".to_string());
  }
  
  // 发送合并前的进度
  window.emit("download_progress", 90).unwrap();
  
  // 合并 TS 片段
  let output_file = output_dir.join(format!("{}.mp4", file_name));
  let merger = FFmpegMerger::default();
  match merger.merge_ts_files(&ts_files, &output_file) {
    Ok(_) => {
      // 清理临时文件
      for ts_file in ts_files {
        let _ = fs::remove_file(ts_file);
      }
      
      // 发送完成进度
      window.emit("download_progress", 100).unwrap();
      
      Ok(output_file.to_str().unwrap_or("").to_string())
    },
    Err(e) => Err(format!("Failed to merge TS segments: {}", e)),
  }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![select_directory, download_m3u8])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
