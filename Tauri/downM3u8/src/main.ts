import './style.css'

document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
  <div class="container">
    <h1>M3U8下载器</h1>
    <div class="input-container">
      <input type="text" id="m3u8-url" placeholder="请输入M3U8格式文件地址" />
      <input type="text" id="video-name" placeholder="视频文件名" />
    </div>
    <div class="dir-container">
      <button id="select-dir">选择目录</button>
      <input type="text" id="dir-path" disabled placeholder="请选择下载目录" />
    </div>
    <div class="button-container">
      <button id="download">下载</button>
      <div class="progress-container" id="progress-container" style="display: none;">
        <div class="progress-bar" id="progress-bar"></div>
        <div class="progress-text" id="progress-text">0%</div>
      </div>
    </div>
  </div>
`

// 选择目录按钮点击事件
const selectDirBtn = document.querySelector<HTMLButtonElement>('#select-dir')
const dirPathInput = document.querySelector<HTMLInputElement>('#dir-path')

if (selectDirBtn && dirPathInput) {
  selectDirBtn.addEventListener('click', async () => {
    console.log('选择目录按钮被点击');
    
    try {
      // 检查 window.__TAURI__ 是否存在
      if (window && (window as any).__TAURI__) {
        console.log('TAURI 对象存在');
        const tauri = (window as any).__TAURI__;
        
        // 检查是否有 dialog 模块
        if (tauri.dialog) {
          console.log('dialog 模块存在');
          const directory = await tauri.dialog.open({
            directory: true,
            multiple: false,
            title: '选择下载目录'
          });
          
          console.log('选择的目录:', directory);
          if (directory) {
            dirPathInput.value = directory as string;
          }
        } else {
          console.error('dialog 模块不存在');
          // 尝试使用其他方法
          if (tauri.core && tauri.core.invoke) {
            console.log('使用 core.invoke');
            const directory = await tauri.core.invoke('select_directory', {});
            console.log('选择的目录:', directory);
            if (directory) {
              dirPathInput.value = directory as string;
            }
          }
        }
      } else {
        console.error('TAURI 对象不存在');
        // 显示一个提示，告诉用户 TAURI 对象不存在
        alert('TAURI 对象不存在，无法选择目录');
      }
    } catch (error) {
      console.error('选择目录时出错:', error);
      alert('选择目录时出错: ' + error);
    }
  });
}

// 下载按钮点击事件
const downloadBtn = document.querySelector<HTMLButtonElement>('#download')
const m3u8UrlInput = document.querySelector<HTMLInputElement>('#m3u8-url')
const videoNameInput = document.querySelector<HTMLInputElement>('#video-name')
const progressContainer = document.querySelector<HTMLDivElement>('#progress-container')
const progressBar = document.querySelector<HTMLDivElement>('#progress-bar')
const progressText = document.querySelector<HTMLDivElement>('#progress-text')

if (downloadBtn && m3u8UrlInput && videoNameInput && dirPathInput && progressContainer && progressBar && progressText) {
  // 添加进度事件监听器
  if (window && (window as any).__TAURI__) {
    const tauri = (window as any).__TAURI__;
    if (tauri.event) {
      tauri.event.listen('download_progress', (event: any) => {
        const progress = event.payload;
        console.log('收到进度更新:', progress);
        progressBar.style.width = `${progress}%`;
        progressText.textContent = `${progress}%`;
      });
    }
  }
  
  downloadBtn.addEventListener('click', async () => {
    console.log('下载按钮被点击');
    
    try {
      // 获取输入框中的值
      const m3u8Url = m3u8UrlInput.value.trim();
      const videoName = videoNameInput.value.trim();
      const dirPath = dirPathInput.value.trim();
      
      console.log('M3U8 URL:', m3u8Url);
      console.log('视频文件名:', videoName);
      console.log('下载目录:', dirPath);
      
      // 验证输入
      if (!m3u8Url) {
        alert('请输入M3U8格式文件地址');
        return;
      }
      
      if (!videoName) {
        alert('请输入视频文件名');
        return;
      }
      
      if (!dirPath) {
        alert('请选择下载目录');
        return;
      }
      
      // 显示进度条
      progressContainer.style.display = 'block';
      progressBar.style.width = '0%';
      progressText.textContent = '0%';
      
      // 检查 window.__TAURI__ 是否存在
      if (window && (window as any).__TAURI__) {
        console.log('TAURI 对象存在');
        const tauri = (window as any).__TAURI__;
        
        // 调用后端的下载命令
        if (tauri.core && tauri.core.invoke) {
          console.log('调用 download_m3u8 命令');
          const result = await tauri.core.invoke('download_m3u8', {
            url: m3u8Url,
            outputPath: dirPath,
            fileName: videoName
          });
          
          console.log('下载结果:', result);
          
          // 确保进度条更新到100%
          progressBar.style.width = '100%';
          progressText.textContent = '100%';
          
          // 清空输入框
          m3u8UrlInput.value = '';
          videoNameInput.value = '';
          
          // 显示完成消息
          alert('下载完成: ' + result);
          
          // 3秒后隐藏进度条
          setTimeout(() => {
            progressContainer.style.display = 'none';
          }, 3000);
        } else {
          console.error('core.invoke 不存在');
          alert('无法调用后端命令，core.invoke 不存在');
          // 隐藏进度条
          progressContainer.style.display = 'none';
        }
      } else {
        console.error('TAURI 对象不存在');
        alert('TAURI 对象不存在，无法下载');
        // 隐藏进度条
        progressContainer.style.display = 'none';
      }
    } catch (error) {
      console.error('下载时出错:', error);
      alert('下载时出错: ' + error);
      // 隐藏进度条
      progressContainer.style.display = 'none';
    }
  });
}
