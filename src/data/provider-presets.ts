/**
 * 模型供应商预设清单 —— 用于「新增渠道」一键填表。
 *
 * 来源：cc-switch (claudeProviderPresets/codexProviderPresets/geminiProviderPresets)
 * + 9router supported providers 合并去重。共 ~70 条。
 *
 * 每条预设包含品牌色 (color) 用于色块图标。
 */
import type { ProtocolKind, ProviderKind } from "@/hooks/useProviders";

export type PresetCategory =
  | "official"        // 官方
  | "cn_official"     // 国内大厂官方
  | "cloud_provider"  // 云服务（AWS/Azure/GCP）
  | "aggregator"      // 聚合路由
  | "third_party"     // 第三方/合作伙伴
  | "self_hosted"     // 自托管
  | "custom";         // 自定义

export type AuthMethod = "api_key" | "oauth";

export interface ProviderPreset {
  id: string;
  name: string;
  base_url: string;
  protocol: ProtocolKind;
  provider_kind: ProviderKind;
  category: PresetCategory;
  default_model?: string;
  /** 品牌色（hex），用于色块图标背景 */
  color: string;
  /** 简短描述/分组用 */
  note?: string;
  /** 适用的 Agent 平台 hint。空表示通用 */
  recommended_for?: string[];
  /** 认证方式：api_key（默认）或 oauth */
  auth_method?: AuthMethod;
  /** simple-icons 风格的 SVG path data，可选；不提供则 fallback 到字母色块 */
  icon_svg_path?: string;
}

// 品牌色调色板
const COLORS = {
  anthropic: "#C97759",
  openai: "#10A37F",
  gemini: "#4285F4",
  google: "#4285F4",
  microsoft: "#0078D4",
  aws: "#FF9900",
  zhipu: "#5B5FE9",
  bytedance: "#D81E06",
  alibaba: "#FF6A00",
  baidu: "#2932E1",
  moonshot: "#6C5CE7",
  deepseek: "#4D6BFE",
  minimax: "#FF6B8A",
  stepfun: "#A78BFA",
  modelscope: "#0F62FE",
  siliconflow: "#7C3AED",
  novita: "#22C55E",
  openrouter: "#7C3AED",
  packycode: "#FB923C",
  github: "#24292F",
  nvidia: "#76B900",
  xiaomi: "#FF6900",
  longcat: "#F59E0B",
  alipay: "#1677FF",
  vanchin: "#E11D48",
  custom: "#6B7280",
  // 三方合作伙伴渐变（按字母）
  partner1: "#0EA5E9",
  partner2: "#14B8A6",
  partner3: "#F97316",
  partner4: "#A855F7",
  partner5: "#EAB308",
  partner6: "#EC4899",
};

