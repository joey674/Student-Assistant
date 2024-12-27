use pyo3::prelude::*;
use pyo3_ffi::c_str;
use std::collections::HashMap;
use std::ffi::CStr;

const CODE: &CStr = c_str!(
    r#"
import whisper

class A:
    def method(self, *args):
        model = whisper.load_model("base")
        result = model.transcribe(args[0], language="de", fp16=False)
        return result["text"]
"#
);

fn convert(text: &str) -> String {
    let german_to_digits_map: HashMap<&'static str, &'static str> = HashMap::from([
        ("Null", "0"),
        ("Eins", "1"),
        ("ein", "1"),
        ("eine", "1"),
        ("Zwei", "2"),
        ("Drei", "3"),
        ("Vier", "4"),
        ("Fünf", "5"),
        ("Sechs", "6"),
        ("Sieben", "7"),
        ("Acht", "8"),
        ("Neun", "9"),
        ("Zehn", "10"),
    ]);
    let mut result = text.to_string();
    for (german, digit) in &german_to_digits_map {
        result = result.replace(german, digit);
    }

    result
        .as_str()
        .chars()
        .filter(|&c| c != ' ' && c != ',' && c != '.')
        .collect::<String>()
}

pub fn asr(path: &str) -> PyResult<String> {
    Python::with_gil(|py| {
        let module = PyModule::from_code(py, CODE, c_str!("whisper"), c_str!("whisper.py"))?;
        let class_a = module.getattr("A")?;
        let instance = class_a.call0()?;
        let args = (path,);
        let raw_result = instance.call_method1("method", args)?.extract::<String>()?;

        let converted_result = convert(&raw_result);
        Ok(converted_result)
    })
}

#[test]
fn test_whisper() {
    let asr_path = "static/download/b0e43337-0e42-4edf-be07-faa65f2fb87c/securimage_audio-27a42435f28ea7d6293f9c85e6d66f6c.wav";
    match asr(asr_path) {
        Ok(result) => println!("Final Output: {}", result),
        Err(err) => eprintln!("Error: {:?}", err),
    }
}
