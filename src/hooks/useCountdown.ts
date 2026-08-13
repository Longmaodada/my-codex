import { useEffect, useState } from "react";
import { formatCountdown } from "../utils/format";

export function useCountdown(resetAt: string | null) {
  const [text, setText] = useState(() => formatCountdown(resetAt));
  useEffect(() => {
    setText(formatCountdown(resetAt));
    const interval = window.setInterval(() => setText(formatCountdown(resetAt)), 1000);
    return () => window.clearInterval(interval);
  }, [resetAt]);
  return text;
}
