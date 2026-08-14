import { Clock, Cpu } from "@phosphor-icons/react";
import { GlassCard } from "../../components/common/GlassCard";
import type { TaskUsage } from "../../types/analytics";
import { formatDateTime, formatTokens } from "../../utils/format";

function shortId(value: string) {
  return value.length > 18 ? `${value.slice(0, 8)}…${value.slice(-6)}` : value;
}

type TaskLedgerTask = Omit<TaskUsage, "skills">;

export function TaskLedger({ tasks }: { tasks: TaskLedgerTask[] }) {
  return (
    <GlassCard className="task-ledger-card">
      <div className="panel-header">
        <div>
          <span className="panel-kicker"><Clock weight="fill" />逐任务记录</span>
          <h2>今日任务</h2>
        </div>
        <span className="task-count">{tasks.length} 条独立任务</span>
      </div>
      <p className="panel-note">每一行对应一个本地 Codex 会话；这里仅展示任务的 Token 与请求数据。</p>
      {tasks.length === 0 ? (
        <div className="task-empty"><Clock /><strong>今天还没有可识别的任务记录</strong><small>刷新本地会话后，任务记录会出现在这里。</small></div>
      ) : (
        <div className="task-ledger-list">
          {tasks.map((task, index) => (
            <article className="task-ledger-row" key={task.id}>
              <span className={`task-index task-index-${index < 3 ? index + 1 : "rest"}`}>{String(index + 1).padStart(2, "0")}</span>
              <div className="task-ledger-main">
                <div className="task-ledger-heading">
                  <strong>{task.projectName}</strong>
                  <small title={task.id}>{shortId(task.id)}</small>
                </div>
                <div className="task-ledger-meta"><span><Clock />{formatDateTime(task.startedAt)}</span><span><Cpu />{task.model}</span></div>
              </div>
              <div className="task-ledger-metrics">
                <span><small>Token</small><strong>{formatTokens(task.total)}</strong></span>
                <span><small>请求</small><strong>{task.requests}</strong></span>
              </div>
            </article>
          ))}
        </div>
      )}
    </GlassCard>
  );
}
