import { Bell, Eye, GearSix, Info, LockKey, RocketLaunch, SlidersHorizontal, WifiHigh } from "@phosphor-icons/react";
import { useState } from "react";
import { GlassCard } from "../../components/common/GlassCard";
import { Toggle } from "../../components/common/Toggle";
import type { AppSettings } from "../../types/analytics";

const sections = [
  ["常规", GearSix], ["显示", Eye], ["刷新", WifiHigh], ["数据", SlidersHorizontal],
  ["通知", Bell], ["隐私", LockKey], ["高级", RocketLaunch], ["关于", Info],
] as const;

export function SettingsPage({ settings, onChange }: { settings: AppSettings; onChange: (patch: Partial<AppSettings>) => void }) {
  const [section, setSection] = useState("常规");
  return (
    <div className="settings-layout">
      <GlassCard className="settings-nav">
        {sections.map(([label, Icon]) => <button key={label} className={section === label ? "active" : ""} onClick={() => setSection(label)}><Icon weight={section === label ? "fill" : "regular"} />{label}</button>)}
      </GlassCard>
      <GlassCard className="settings-panel">
        <div className="panel-header"><div><span className="panel-kicker">My Codex 设置</span><h2>{section}</h2></div></div>
        {section === "常规" && <div className="settings-group">
          <Toggle label="始终置顶" description="让桌面悬浮窗显示在其他窗口上方" checked={settings.alwaysOnTop} onChange={(value) => onChange({ alwaysOnTop: value })} />
          <Toggle label="启用胶囊模式" description="开启后悬浮窗默认收起，鼠标移入胶囊展开，移出后自动收起" checked={settings.capsuleMode} onChange={(value) => onChange({ capsuleMode: value })} />
          <Toggle label="开机启动" checked={settings.launchAtStartup} onChange={(value) => onChange({ launchAtStartup: value })} />
          <Toggle label="启动时显示悬浮窗" checked={settings.showWidgetOnLaunch} onChange={(value) => onChange({ showWidgetOnLaunch: value })} />
          <Toggle label="关闭主面板时隐藏到托盘" checked={settings.closeToTray} onChange={(value) => onChange({ closeToTray: value })} />
          <Toggle label="锁定悬浮窗位置" checked={settings.lockPosition} onChange={(value) => onChange({ lockPosition: value })} />
        </div>}
        {section === "显示" && <div className="settings-group">
          <div className="select-row"><span><strong>主题</strong></span><select value={settings.theme} onChange={(event) => onChange({ theme: event.target.value as AppSettings["theme"] })}><option value="system">跟随系统</option><option value="light">浅色</option><option value="dark">深色</option></select></div>
          <div className="select-row"><span><strong>语言</strong></span><select value={settings.language} onChange={(event) => onChange({ language: event.target.value as AppSettings["language"] })}><option value="zh-CN">中文</option><option value="en">English</option></select></div>
        </div>}
        {section === "刷新" && <div className="settings-group"><div className="select-row"><span><strong>刷新模式</strong><small>智能模式会根据活动和重置时间调整频率</small></span><select value={settings.refreshMode} onChange={(event) => onChange({ refreshMode: event.target.value as AppSettings["refreshMode"] })}><option value="smart">智能</option><option value="10s">10 秒</option><option value="30s">30 秒</option><option value="1m">1 分钟</option><option value="5m">5 分钟</option></select></div></div>}
        {section === "数据" && <div className="settings-group"><Toggle label="Mock Mode" description="使用演示数据验证界面，不写入真实统计库" checked={settings.mockMode} onChange={(value) => onChange({ mockMode: value })} /></div>}
        {section === "通知" && <div className="settings-group"><div className="thresholds"><strong>额度剩余阈值</strong><span>{[50, 25, 20, 10, 5].map((value) => <button className={settings.notificationThresholds.includes(value) ? "active" : ""} key={value} onClick={() => onChange({ notificationThresholds: settings.notificationThresholds.includes(value) ? settings.notificationThresholds.filter((item) => item !== value) : [...settings.notificationThresholds, value].sort((a, b) => b - a) })}>{value}%</button>)}</span></div></div>}
        {section === "隐私" && <div className="privacy-card"><LockKey weight="fill" /><h3>Local First</h3><p>数据仅在本机处理，不上传 Access Token、Prompt、聊天原文或项目内容。</p></div>}
        {section === "高级" && <div className="settings-group"><div className="setting-note">快捷键：Ctrl + Shift + Q 显示/隐藏悬浮窗；Ctrl + Shift + D 打开主面板。</div></div>}
        {section === "关于" && <div className="privacy-card"><Info weight="fill" /><h3>My Codex 0.1.0</h3><p>本项目是独立的本地分析工具，与 OpenAI 没有官方隶属关系。</p></div>}
      </GlassCard>
    </div>
  );
}
