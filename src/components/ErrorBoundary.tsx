import { Component, ReactNode } from "react";
import { AlertOctagon, RefreshCw } from "lucide-react";

interface Props {
  children: ReactNode;
  fallback?: (error: Error, reset: () => void) => ReactNode;
}

interface State {
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: { componentStack?: string | null }) {
    // W14 起接入 sentry / opentelemetry；现在仅 console
    console.error("[ClawHeart] UI crash:", error, info);
  }

  reset = () => this.setState({ error: null });

  render() {
    const { error } = this.state;
    if (!error) return this.props.children;

    if (this.props.fallback) return this.props.fallback(error, this.reset);

    return (
      <div className="flex flex-col items-center justify-center h-full px-12 py-20 text-center">
        <div
          className="w-16 h-16 rounded-2xl flex items-center justify-center mb-6"
          style={{
            background: "rgb(var(--critical) / 0.12)",
            color: "rgb(var(--critical))",
          }}
        >
          <AlertOctagon className="w-8 h-8" />
        </div>
        <h2 className="text-xl font-semibold mb-2">出错了</h2>
        <p className="text-text-dim text-sm max-w-md mb-6 leading-relaxed font-mono">
          {error.name}: {error.message}
        </p>
        <button onClick={this.reset} className="btn-primary">
          <RefreshCw className="w-4 h-4" />
          重试
        </button>
        <details className="mt-8 max-w-2xl text-left">
          <summary className="cursor-pointer text-text-muted text-[12px] font-mono">
            技术详情（脱敏后可附在反馈中）
          </summary>
          <pre className="mt-3 p-4 bg-bg-elev2 rounded-lg overflow-auto text-[11px] font-mono text-text-dim">
            {error.stack ?? error.message}
          </pre>
        </details>
      </div>
    );
  }
}
