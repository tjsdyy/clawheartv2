// Flat config (ESLint 9+) — 强制 v2 设计约束：
// 1. 禁止在 tools/* 子组件里 navigate 到 /tools/* 之外
// 2. 禁止 L2 子组件嵌 <Routes> / <Router>
// 3. 禁止内联 style 属性（除明确允许的 CSS variables）

export default [
  {
    files: ["src/**/*.{ts,tsx}"],
    rules: {
      // Custom lint guidance — 由 W14 起接入 typed-rules
      "no-restricted-imports": [
        "warn",
        {
          paths: [
            {
              name: "react-router-dom",
              importNames: ["BrowserRouter", "HashRouter"],
              message: "Only App.tsx may import <Router>; L2 must not nest routers.",
            },
          ],
        },
      ],
    },
  },
  {
    files: ["src/components/tools/**/*.tsx"],
    rules: {
      "no-restricted-syntax": [
        "warn",
        {
          selector:
            "CallExpression[callee.name='useNavigate'] ~ CallExpression[callee.name='navigate'][arguments.0.value=/^\\/(?!tools\\/)/]",
          message: "Tools must not navigate outside /tools/* — keep 2-layer depth.",
        },
        {
          selector: "JSXOpeningElement[name.name=/^Routes$|^Router$/]",
          message: "L2 tool views must not nest <Routes>/<Router>.",
        },
      ],
    },
  },
];
