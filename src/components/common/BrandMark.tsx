export function BrandMark({ size = 34 }: { size?: number }) {
  return (
    <span className="brand-mark" style={{ width: size, height: size }} aria-label="My Codex">
      <img src="/app-icon.png" alt="" aria-hidden="true" />
    </span>
  );
}
