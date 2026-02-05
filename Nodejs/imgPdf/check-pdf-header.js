const fs = require('fs');

// 读取PDF文件的前几个字节并以十六进制格式显示
function checkPdfHeader(pdfPath) {
  try {
    if (!fs.existsSync(pdfPath)) {
      console.error('PDF文件不存在');
      return;
    }

    // 读取文件的前10个字节
    const buffer = fs.readFileSync(pdfPath, { encoding: null, start: 0, end: 10 });
    
    // 转换为十六进制字符串
    const hexString = buffer.toString('hex');
    console.log(`PDF文件头（十六进制）: ${hexString}`);
    
    // 转换为ASCII字符串
    const asciiString = buffer.toString('ascii');
    console.log(`PDF文件头（ASCII）: ${asciiString}`);
    
  } catch (error) {
    console.error('检查PDF文件头失败:', error.message);
  }
}

// 运行检查
checkPdfHeader('./out.pdf');
