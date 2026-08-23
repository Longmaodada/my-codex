import { ArrowsClockwise, CaretDown, GearSix, Minus, Moon, SidebarSimple, Square, Sun, X } from "@phosphor-icons/react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { AnimatePresence, motion } from "framer-motion";
import { useEffect, useState } from "react";
import { BrandMark } from "../components/common/BrandMark";
import { SourceBadge } from "../components/common/SourceBadge";
import { windowAction } from "../services/backend";
import { isTauri } from "../services/backend";
import type { NavigationTarget } from "../services/backendEvents";
import { useAppStore } from "../stores/appStore";
import type { ProjectUsage } from "../types/analytics";
import { providerStatusLabel } from "../utils/providerStatus";
import { resolveTheme } from "../utils/theme";
import { SummaryCards } from "./components/SummaryCards";
import { TaskLedger } from "./components/TaskLedger";
import { CachePage } from "./pages/CachePage";
import { ModelsPage } from "./pages/ModelsPage";
import { OverviewPage } from "./pages/OverviewPage";
import { ProjectsPage } from "./pages/ProjectsPage";
import { SettingsPage } from "./pages/SettingsPage";
import { SkillsPage } from "./pages/SkillsPage";

type Page = NavigationTarget;

const tabs: Array<[Page, string]> = [["overview", "今日任务"], ["month", "月度趋势"], ["week", "逐任务记录"], ["skills", "Skill"], ["projects", "项目"], ["models", "模型"], ["cache", "缓存"]];

export function Dashboard() {
  const { snapshot, refreshing, refresh, clearLocalData, lastRefreshedAt, updateSettings, navigationTarget } = useAppStore();
  const [page, setPage] = useState<Page>(() => new URLSearchParams(location.search).get("page") === "settings" ? "settings" : "overview");
  const [selectedProject, setSelectedProject] = useState<ProjectUsage | null>(null);
  const [sidebarOpen, setSidebarOpen] = useState(false);

  useEffect(() => {
    if (!navigationTarget) return;
    setPage(navigationTarget);
    setSidebarOpen(false);
    if (navigationTarget !== "projects") setSelectedProject(null);
  }, [navigationTarget]);

  if (!snapshot) return null;
  const source = snapshot.settings.mockMode ? "mock" : snapshot.quota.source;

  const openProject = (project: ProjectUsage) => {
    setSelectedProject(project);
    setPage("projects");
  };

  return (
    <motion.main className="dashboard-shell" initial={{ opacity: 0, scale: 0.975 }} animate={{ opacity: 1, scale: 1 }} transition={{ duration: 0.48, ease: [0.22, 1, 0.36, 1] }}>
      <div className="ambient ambient-one" /><div className="ambient ambient-two" /><div className="ambient ambient-three" />
      <div className="top-light" />
      <header className="dashboard-header" data-tauri-drag-region onPointerDown={(event) => {
        if (event.button !== 0 || !isTauri()) return;
        if ((event.target as HTMLElement).closest("button, a, input, select")) return;
        void getCurrentWindow().startDragging();
      }}>
        <button className="sidebar-toggle" onClick={() => setSidebarOpen((value) => !value)}><SidebarSimple /></button>
        <div className="brand-lockup" data-tauri-drag-region><BrandMark size={38} /><span><strong>My Codex</strong><small>{snapshot.quota.plan ?? "未连接"}</small></span></div>
        <div className="dashboard-center" data-tauri-drag-region><strong>Codex</strong><SourceBadge source={source} compact /></div>
        <div className="dashboard-actions">
          <button className="provider-pill" title={snapshot.quota.message}><span className={`status-dot status-${snapshot.quota.status}`} />{snapshot.quota.provider === "codex-app-server" ? "已连接" : snapshot.settings.mockMode ? "Mock Mode" : "数据不可用"}<CaretDown /></button>
          <button className={`icon-button ${refreshing ? "is-spinning" : ""}`} onClick={() => void refresh()} title="立即刷新"><ArrowsClockwise /></button>
          <button className="icon-button" onClick={() => void updateSettings({ theme: snapshot.settings.theme === "dark" ? "light" : "dark" })} title="切换主题">{resolveTheme(snapshot.settings.theme) === "dark" ? <Sun /> : <Moon />}</button>
          <button className={`icon-button ${page === "settings" ? "is-active" : ""}`} onClick={() => setPage("settings")} title="设置"><GearSix /></button>
          <span className="window-divider" />
          <button className="window-button" onClick={() => void windowAction("minimize")}><Minus /></button>
          <button className="window-button" onClick={() => void windowAction("maximize")}><Square /></button>
          <button className="window-button close-button" onClick={() => void windowAction("close")}><X /></button>
        </div>
      </header>

      <section className="dashboard-body">
        {snapshot.settings.mockMode && <div className="dashboard-mock-banner"><SourceBadge source="mock" /><strong>正在使用展示数据</strong><span>切换到真实模式后，只显示可靠取得的官方额度与本地统计；不可用字段显示“—”。</span><button onClick={() => void updateSettings({ mockMode: false })}>切换真实模式</button></div>}
        {page !== "settings" && <SummaryCards snapshot={snapshot} />}
        <nav className={`dashboard-tabs ${sidebarOpen ? "is-open" : ""}`}>
          {tabs.map(([id, label]) => <button key={id} className={page === id ? "active" : ""} onClick={() => { setPage(id); setSidebarOpen(false); if (id !== "projects") setSelectedProject(null); }}>{label}{page === id && <motion.span layoutId="tab-indicator" />}</button>)}
          <span className="tab-updated">最后更新：{lastRefreshedAt?.toLocaleTimeString("zh-CN", { hour12: false }) ?? "—"}</span>
        </nav>
        <AnimatePresence mode="wait">
          <motion.section className="page-content" key={page + (selectedProject?.id ?? "")} initial={{ opacity: 0, y: 7 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -5 }} transition={{ duration: 0.22 }}>
            {page === "overview" && <OverviewPage snapshot={snapshot} onProject={openProject} />}
            {page === "month" && <OverviewPage snapshot={snapshot} onProject={openProject} defaultRange="30D" />}
            {page === "week" && <TaskLedger tasks={snapshot.usage.tasks} />}
            {page === "projects" && <ProjectsPage snapshot={snapshot} selected={selectedProject} onSelect={setSelectedProject} onBack={() => setSelectedProject(null)} />}
            {page === "skills" && <SkillsPage skills={snapshot.usage.skills} />}
            {page === "models" && <ModelsPage models={snapshot.usage.models} />}
            {page === "cache" && <CachePage snapshot={snapshot} />}
            {page === "settings" && <SettingsPage settings={snapshot.settings} onChange={(patch) => void updateSettings(patch)} onResetData={clearLocalData} />}
          </motion.section>
        </AnimatePresence>
      </section>
    </motion.main>
  );
}