const RAW_PRESETS: ProviderPreset[] = [
  // ──────────── 官方 ────────────
  {
    id: "anthropic_official",
    name: "Claude Official",
    base_url: "https://api.anthropic.com",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "official",
    color: COLORS.anthropic,
    note: "Anthropic 官方",
    recommended_for: ["claude"],
  },
  {
    id: "openai_official",
    name: "OpenAI Official",
    base_url: "https://api.openai.com/v1",
    protocol: "openai",
    provider_kind: "openai",
    category: "official",
    color: COLORS.openai,
    note: "OpenAI 官方",
    recommended_for: ["codex"],
  },
  {
    id: "openai_codex",
    name: "OpenAI Codex",
    base_url: "https://chatgpt.com/backend-api/codex",
    protocol: "openai_responses",
    provider_kind: "openai",
    category: "official",
    color: COLORS.openai,
    note: "Codex 官方（需 ChatGPT 订阅）",
    recommended_for: ["codex"],
  },
  {
    id: "gemini_official",
    name: "Google Gemini",
    base_url: "https://generativelanguage.googleapis.com",
    protocol: "gemini",
    provider_kind: "custom",
    category: "official",
    color: COLORS.gemini,
    note: "Google AI Studio",
    recommended_for: ["gemini"],
  },

  // ──────────── 国内大厂 ────────────
  {
    id: "deepseek",
    name: "DeepSeek",
    base_url: "https://api.deepseek.com",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "cn_official",
    color: COLORS.deepseek,
    note: "深度求索",
  },
  {
    id: "zhipu_glm",
    name: "Zhipu GLM",
    base_url: "https://open.bigmodel.cn/api/anthropic",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "cn_official",
    color: COLORS.zhipu,
    note: "智谱 AI",
  },
  {
    id: "zhipu_glm_en",
    name: "Z.ai GLM",
    base_url: "https://api.z.ai/api/anthropic",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "cn_official",
    color: COLORS.zhipu,
    note: "智谱国际版",
  },
  {
    id: "kimi",
    name: "Kimi",
    base_url: "https://api.moonshot.cn/anthropic",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "cn_official",
    color: COLORS.moonshot,
    note: "月之暗面",
  },
  {
    id: "kimi_for_coding",
    name: "Kimi For Coding",
    base_url: "https://api.kimi.com/coding",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "cn_official",
    color: COLORS.moonshot,
    note: "Kimi 编程专项",
  },
  {
    id: "qwen_coder",
    name: "Qwen-Coder",
    base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1",
    protocol: "openai",
    provider_kind: "openai",
    category: "cn_official",
    color: COLORS.alibaba,
    note: "通义千问编码",
  },
  {
    id: "bailian",
    name: "阿里百炼",
    base_url: "https://dashscope.aliyuncs.com/apps/anthropic",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "cn_official",
    color: COLORS.alibaba,
    note: "阿里百炼",
  },
  {
    id: "baidu_qianfan",
    name: "百度千帆",
    base_url: "https://qianfan.baidubce.com/anthropic/coding",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "cn_official",
    color: COLORS.baidu,
    note: "百度千帆",
  },
  {
    id: "minimax",
    name: "MiniMax",
    base_url: "https://api.minimaxi.com/anthropic",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "cn_official",
    color: COLORS.minimax,
    note: "MiniMax 国内",
  },
  {
    id: "minimax_en",
    name: "MiniMax en",
    base_url: "https://api.minimax.io/anthropic",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "cn_official",
    color: COLORS.minimax,
    note: "MiniMax 海外",
  },
  {
    id: "stepfun",
    name: "StepFun",
    base_url: "https://api.stepfun.com/step_plan",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "cn_official",
    color: COLORS.stepfun,
    note: "阶跃星辰",
  },
  {
    id: "stepfun_en",
    name: "StepFun en",
    base_url: "https://api.stepfun.ai/step_plan",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "cn_official",
    color: COLORS.stepfun,
    note: "阶跃国际版",
  },
  {
    id: "doubao_seed",
    name: "DouBaoSeed",
    base_url: "https://ark.cn-beijing.volces.com/api/compatible",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "cn_official",
    color: COLORS.bytedance,
    note: "豆包 Seed",
  },
  {
    id: "volcengine_agentplan",
    name: "火山 Agentplan",
    base_url: "https://ark.cn-beijing.volces.com/api/coding",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "cn_official",
    color: COLORS.bytedance,
    note: "字节火山方舟",
  },
  {
    id: "byteplus",
    name: "BytePlus",
    base_url: "https://ark.ap-southeast.bytepluses.com/api/coding",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "cn_official",
    color: COLORS.bytedance,
    note: "字节海外",
  },
  {
    id: "bailing",
    name: "BaiLing",
    base_url: "https://api.tbox.cn/api/anthropic",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "cn_official",
    color: COLORS.alipay,
    note: "支付宝百灵",
  },
  {
    id: "xiaomi_mimo",
    name: "Xiaomi MiMo",
    base_url: "https://api.xiaomimimo.com/anthropic",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "cn_official",
    color: COLORS.xiaomi,
    note: "小米 MiMo",
  },
  {
    id: "kat_coder",
    name: "KAT-Coder",
    base_url: "https://vanchin.streamlake.ai",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "cn_official",
    color: COLORS.vanchin,
    note: "万芯 KAT-Coder",
  },
  {
    id: "longcat",
    name: "Longcat",
    base_url: "https://api.longcat.chat/anthropic",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "cn_official",
    color: COLORS.longcat,
    note: "长猫",
  },

  // ──────────── 云服务 ────────────
  {
    id: "azure_openai",
    name: "Azure OpenAI",
    base_url: "https://YOUR_RESOURCE.openai.azure.com/openai",
    protocol: "openai_responses",
    provider_kind: "azure",
    category: "cloud_provider",
    color: COLORS.microsoft,
    note: "需替换 YOUR_RESOURCE",
  },
  {
    id: "aws_bedrock",
    name: "AWS Bedrock",
    base_url: "https://bedrock-runtime.us-east-1.amazonaws.com",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "cloud_provider",
    color: COLORS.aws,
    note: "需配置 IAM AKSK",
  },
  {
    id: "github_copilot",
    name: "GitHub Copilot",
    base_url: "https://api.githubcopilot.com",
    protocol: "openai",
    provider_kind: "custom",
    category: "cloud_provider",
    color: COLORS.github,
    note: "需 OAuth",
  },
  {
    id: "nvidia_nim",
    name: "Nvidia NIM",
    base_url: "https://integrate.api.nvidia.com",
    protocol: "openai",
    provider_kind: "custom",
    category: "cloud_provider",
    color: COLORS.nvidia,
    note: "NVIDIA NIM",
  },

  // ──────────── 聚合路由 ────────────
  {
    id: "openrouter",
    name: "OpenRouter",
    base_url: "https://openrouter.ai/api",
    protocol: "openai",
    provider_kind: "openrouter",
    category: "aggregator",
    color: COLORS.openrouter,
    note: "50+ 模型路由",
  },
  {
    id: "therouter",
    name: "TheRouter",
    base_url: "https://api.therouter.ai",
    protocol: "openai",
    provider_kind: "custom",
    category: "aggregator",
    color: COLORS.partner4,
    note: "路由聚合",
  },
  {
    id: "siliconflow",
    name: "SiliconFlow",
    base_url: "https://api.siliconflow.cn",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "aggregator",
    color: COLORS.siliconflow,
    note: "硅基流动",
  },
  {
    id: "siliconflow_en",
    name: "SiliconFlow en",
    base_url: "https://api.siliconflow.com",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "aggregator",
    color: COLORS.siliconflow,
    note: "硅基流动海外",
  },
  {
    id: "modelscope",
    name: "ModelScope",
    base_url: "https://api-inference.modelscope.cn",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "aggregator",
    color: COLORS.modelscope,
    note: "阿里魔搭",
  },
  {
    id: "aihubmix",
    name: "AiHubMix",
    base_url: "https://aihubmix.com",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "aggregator",
    color: COLORS.partner1,
    note: "聚合器",
  },
  {
    id: "novita",
    name: "Novita AI",
    base_url: "https://api.novita.ai/anthropic",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "aggregator",
    color: COLORS.novita,
    note: "聚合器",
  },
  {
    id: "dmxapi",
    name: "DMXAPI",
    base_url: "https://www.dmxapi.cn",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "aggregator",
    color: COLORS.partner6,
    note: "聚合器",
  },
  {
    id: "compshare",
    name: "Compshare",
    base_url: "https://api.modelverse.cn",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "aggregator",
    color: COLORS.partner2,
    note: "聚合平台",
  },
  {
    id: "shengsuanyun",
    name: "Shengsuanyun",
    base_url: "https://router.shengsuanyun.com/api",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "aggregator",
    color: COLORS.partner3,
    note: "声算云路由",
  },
  {
    id: "pipellm",
    name: "PIPELLM",
    base_url: "https://cc-api.pipellm.ai",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "aggregator",
    color: COLORS.partner5,
    note: "聚合器",
  },

  // ──────────── 第三方/合作伙伴 ────────────
  {
    id: "packycode",
    name: "PackyCode",
    base_url: "https://www.packyapi.com",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "third_party",
    color: COLORS.packycode,
    note: "合作伙伴",
  },
  {
    id: "claudeapi",
    name: "ClaudeAPI",
    base_url: "https://gw.claudeapi.com",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "third_party",
    color: COLORS.partner1,
  },
  {
    id: "claudecn",
    name: "ClaudeCN",
    base_url: "https://claudecn.top",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "third_party",
    color: COLORS.partner2,
  },
  {
    id: "runapi",
    name: "RunAPI",
    base_url: "https://runapi.co",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "third_party",
    color: COLORS.partner3,
  },
  {
    id: "relaxycode",
    name: "RelaxyCode",
    base_url: "https://www.relaxycode.com",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "third_party",
    color: COLORS.partner4,
  },
  {
    id: "cubence",
    name: "Cubence",
    base_url: "https://api.cubence.com",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "third_party",
    color: COLORS.partner5,
  },
  {
    id: "aigocode",
    name: "AIGoCode",
    base_url: "https://api.aigocode.com",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "third_party",
    color: COLORS.partner6,
  },
  {
    id: "rightcode",
    name: "RightCode",
    base_url: "https://www.right.codes/claude",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "third_party",
    color: COLORS.partner1,
  },
  {
    id: "aicodemirror",
    name: "AICodeMirror",
    base_url: "https://api.aicodemirror.com/api/claudecode",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "third_party",
    color: COLORS.partner2,
  },
  {
    id: "aicoding",
    name: "AICoding",
    base_url: "https://api.aicoding.sh",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "third_party",
    color: COLORS.partner3,
  },
  {
    id: "crazyrouter",
    name: "CrazyRouter",
    base_url: "https://cn.crazyrouter.com",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "third_party",
    color: COLORS.partner4,
  },
  {
    id: "sssaicode",
    name: "SSSAiCode",
    base_url: "https://node-hk.sssaicode.com/api",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "third_party",
    color: COLORS.partner5,
  },
  {
    id: "micu",
    name: "Micu",
    base_url: "https://www.micuapi.ai",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "third_party",
    color: COLORS.partner6,
  },
  {
    id: "ctok",
    name: "CTok.ai",
    base_url: "https://api.ctok.ai",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "third_party",
    color: COLORS.partner1,
  },
  {
    id: "eflowcode",
    name: "E-FlowCode",
    base_url: "https://e-flowcode.cc",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "third_party",
    color: COLORS.partner2,
  },
  {
    id: "lionccapi",
    name: "LionCCAPI",
    base_url: "https://vibecodingapi.ai",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "third_party",
    color: COLORS.partner3,
  },
  {
    id: "lemondata",
    name: "LemonData",
    base_url: "https://api.lemondata.cc",
    protocol: "anthropic",
    provider_kind: "anthropic",
    category: "third_party",
    color: COLORS.partner4,
  },

  // ──────────── 自定义 ────────────
  {
    id: "custom",
    name: "自定义",
    base_url: "",
    protocol: "openai",
    provider_kind: "custom",
    category: "custom",
    color: COLORS.custom,
    note: "完全自定义 base URL",
  },
];

