import easyocr

# 初始化 OCR 读取器
reader = easyocr.Reader(['en'])  

# 读取图片并进行 OCR
result = reader.readtext('static/verify_pic.png')

# 打印识别结果
for (bbox, text, prob) in result:
    print(f"Detected text: {text} (Confidence: {prob:.2f})")
