import { useEffect } from "react";
import { Dashboard } from "./dashboard/Dashboard";
import { FloatingWidget } from "./floating/FloatingWidget";
import { subscribeBackendEvents } from "./services/backendEvents";
import { useAppStore } from "./stores/appStore";
import { installTheme } from "./utils/theme";

function AppLoading() {
  return <main className="app-loading"><span className="loading-orb" /><strong>My Codex</strong><small>正在读取本地数据…</small></main>;
}

export function App() {
  const { snapshot, loading, error, initialize, reload, navigate } = useAppStore();
  useEffect(() => { void initialize(); }, [initialize]);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void subscribeBackendEvents({ reload, navigate }).then((cleanup) => {
      if (disposed) cleanup();
      else unlisten = cleanup;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [navigate, reload]);

  useEffect(() => {
    if (!snapshot) return;
    return installTheme(snapshot.settings.theme);
  }, [snapshot?.settings.theme]);

  if (loading || !snapshot) return <AppLoading />;
  if (error && !snapshot) return <main className="app-error"><strong>数据初始化失败</strong><span>{error}</span><button onClick={() => void initialize()}>重试</button></main>;

  const params = new URLSearchParams(location.search);
  const requestedWindow = params.get("window");
  const isWidget = requestedWindow
    ? requestedWindow === "widget" || requestedWindow === "floating"
    : window.innerWidth <= 420;
  return isWidget ? <FloatingWidget /> : <Dashboard />;
}
