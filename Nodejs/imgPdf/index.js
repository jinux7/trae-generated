const PDFDocument = require('pdfkit');
const fs = require('fs');
const path = require('path');

/**
 * 将图片转换为PDF文件
 * @param {string} imagePath - 图片文件路径或文件夹路径
 * @param {string} outputPdfPath - 输出PDF文件路径
 * @param {Object} options - 可选配置
 * @param {number} options.width - PDF页面宽度
 * @param {number} options.height - PDF页面高度
 */
function convertImageToPdf(imagePath, outputPdfPath, options = {}) {
  return new Promise((resolve, reject) => {
    try {
      // 检查路径是否存在
      if (!fs.existsSync(imagePath)) {
        throw new Error('路径不存在');
      }

      // 检查是文件还是文件夹
      const stats = fs.statSync(imagePath);
      let imageFiles = [];

      if (stats.isDirectory()) {
        // 如果是文件夹，读取所有支持的图片文件
        const files = fs.readdirSync(imagePath);
        const supportedFormats = ['.jpg', '.jpeg', '.png', '.gif'];

        // 过滤出支持的图片文件
        imageFiles = files
          .filter(file => {
            const ext = path.extname(file).toLowerCase();
            return supportedFormats.includes(ext);
          })
          .sort() // 按文件名升序排列
          .map(file => path.join(imagePath, file)); // 构建完整路径

        if (imageFiles.length === 0) {
          throw new Error('文件夹中没有支持的图片文件');
        }
      } else {
        // 如果是单个文件，检查是否是支持的图片格式
        const imageExt = path.extname(imagePath).toLowerCase();
        const supportedFormats = ['.jpg', '.jpeg', '.png', '.gif'];
        
        if (!supportedFormats.includes(imageExt)) {
          throw new Error('不支持的图片格式，仅支持: ' + supportedFormats.join(', '));
        }

        imageFiles = [imagePath];
      }

      // 创建PDF文档
      const doc = new PDFDocument({
        size: [options.width || 612, options.height || 792], // 默认A4尺寸
        margin: 0
      });

      // 创建输出流
      const writeStream = fs.createWriteStream(outputPdfPath);
      doc.pipe(writeStream);

      // 计算PDF页面尺寸
      const pageWidth = options.width || 612;
      const pageHeight = options.height || 792;

      // 依次添加图片到PDF
      imageFiles.forEach((imgPath, index) => {
        // 除了第一张图片外，其他图片都需要添加新页面
        if (index > 0) {
          doc.addPage({
            size: [pageWidth, pageHeight],
            margin: 0
          });
        }

        // 添加图片到PDF
        doc.image(imgPath, 0, 0, {
          width: pageWidth,
          height: pageHeight,
          fit: [pageWidth, pageHeight],
          align: 'center',
          valign: 'center'
        });
      });

      // 结束PDF文档
      doc.end();

      // 监听写入完成事件
      writeStream.on('finish', () => {
        resolve(outputPdfPath);
      });

      // 监听错误事件
      writeStream.on('error', (error) => {
        reject(error);
      });

    } catch (error) {
      reject(error);
    }
  });
}

// 导出函数
module.exports = convertImageToPdf;

// 如果直接运行此文件，则执行示例
if (require.main === module) {
  const readline = require('readline');
  
  // 创建readline接口
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
  });
  
  // 提示用户输入图片路径或文件夹路径
  rl.question('请输入图片路径或文件夹路径: ', (imagePath) => {
    if (!imagePath) {
      console.error('路径不能为空');
      rl.close();
      process.exit(1);
    }
    
    // 提示用户输入PDF文件名（不需要输入后缀）
    rl.question('请输入PDF文件名（不需要输入后缀，不输入默认out）: ', (pdfFileName) => {
      // 关闭readline接口
      rl.close();
      
      // 处理PDF文件名
      const fileName = pdfFileName || 'out';
      const outputPdfPath = `./${fileName}.pdf`;
      
      console.log('开始转换图片到PDF...');
      console.log(`输入路径: ${imagePath}`);
      console.log(`输出PDF: ${outputPdfPath}`);
      
      convertImageToPdf(imagePath, outputPdfPath)
        .then((pdfPath) => {
          console.log(`PDF文件已成功生成: ${pdfPath}`);
        })
        .catch((error) => {
          console.error('转换失败:', error.message);
          process.exit(1);
        });
    });
  });
}