// ──────────────────────────────────────────────────────────────────
// Post-processing：根据 protocol 推断 recommended_for；OAuth 类标记
// ──────────────────────────────────────────────────────────────────
const OAUTH_PROVIDER_IDS = new Set([
  "openai_codex",   // ChatGPT 订阅 OAuth
  "github_copilot", // GitHub OAuth
]);

function inferRecommended(p: ProviderPreset): string[] {
  if (p.recommended_for && p.recommended_for.length > 0) return p.recommended_for;
  if (p.category === "custom") return [];
  switch (p.protocol) {
    case "anthropic":
      // 聚合路由通用：claude + codex
      if (p.category === "aggregator") return ["claude", "codex"];
      return ["claude"];
    case "openai":
      if (p.category === "aggregator") return ["claude", "codex"];
      return ["codex"];
    case "openai_responses":
      return ["codex"];
    case "gemini":
      return ["gemini"];
    case "ollama":
      return [];
    default:
      return [];
  }
}

export const PROVIDER_PRESETS: ProviderPreset[] = RAW_PRESETS.map((p) => ({
  ...p,
  recommended_for: inferRecommended(p),
  auth_method: p.auth_method ?? (OAUTH_PROVIDER_IDS.has(p.id) ? "oauth" : "api_key"),
}));

