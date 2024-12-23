import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [activateMsg, setActivateMsg] = useState("");
  const [email, setEmail] = useState("");

  async function activate() {
    try {
      const response = await invoke("activate", { email });
      setActivateMsg(response);
    } catch (err) {
      console.error("fail:", err);
      setActivateMsg(err);
    }
  }

  return (
    <main className="container">
      <h1>Book appointment in RWTH</h1>
      <h2>
        Please pay attention to the mailbox, when there are availiable
        appointments, you will get email instantly!!!
      </h2>
      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          activate();
        }}
      >
        <input
          id="activate-input"
          onChange={(e) => setEmail(e.currentTarget.value)}
          placeholder="Enter your email..."
        />
        <button type="submit">Activate</button>
      </form>
      <p>{activateMsg}</p>
    </main>
  );
}

export default App;
