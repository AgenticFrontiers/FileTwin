import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { openPath } from "@tauri-apps/plugin-opener";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";

interface Peer {
  name: string;
  host: string;
  port: number;
}

type ConnectionStatus = "idle" | "hosting" | "browsing" | "connected";

function App() {
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>("idle");
  const [peers, setPeers] = useState<Peer[]>([]);
  const [connectedPeer, setConnectedPeer] = useState<string | null>(null);
  const [clipboardContent, setClipboardContent] = useState("");
  const [syncClipboard, setSyncClipboard] = useState(true);
  const [transferring, setTransferring] = useState(false);
  const [receivedFiles, setReceivedFiles] = useState<{ name: string; data: string }[]>([]);
  const [hostName, setHostName] = useState("");

  useEffect(() => {
    invoke<string>("get_host_name").then(setHostName).catch(() => setHostName("This Mac"));
  }, []);

  useEffect(() => {
    const unlistenPeers = listen<Peer[]>("peers", (e) => setPeers(e.payload));
    const unlistenConnected = listen<{ name: string }>("connected", (e) => {
      setConnectedPeer(e.payload.name);
      setConnectionStatus("connected");
    });
    const unlistenDisconnected = listen("disconnected", () => {
      setConnectedPeer(null);
      setConnectionStatus("idle");
    });
    const unlistenClipboard = listen<{ text: string }>("remote_clipboard", (e) => {
      if (syncClipboard && e.payload.text) {
        setClipboardContent(e.payload.text);
        writeText(e.payload.text).catch(() => {});
      }
    });
    const unlistenFile = listen<{ name: string; data: string }>("remote_file", (e) => {
      setReceivedFiles((prev) => [...prev, { name: e.payload.name, data: e.payload.data }]);
    });
    const unlistenBringToFront = listen("bring_to_front", () => {
      getCurrentWindow().setFocus().catch(() => {});
    });
    return () => {
      unlistenPeers.then((u) => u());
      unlistenConnected.then((u) => u());
      unlistenDisconnected.then((u) => u());
      unlistenClipboard.then((u) => u());
      unlistenFile.then((u) => u());
      unlistenBringToFront.then((u) => u());
    };
  }, [syncClipboard]);

  const startHosting = async () => {
    try {
      await invoke("start_host");
      setConnectionStatus("hosting");
    } catch (e) {
      console.error(e);
    }
  };

  const stopHosting = async () => {
    try {
      await invoke("stop_host");
      setConnectionStatus("idle");
    } catch (e) {
      console.error(e);
    }
  };

  const startBrowsing = async () => {
    try {
      await invoke("start_browse");
      setConnectionStatus("browsing");
    } catch (e) {
      console.error(e);
    }
  };

  const stopBrowsing = async () => {
    try {
      await invoke("stop_browse");
      setConnectionStatus("idle");
      setPeers([]);
    } catch (e) {
      console.error(e);
    }
  };

  const connectTo = async (peer: Peer) => {
    try {
      await invoke("connect_to", { host: peer.host, port: peer.port });
    } catch (e) {
      console.error(e);
    }
  };

  const disconnect = async () => {
    try {
      await invoke("disconnect");
      setConnectedPeer(null);
      setConnectionStatus("idle");
    } catch (e) {
      console.error(e);
    }
  };

  const sendClipboard = async () => {
    try {
      const text = await readText();
      if (text) await invoke("send_clipboard", { text });
    } catch (e) {
      console.error(e);
    }
  };

  const pasteFromRemote = () => {
    if (clipboardContent) writeText(clipboardContent).catch(() => {});
  };

  const requestOtherFocus = async () => {
    try {
      await invoke("send_bring_to_front");
    } catch (e) {
      console.error(e);
    }
  };

  const pickAndSendFile = async () => {
    try {
      setTransferring(true);
      await invoke("pick_and_send_file");
    } catch (e) {
      console.error(e);
    } finally {
      setTransferring(false);
    }
  };

  const saveReceivedFile = async (name: string, data: string) => {
    try {
      await invoke("save_received_file", { name, data });
    } catch (e) {
      console.error(e);
    }
  };

  const openReceivedFile = async (path: string) => {
    try {
      await openPath(path);
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div className="app">
      <header className="header">
        <h1>RemoteSync</h1>
        <p className="subtitle">WiFi &amp; Mac sharing — no IP needed</p>
        <p className="host-name">This device: {hostName}</p>
      </header>

      <section className="card connection">
        <h2>Connection</h2>
        {connectionStatus === "idle" && (
          <div className="actions">
            <button type="button" className="btn primary" onClick={startHosting}>
              Share this Mac (host)
            </button>
            <button type="button" className="btn secondary" onClick={startBrowsing}>
              Find other Macs
            </button>
          </div>
        )}
        {connectionStatus === "hosting" && (
          <div className="status">
            <span className="badge success">Sharing — others can find you</span>
            <button type="button" className="btn ghost" onClick={stopHosting}>
              Stop sharing
            </button>
          </div>
        )}
        {connectionStatus === "browsing" && (
          <div className="status">
            <span className="badge">Looking for devices…</span>
            <button type="button" className="btn ghost" onClick={stopBrowsing}>
              Stop
            </button>
            {peers.length > 0 && (
              <ul className="peer-list">
                {peers.map((p) => (
                  <li key={`${p.host}:${p.port}`}>
                    <span>{p.name}</span>
                    <button type="button" className="btn small" onClick={() => connectTo(p)}>
                      Connect
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}
        {connectionStatus === "connected" && (
          <div className="status">
            <span className="badge success">Connected to {connectedPeer}</span>
            <button type="button" className="btn ghost" onClick={disconnect}>
              Disconnect
            </button>
            <button type="button" className="btn small" onClick={requestOtherFocus} title="Bring app to front on other device">
              Open on other device
            </button>
          </div>
        )}
      </section>

      {connectionStatus === "connected" && (
        <>
          <section className="card clipboard">
            <h2>Clipboard sync</h2>
            <label className="toggle">
              <input
                type="checkbox"
                checked={syncClipboard}
                onChange={(e) => setSyncClipboard(e.target.checked)}
              />
              <span>Sync clipboard in real time</span>
            </label>
            <div className="row">
              <button type="button" className="btn primary" onClick={sendClipboard}>
                Send my clipboard
              </button>
              <button type="button" className="btn secondary" onClick={pasteFromRemote} disabled={!clipboardContent}>
                Paste from remote
              </button>
            </div>
            {clipboardContent && (
              <div className="clipboard-preview">
                <small>Remote clipboard:</small>
                <pre>{clipboardContent.slice(0, 200)}{clipboardContent.length > 200 ? "…" : ""}</pre>
              </div>
            )}
          </section>

          <section className="card files">
            <h2>Transfer files</h2>
            <button
              type="button"
              className="btn primary"
              onClick={pickAndSendFile}
              disabled={transferring}
            >
              {transferring ? "Sending…" : "Send a file"}
            </button>
            {receivedFiles.length > 0 && (
              <div className="received-files">
                <h3>Received</h3>
                <ul>
                  {receivedFiles.map((f, i) => (
                    <li key={i}>
                      <span>{f.name}</span>
                      <button type="button" className="btn small" onClick={() => saveReceivedFile(f.name, f.data)}>
                        Save
                      </button>
                      <button
                        type="button"
                        className="btn small"
                        onClick={async () => {
                          try {
                            const path = await invoke<string>("save_received_file", { name: f.name, data: f.data });
                            if (path) openReceivedFile(path);
                          } catch {
                            // User cancelled save dialog
                          }
                        }}
                      >
                        Open
                      </button>
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </section>
        </>
      )}

      <footer className="footer">
        Connect over the same WiFi or Mac sharing. No IP address needed — devices find each other by name.
      </footer>
    </div>
  );
}

export default App;
