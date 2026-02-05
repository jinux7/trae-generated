const convertImageToPdf = require('./index');
const fs = require('fs');
const path = require('path');

/**
 * 创建一个简单的测试图片（使用Base64编码的PNG图片）
 * @param {string} filePath - 保存文件路径
 */
function createTestImage(filePath) {
  return new Promise((resolve, reject) => {
    try {
      // 一个简单的1x1像素的红色PNG图片（Base64编码）
      const base64Image = 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==';
      const imageBuffer = Buffer.from(base64Image, 'base64');
      
      fs.writeFileSync(filePath, imageBuffer);
      resolve(filePath);
    } catch (error) {
      reject(error);
    }
  });
}

/**
 * 运行测试
 */
async function runTest() {
  console.log('开始测试图片转PDF功能...');
  
  try {
    // 测试图片路径
    const testImagePath = './test-image.png';
    const outputPdfPath = './test-output.pdf';
    
    console.log('1. 创建测试图片...');
    await createTestImage(testImagePath);
    console.log(`测试图片已创建: ${testImagePath}`);
    
    console.log('2. 转换图片为PDF...');
    await convertImageToPdf(testImagePath, outputPdfPath);
    console.log(`PDF文件已生成: ${outputPdfPath}`);
    
    console.log('3. 验证PDF文件...');
    if (fs.existsSync(outputPdfPath)) {
      const stats = fs.statSync(outputPdfPath);
      console.log(`PDF文件大小: ${stats.size} 字节`);
      
      // 检查文件是否为PDF格式（简单检查文件头）
      const fileBuffer = fs.readFileSync(outputPdfPath);
      // 获取前4个字节
      const headerBuffer = fileBuffer.slice(0, 4);
      const hexString = headerBuffer.toString('hex');
      const asciiString = headerBuffer.toString('ascii');
      
      // 检查前4个字节是否为PDF文件头标识（%PDF）
      if (hexString === '25504446') { 
        console.log('✓ PDF文件格式正确');
        console.log(`PDF版本: ${fileBuffer.slice(5, 10).toString('ascii')}`);
      } else {
        console.log('✗ PDF文件格式可能不正确');
        console.log(`实际文件头（十六进制）: ${hexString}`);
        console.log(`实际文件头（ASCII）: ${asciiString}`);
      }
      
      console.log('测试完成！');
    } else {
      console.log('✗ PDF文件未生成');
    }
    
  } catch (error) {
    console.error('测试失败:', error.message);
  } finally {
    // 清理测试文件
    try {
      if (fs.existsSync('./test-image.png')) {
        fs.unlinkSync('./test-image.png');
        console.log('已清理测试图片');
      }
      
      // 保留生成的PDF文件供检查
      console.log('测试PDF文件已保留在: ./test-output.pdf');
    } catch (cleanupError) {
      console.error('清理测试文件时出错:', cleanupError.message);
    }
  }
}

// 运行测试
runTest();