export const CATEGORY_LABELS: Record<PresetCategory, string> = {
  official: "官方",
  cn_official: "国内大厂",
  cloud_provider: "云服务",
  aggregator: "聚合路由",
  third_party: "第三方",
  self_hosted: "自托管",
  custom: "自定义",
};

/** 按 category 分组，便于 UI 渲染 */
export function groupedPresets(): Array<{
  category: PresetCategory;
  label: string;
  items: ProviderPreset[];
}> {
  const order: PresetCategory[] = [
    "official",
    "cn_official",
    "cloud_provider",
    "aggregator",
    "third_party",
    "self_hosted",
    "custom",
  ];
  return order
    .map((cat) => ({
      category: cat,
      label: CATEGORY_LABELS[cat],
      items: PROVIDER_PRESETS.filter((p) => p.category === cat),
    }))
    .filter((g) => g.items.length > 0);
}

/** 用预设名称首字母（或前 2 字）作为色块图标显示 */
export function presetInitial(p: ProviderPreset): string {
  // 英文取首字母大写；中文取第一个字
  const name = p.name.trim();
  const firstChar = name.charAt(0);
  if (/[一-龥]/.test(firstChar)) return firstChar;
  return firstChar.toUpperCase();
}

// ──────────────────────────────────────────────────────────────────
// 主流厂商品牌 SVG 图标（simple-icons 风格，viewBox 0 0 24 24, 单色 path）
// 其他预设 fallback 到字母色块。
// ──────────────────────────────────────────────────────────────────
export const BRAND_ICONS: Record<string, string> = {
  anthropic_official:
    "M14.281 0H8.66L17.34 24h5.62L14.281 0zm-4.943 0H3.717L0 24h5.621L9.338 0z",
  // OpenAI 旋花
  openai_official:
    "M22.282 9.821a5.985 5.985 0 0 0-.516-4.91 6.046 6.046 0 0 0-6.51-2.9A6.065 6.065 0 0 0 4.981 4.18a5.985 5.985 0 0 0-3.998 2.9 6.046 6.046 0 0 0 .743 7.097 5.98 5.98 0 0 0 .51 4.911 6.051 6.051 0 0 0 6.515 2.9A5.985 5.985 0 0 0 13.26 24a6.056 6.056 0 0 0 5.772-4.206 5.99 5.99 0 0 0 3.997-2.9 6.056 6.056 0 0 0-.747-7.073zM13.26 22.43a4.476 4.476 0 0 1-2.876-1.04l.141-.081 4.779-2.758a.795.795 0 0 0 .392-.681v-6.737l2.02 1.168a.071.071 0 0 1 .038.052v5.583a4.504 4.504 0 0 1-4.494 4.494zM3.6 18.304a4.47 4.47 0 0 1-.535-3.014l.142.085 4.783 2.759a.771.771 0 0 0 .78 0l5.843-3.369v2.332a.08.08 0 0 1-.033.062L9.74 19.95a4.5 4.5 0 0 1-6.14-1.646zM2.34 7.896a4.485 4.485 0 0 1 2.366-1.973V11.6a.766.766 0 0 0 .388.677l5.815 3.354-2.02 1.168a.076.076 0 0 1-.071 0l-4.83-2.786A4.504 4.504 0 0 1 2.34 7.872zm16.597 3.855l-5.833-3.387L15.119 7.2a.076.076 0 0 1 .071 0l4.83 2.791a4.494 4.494 0 0 1-.676 8.105v-5.678a.79.79 0 0 0-.407-.667zm2.01-3.023l-.141-.085-4.774-2.782a.776.776 0 0 0-.785 0L9.409 9.23V6.897a.066.066 0 0 1 .028-.061l4.83-2.787a4.5 4.5 0 0 1 6.68 4.66zm-12.64 4.135l-2.02-1.164a.08.08 0 0 1-.038-.057V6.075a4.5 4.5 0 0 1 7.375-3.453l-.142.08L8.704 5.46a.795.795 0 0 0-.393.681zm1.097-2.365l2.602-1.5 2.607 1.5v2.999l-2.597 1.5-2.607-1.5Z",
  openai_codex:
    "M22.282 9.821a5.985 5.985 0 0 0-.516-4.91 6.046 6.046 0 0 0-6.51-2.9A6.065 6.065 0 0 0 4.981 4.18a5.985 5.985 0 0 0-3.998 2.9 6.046 6.046 0 0 0 .743 7.097 5.98 5.98 0 0 0 .51 4.911 6.051 6.051 0 0 0 6.515 2.9A5.985 5.985 0 0 0 13.26 24a6.056 6.056 0 0 0 5.772-4.206 5.99 5.99 0 0 0 3.997-2.9 6.056 6.056 0 0 0-.747-7.073zM13.26 22.43a4.476 4.476 0 0 1-2.876-1.04l.141-.081 4.779-2.758a.795.795 0 0 0 .392-.681v-6.737l2.02 1.168a.071.071 0 0 1 .038.052v5.583a4.504 4.504 0 0 1-4.494 4.494zM3.6 18.304a4.47 4.47 0 0 1-.535-3.014l.142.085 4.783 2.759a.771.771 0 0 0 .78 0l5.843-3.369v2.332a.08.08 0 0 1-.033.062L9.74 19.95a4.5 4.5 0 0 1-6.14-1.646zM2.34 7.896a4.485 4.485 0 0 1 2.366-1.973V11.6a.766.766 0 0 0 .388.677l5.815 3.354-2.02 1.168a.076.076 0 0 1-.071 0l-4.83-2.786A4.504 4.504 0 0 1 2.34 7.872zm16.597 3.855l-5.833-3.387L15.119 7.2a.076.076 0 0 1 .071 0l4.83 2.791a4.494 4.494 0 0 1-.676 8.105v-5.678a.79.79 0 0 0-.407-.667zm2.01-3.023l-.141-.085-4.774-2.782a.776.776 0 0 0-.785 0L9.409 9.23V6.897a.066.066 0 0 1 .028-.061l4.83-2.787a4.5 4.5 0 0 1 6.68 4.66zm-12.64 4.135l-2.02-1.164a.08.08 0 0 1-.038-.057V6.075a4.5 4.5 0 0 1 7.375-3.453l-.142.08L8.704 5.46a.795.795 0 0 0-.393.681zm1.097-2.365l2.602-1.5 2.607 1.5v2.999l-2.597 1.5-2.607-1.5Z",
  // Google Gemini（双菱形花）
  gemini_official:
    "M24 12c-6.626 0-12 5.373-12 12 0-6.627-5.373-12-12-12 6.627 0 12-5.373 12-12 0 6.627 5.374 12 12 12",
  // GitHub Octocat
  github_copilot:
    "M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12",
  // Nvidia 之眼简化
  nvidia_nim:
    "M11.166 7.792v-1.71c.168-.013.337-.024.508-.024 4.696-.147 7.776 4.034 7.776 4.034s-3.326 4.62-6.892 4.62c-.483 0-.949-.078-1.39-.215v-5.193c1.832.223 2.196 1.022 3.293 2.852l2.452-2.069s-1.79-2.346-4.808-2.346c-.328 0-.65.024-.94.05zm0-5.667v2.556c.168-.013.337-.024.508-.024 6.526-.22 10.781 5.353 10.781 5.353s-4.881 5.949-9.97 5.949c-.466 0-.91-.05-1.32-.122v1.583c.35.045.708.072 1.082.072 4.74 0 8.165-2.42 11.484-5.286.55.44 2.804 1.514 3.27 1.985-3.156 2.643-10.498 4.776-14.665 4.776-.404 0-.79-.024-1.171-.061v2.227h18.124V2.125H11.166z",
  // AWS Bedrock - 简化盒子
  aws_bedrock:
    "M6.763 10.036c0 .296.032.535.088.71.064.176.144.368.256.576.04.063.056.127.056.183 0 .08-.048.16-.152.24l-.503.335a.383.383 0 0 1-.208.072c-.08 0-.16-.04-.239-.112a2.473 2.473 0 0 1-.287-.375 6.18 6.18 0 0 1-.248-.471c-.622.734-1.405 1.101-2.347 1.101-.67 0-1.205-.191-1.596-.574-.391-.384-.59-.894-.59-1.533 0-.678.239-1.23.726-1.644.487-.415 1.133-.623 1.955-.623.272 0 .551.024.846.064.296.04.6.104.918.176v-.583c0-.607-.127-1.03-.375-1.277-.255-.248-.686-.367-1.3-.367-.28 0-.567.031-.862.103-.295.072-.583.16-.862.272a2.287 2.287 0 0 1-.28.104.488.488 0 0 1-.127.023c-.112 0-.168-.08-.168-.247v-.391c0-.128.016-.224.056-.28a.597.597 0 0 1 .224-.167c.279-.144.614-.264 1.005-.36a4.84 4.84 0 0 1 1.246-.151c.95 0 1.644.216 2.091.647.439.43.662 1.085.662 1.963v2.586zm-3.24 1.214c.263 0 .534-.048.822-.144.287-.096.543-.271.758-.51.128-.152.224-.32.272-.512.047-.191.08-.423.08-.694v-.335a6.66 6.66 0 0 0-.735-.136 6.02 6.02 0 0 0-.75-.048c-.535 0-.926.104-1.19.32-.263.215-.39.518-.39.917 0 .375.095.655.295.846.191.2.47.295.838.295zm6.41.862c-.144 0-.24-.024-.304-.08-.064-.048-.12-.16-.168-.311L7.586 5.55a1.398 1.398 0 0 1-.072-.32c0-.128.064-.2.191-.2h.783c.151 0 .255.025.31.08.065.048.113.16.16.312l1.342 5.284 1.245-5.284c.04-.16.088-.264.151-.312a.549.549 0 0 1 .32-.08h.638c.152 0 .256.025.32.08.063.048.12.16.151.312l1.261 5.348 1.381-5.348c.048-.16.104-.264.16-.312a.52.52 0 0 1 .311-.08h.743c.127 0 .2.065.2.2 0 .04-.009.08-.017.128a1.137 1.137 0 0 1-.056.2l-1.923 6.17c-.048.16-.104.263-.168.311a.51.51 0 0 1-.303.08h-.687c-.151 0-.255-.024-.32-.08-.063-.056-.119-.16-.15-.32l-1.238-5.148-1.23 5.14c-.04.16-.087.264-.15.32-.065.056-.177.08-.32.08zm10.256.215c-.415 0-.83-.048-1.229-.143-.399-.096-.71-.2-.918-.32-.128-.071-.215-.151-.247-.223a.563.563 0 0 1-.048-.224v-.407c0-.167.064-.247.183-.247.048 0 .096.008.144.024.048.016.12.048.2.08.271.12.566.215.878.279.319.064.63.096.95.096.502 0 .894-.088 1.165-.264a.86.86 0 0 0 .415-.758.778.778 0 0 0-.215-.559c-.144-.151-.416-.287-.807-.415l-1.157-.36c-.583-.183-1.014-.454-1.277-.813a1.902 1.902 0 0 1-.4-1.158c0-.335.073-.63.216-.886.144-.255.335-.479.575-.654.24-.184.51-.32.83-.415.32-.096.655-.136 1.006-.136.175 0 .359.008.535.032.183.024.35.056.518.088.16.04.312.08.455.127.144.048.256.096.336.144a.69.69 0 0 1 .24.2.43.43 0 0 1 .071.263v.375c0 .168-.064.256-.184.256a.83.83 0 0 1-.303-.096 3.652 3.652 0 0 0-1.532-.311c-.455 0-.815.071-1.062.223-.248.152-.375.383-.375.71 0 .224.08.416.24.567.159.152.454.304.877.44l1.134.358c.574.184.99.44 1.237.768.247.327.367.702.367 1.117 0 .343-.072.655-.207.926-.144.272-.336.511-.583.703-.248.2-.543.343-.886.447-.36.111-.734.167-1.142.167zM21.698 16.207c-2.626 1.94-6.442 2.969-9.722 2.969-4.598 0-8.74-1.7-11.87-4.526-.247-.223-.024-.527.27-.351 3.384 1.963 7.559 3.153 11.877 3.153 2.914 0 6.114-.607 9.06-1.852.439-.2.814.287.385.607zm1.094-1.246c-.336-.43-2.22-.207-3.074-.103-.255.032-.295-.192-.063-.36 1.5-1.053 3.967-.75 4.254-.399.287.36-.08 2.826-1.485 4.007-.215.184-.423.088-.327-.151.32-.79 1.03-2.57.694-2.994z",
  // 中国大厂用简化色块字母（fallback），不画 logo
};

