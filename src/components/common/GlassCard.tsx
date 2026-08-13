import { motion, type HTMLMotionProps } from "framer-motion";
import type { PropsWithChildren } from "react";

export function GlassCard({ children, className = "", ...props }: PropsWithChildren<HTMLMotionProps<"section">>) {
  return (
    <motion.section
      className={`glass-card ${className}`}
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.42, ease: [0.22, 1, 0.36, 1] }}
      {...props}
    >
      {children}
    </motion.section>
  );
}
