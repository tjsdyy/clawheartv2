/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        sans: [
          "Inter",
          "-apple-system",
          "BlinkMacSystemFont",
          "PingFang SC",
          "Microsoft YaHei",
          "sans-serif",
        ],
        mono: [
          "JetBrains Mono",
          "SF Mono",
          "Menlo",
          "monospace",
        ],
      },
      colors: {
        // Semantic tokens; values come from CSS variables in globals.css
        bg: {
          DEFAULT: "rgb(var(--bg-base) / <alpha-value>)",
          elev: "rgb(var(--bg-elev) / <alpha-value>)",
          elev2: "rgb(var(--bg-elev2) / <alpha-value>)",
        },
        text: {
          DEFAULT: "rgb(var(--text) / <alpha-value>)",
          dim: "rgb(var(--text-dim) / <alpha-value>)",
          muted: "rgb(var(--text-muted) / <alpha-value>)",
        },
        border: "rgb(var(--border) / <alpha-value>)",
        "border-soft": "rgb(var(--border-soft) / <alpha-value>)",
        accent: "rgb(var(--accent) / <alpha-value>)",
        "accent-dim": "rgb(var(--accent-dim) / <alpha-value>)",
        critical: "rgb(var(--critical) / <alpha-value>)",
        high: "rgb(var(--high) / <alpha-value>)",
        medium: "rgb(var(--medium) / <alpha-value>)",
        low: "rgb(var(--low) / <alpha-value>)",
      },
      keyframes: {
        pulse: {
          "0%, 100%": { opacity: "1", transform: "translateY(-50%) scale(1)" },
          "50%": { opacity: "0.4", transform: "translateY(-50%) scale(0.8)" },
        },
        livepulse: {
          "0%, 100%": { opacity: "1" },
          "50%": { opacity: "0.55" },
        },
        fadein: {
          from: { opacity: "0", transform: "translateY(4px)" },
          to: { opacity: "1", transform: "translateY(0)" },
        },
        "slidein-right": {
          from: { transform: "translateX(100%)" },
          to: { transform: "translateX(0)" },
        },
      },
      animation: {
        pulse: "pulse 1.4s infinite",
        livepulse: "livepulse 2s infinite",
        fadein: "fadein 120ms ease-out",
        "slidein-right": "slidein-right 180ms ease-out",
      },
    },
  },
  plugins: [],
};
