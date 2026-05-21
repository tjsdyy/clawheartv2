import { Toaster as Sonner } from "sonner";

/**
 * 全局 toast 通知。
 * 用法：`import { toast } from "sonner"; toast.success("xxx")`
 */
export function Toaster() {
  return (
    <Sonner
      position="top-right"
      theme="system"
      richColors
      closeButton
      duration={4000}
      toastOptions={{
        className: "!font-sans",
        style: {
          fontFamily: "inherit",
          background: "rgb(var(--bg-elev))",
          color: "rgb(var(--text))",
          border: "1px solid rgb(var(--border))",
        },
      }}
    />
  );
}
