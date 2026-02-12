import { useEffect, useRef, useState } from "react";
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
  const [screenshotting, setScreenshotting] = useState(false);
  const [receivedFiles, setReceivedFiles] = useState<{ name: string; data: string }[]>([]);
  const [hostName, setHostName] = useState("");
  const [connecting, setConnecting] = useState(false);
  const [connectingToPeer, setConnectingToPeer] = useState<string | null>(null);
  const [showConnectionSuccess, setShowConnectionSuccess] = useState(false);
  const [connectionError, setConnectionError] = useState<string | null>(null);
  const connectionSuccessTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    invoke<string>("get_host_name").then(setHostName).catch(() => setHostName("This Mac"));
  }, []);

  useEffect(() => {
    const unlistenPeers = listen<Peer[]>("peers", (e) => setPeers(e.payload));
    const unlistenConnected = listen<{ name: string }>("connected", (e) => {
      if (connectionSuccessTimeoutRef.current) clearTimeout(connectionSuccessTimeoutRef.current);
      setConnectedPeer(e.payload.name);
      setConnectionStatus("connected");
      setConnecting(false);
      setConnectingToPeer(null);
      setShowConnectionSuccess(true);
      connectionSuccessTimeoutRef.current = setTimeout(() => {
        setShowConnectionSuccess(false);
        connectionSuccessTimeoutRef.current = null;
      }, 4000);
    });
    const unlistenDisconnected = listen("disconnected", () => {
      setConnectedPeer(null);
      setConnectionStatus("idle");
      setConnecting(false);
      setConnectingToPeer(null);
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
      setConnectionError(null);
    } catch (e) {
      console.error(e);
    }
  };

  const connectTo = async (peer: Peer) => {
    try {
      setConnectionError(null);
      setConnecting(true);
      setConnectingToPeer(peer.name);
      await invoke("connect_to", { host: peer.host, port: peer.port });
    } catch (e) {
      console.error(e);
      setConnectionError(e instanceof Error ? e.message : String(e));
      setConnecting(false);
      setConnectingToPeer(null);
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

  const captureScreenshotAndSend = async () => {
    try {
      setScreenshotting(true);
      await invoke("capture_screenshot_and_send");
    } catch (e) {
      console.error(e);
    } finally {
      setScreenshotting(false);
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
          <div className="status connection-browsing-wrap">
            <div className="status">
              {connecting ? (
                <>
                  <span className="spinner" aria-hidden />
                  <span className="badge">Connecting to {connectingToPeer}…</span>
                </>
              ) : (
                <>
                  <span className="badge">Looking for devices…</span>
                  <button type="button" className="btn ghost" onClick={stopBrowsing}>
                    Stop
                  </button>
                </>
              )}
            </div>
            {connecting && (
              <p className="connection-hint">Up to 3 attempts (15 sec each). Ensure both Macs are on the same network.</p>
            )}
            {connectionError && (
              <p className="connection-error">{connectionError}</p>
            )}
            {peers.length > 0 && !connecting && (
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
          <div className="status connection-success-wrap">
            <div className="status">
              <span className="badge success">Connected to {connectedPeer}</span>
              <button type="button" className="btn ghost" onClick={disconnect}>
                Disconnect
              </button>
              <button type="button" className="btn small" onClick={requestOtherFocus} title="Bring app to front on other device">
                Open on other device
              </button>
            </div>
            {showConnectionSuccess && (
              <p className="connection-success-msg">Connection successful!</p>
            )}
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
            <div className="row">
              <button
                type="button"
                className="btn primary"
                onClick={pickAndSendFile}
                disabled={transferring}
              >
                {transferring ? "Sending…" : "Send a file"}
              </button>
              <button
                type="button"
                className="btn primary"
                onClick={captureScreenshotAndSend}
                disabled={screenshotting || transferring}
                title="Capture screen (select region), save as JPG, and send to remote"
              >
                {screenshotting ? "Capturing…" : "Capture screenshot"}
              </button>
            </div>
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
