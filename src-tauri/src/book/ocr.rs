use pyo3::prelude::*;
use pyo3_ffi::c_str;
use std::ffi::CStr;

const CODE: &CStr = c_str!(
    r#"
import easyocr

class A:
    def method(self, *args):
        reader = easyocr.Reader(['en'])  
        result = reader.readtext(args[0])  
        text = "[default]"
        for (bbox, text, prob) in result:
            print(f"Detected text: {text} (Confidence: {prob:.2f})")
        return text
"#
);

pub fn ocr(path: &str) -> PyResult<String> {
    Python::with_gil(|py| {
        let module = PyModule::from_code(
            py,
            CODE,
            c_str!("easyocr_module"),
            c_str!("easyocr_module.py"),
        )?;

        let class_a = module.getattr("A")?;
        let instance = class_a.call0()?;

        let args = (path,);
        let result = instance.call_method1("method", args)?.extract::<String>()?;

        // dbg!(&result);
        Ok(result)
    })
}

#[test]
fn test_easyocr() {
    let ocr_path = "static/download/b0e43337-0e42-4edf-be07-faa65f2fb87c/captcha_image.png";
    dbg!(ocr(&ocr_path));
}

#[test]
pub fn test_tesseract() {
    // let path  = std::path::Path::new("static/horizontal_text.png");
    let path = std::path::Path::new("static/verify_pic.png");

    let img = rusty_tesseract::Image::from_path(path).unwrap();
    let mut my_args = rusty_tesseract::Args {
        dpi: Some(300),
        psm: Some(6),
        oem: Some(1),
        ..rusty_tesseract::Args::default()
    };
    let output = rusty_tesseract::image_to_string(&img, &my_args).unwrap();
    println!("The String output is: {:?}", output);
}
