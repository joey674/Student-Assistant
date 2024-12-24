import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { attachConsole } from "@tauri-apps/plugin-log";
import "./App.css";

function App() {
  const [statusMsg, setStatusMsg] = useState("");
  const [email, setEmail] = useState("");
  const [appointmentType, setAppointmentType] = useState("");
  const [logs, setLogs] = useState([]);
  const logRef = useRef(null);

  async function activate() {
    console.log(email);
    console.log(appointmentType);
    if (!email || !appointmentType) {
      setStatusMsg("Please fill in all fields.");
      return;
    }
    let appointmentTypename = parseInt(appointmentType);
    await invoke("activate", {
      email,
      appointmentTypename,
    });
  }

  async function initLogger() {
    const detach = await attachConsole();
  }
  initLogger();

  useEffect(() => {
    // 保存原始 console 方法
    const originalConsole = { ...console };

    // 重写需要捕获的标准方法
    const methodsToOverride = ["log", "info", "warn", "error", "debug"];
    methodsToOverride.forEach((method) => {
      console[method] = (...args) => {
        // 调用原始方法
        originalConsole[method](...args);

        // 格式化日志内容
        const formattedMessage = args
          .map((arg) => {
            // 尽量保留原始格式，如果是对象则尝试 JSON.stringify
            if (typeof arg === "string") {
              return arg;
            } else if (typeof arg === "object") {
              try {
                return JSON.stringify(arg, null, 2); // 美化格式输出
              } catch (e) {
                return "[Circular]"; // 处理循环引用问题
              }
            }
            return String(arg);
          })
          .join(" ");

        // 添加日志类型前缀
        const logMessage = `[${method.toUpperCase()}] ${formattedMessage}`;

        // 更新日志状态
        setLogs((prevLogs) => [...prevLogs, logMessage]);
      };
    });

    // 绑定后端日志到前端控制台
    attachConsole();

    // 恢复原始 console 方法
    return () => {
      methodsToOverride.forEach((method) => {
        console[method] = originalConsole[method];
      });
    };
  }, []); // 仅在组件挂载时运行一次

  useEffect(() => {
    if (logRef.current) {
      logRef.current.scrollTop = logRef.current.scrollHeight;
    }
  }, [logs]);

  return (
    <main className="container">
      <h2>Book appointment in RWTH</h2>

      <h4>
        Please pay attention to the mailbox. When there are available
        appointments, you will get an email instantly!
      </h4>

      <form
        className="form-container"
        onSubmit={(e) => {
          e.preventDefault();
          activate();
        }}
      >
        {/* Email Input */}
        <div className="form-group">
          <label htmlFor="email-input">Email:</label>
          <input
            id="email-input"
            type="email"
            onChange={(e) => setEmail(e.currentTarget.value)}
            placeholder="Enter your email..."
            required
          />
        </div>

        {/* Appointment Type Selection */}
        <div className="form-group">
          <label htmlFor="appointment-select">Appointment Type:</label>
          <select
            id="appointment-select"
            onChange={(e) => setAppointmentType(e.currentTarget.value)}
            defaultValue=""
            required
          >
            <option value="" disabled>
              Select an appointment type
            </option>
            <option value="0">Book Visa Pickup</option>
            <option value="1">Book Visa extension</option>
          </select>
        </div>

        {/* Submit Button */}
        <div>
          <button type="submit">Activate</button>
        </div>
      </form>

      <p>{statusMsg}</p>

      {/* logs */}
      <div className="log-container">
        <h4>Logs:</h4>
        <textarea readOnly ref={logRef} value={logs.join("\n")} />
      </div>
    </main>
  );
}

export default App;
