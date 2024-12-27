import whisper

model = whisper.load_model("base")
result = model.transcribe("E:\\repo\\StudentAssistant\\src-tauri\\static\\download\\b0e43337-0e42-4edf-be07-faa65f2fb87c\\securimage_audio-27a42435f28ea7d6293f9c85e6d66f6c.wav")
print(result["text"])