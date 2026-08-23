import { ArrowCounterClockwise, CheckCircle, Trash } from "@phosphor-icons/react";
import { useState } from "react";

type ResetStatus = "idle" | "success" | "error";

export function DataResetCard({ onReset }: { onReset: () => Promise<void> }) {
  const [resetting, setResetting] = useState(false);
  const [status, setStatus] = useState<ResetStatus>("idle");

  const handleReset = async () => {
    if (resetting || !window.confirm("确定要重置本地统计吗？会删除会话、项目、Skill、模型统计和已缓存的额度快照，但不会影响设置或 Codex 账户。")) return;
    setResetting(true);
    setStatus("idle");
    try {
      await onReset();
      setStatus("success");
    } catch {
      setStatus("error");
    } finally {
      setResetting(false);
    }
  };

  return (
    <div className="data-reset-card">
      <div className="data-reset-icon"><Trash weight="fill" /></div>
      <div className="data-reset-copy">
        <strong>重置本地统计</strong>
        <small>清除本机的会话、项目、Skill、模型统计和额度缓存；设置与 Codex 账户不受影响。Mock Mode 的演示数据不会被清除，此操作可以重复使用。</small>
        {status === "success" && <span className="data-reset-status is-success"><CheckCircle weight="fill" />已重置，本地统计已刷新</span>}
        {status === "error" && <span className="data-reset-status is-error">重置失败，请稍后重试</span>}
      </div>
      <button className="danger-button" type="button" onClick={() => void handleReset()} disabled={resetting}>
        <ArrowCounterClockwise className={resetting ? "is-spinning" : ""} />
        {resetting ? "重置中…" : "重置数据"}
      </button>
    </div>
  );
}
