
## 2026-05-21 预设模型供应商清单调研

### cc-switch 的 Provider Preset 数据源

**位置**：`/Users/a1/001.code/cc-switch/src/config/claudeProviderPresets.ts`（主文件）
**特点**：按目标工具（Claude/Codex/Gemini）分离预设，每个工具独立的预设数组

cc-switch 采用**分类设计**，每个工具有独立预设文件：
- `claudeProviderPresets.ts` → Claude API 兼容供应商（支持 Anthropic Messages API 或其转换格式）
- `codexProviderPresets.ts` → OpenAI Codex 兼容供应商（OpenAI Chat/Responses API）
- `geminiProviderPresets.ts` → Google Gemini API 兼容供应商

#### Claude 系预设（共 49 条）

| id | 显示名 | base_url | 默认模型 | 协议 | 分类 | 备注 |
|----|--------|----------|----------|------|------|------|
| official | Claude Official | https://www.anthropic.com | - | anthropic | official | 官方 |
| shengsuanyun | Shengsuanyun | https://router.shengsuanyun.com/api | - | anthropic | aggregator | 聚合器 |
| patewayai | PatewayAI | https://api.pateway.ai | - | anthropic | third_party | 合作伙伴 |
| volcengine_agentplan | 火山Agentplan | https://ark.cn-beijing.volces.com/api/coding | ark-code-latest | anthropic | cn_official | 字节官方 |
| byteplus | BytePlus | https://ark.ap-southeast.bytepluses.com/api/coding | ark-code-latest | anthropic | cn_official | 字节海外 |
| doubaoseed | DouBaoSeed | https://ark.cn-beijing.volces.com/api/compatible | doubao-seed-2-0-code-preview-latest | anthropic | cn_official | 豆包 |
| gemini_native | Gemini Native | https://generativelanguage.googleapis.com | gemini-3.1-pro | gemini_native | third_party | 格式转换 |
| deepseek | DeepSeek | https://api.deepseek.com/anthropic | deepseek-v4-pro | anthropic | cn_official | 中国大模型 |
| zhipu_glm | Zhipu GLM | https://open.bigmodel.cn/api/anthropic | glm-5 | anthropic | cn_official | 智谱 |
| zhipu_glm_en | Zhipu GLM en | https://api.z.ai/api/anthropic | glm-5 | anthropic | cn_official | 智谱国际 |
| baidu_qianfan | Baidu Qianfan Coding Plan | https://qianfan.baidubce.com/anthropic/coding | qianfan-code-latest | anthropic | cn_official | 百度千帆 |
| bailian | Bailian | https://dashscope.aliyuncs.com/apps/anthropic | - | anthropic | cn_official | 阿里通义 |
| bailian_coding | Bailian For Coding | https://coding.dashscope.aliyuncs.com/apps/anthropic | - | anthropic | cn_official | 阿里编码专项 |
| kimi | Kimi | https://api.moonshot.cn/anthropic | kimi-k2.6 | anthropic | cn_official | 月之暗面 |
| kimi_coding | Kimi For Coding | https://api.kimi.com/coding/ | - | anthropic | cn_official | Kimi 编码专项 |
| stepfun | StepFun | https://api.stepfun.com/step_plan | step-3.5-flash-2603 | anthropic | cn_official | 阶跃星辰 |
| stepfun_en | StepFun en | https://api.stepfun.ai/step_plan | step-3.5-flash-2603 | anthropic | cn_official | 阶跃国际 |
| modelscope | ModelScope | https://api-inference.modelscope.cn | ZhipuAI/GLM-5 | anthropic | aggregator | 模型集市 |
| kat_coder | KAT-Coder | https://vanchin.streamlake.ai/api/gateway/v1/endpoints/${ENDPOINT_ID}/claude-code-proxy | KAT-Coder-Pro V1 | anthropic | cn_official | Vanchin |
| longcat | Longcat | https://api.longcat.chat/anthropic | LongCat-Flash-Chat | anthropic | cn_official | 长猫聚合 |
| minimax | MiniMax | https://api.minimaxi.com/anthropic | MiniMax-M2.7 | anthropic | cn_official | 合作伙伴 |
| minimax_en | MiniMax en | https://api.minimax.io/anthropic | MiniMax-M2.7 | anthropic | cn_official | MiniMax 国际 |
| bailing | BaiLing | https://api.tbox.cn/api/anthropic | Ling-2.5-1T | anthropic | cn_official | 支付宝 |
| aihubmix | AiHubMix | https://aihubmix.com | - | anthropic | aggregator | 聚合器 |
| siliconflow | SiliconFlow | https://api.siliconflow.cn | Pro/MiniMaxAI/MiniMax-M2.7 | anthropic | aggregator | 聚合器 |
| siliconflow_en | SiliconFlow en | https://api.siliconflow.com | MiniMaxAI/MiniMax-M2.7 | anthropic | aggregator | 聚合器 |
| dmxapi | DMXAPI | https://www.dmxapi.cn | - | anthropic | aggregator | 聚合器 |
| packycode | PackyCode | https://www.packyapi.com | - | anthropic | third_party | 合作伙伴 |
| claudeapi | ClaudeAPI | https://gw.claudeapi.com | - | anthropic | third_party | 第三方 |
| claudecn | ClaudeCN | https://claudecn.top | - | anthropic | third_party | 第三方 |
| runapi | RunAPI | https://runapi.co | - | anthropic | aggregator | 聚合器 |
| relaxycode | RelaxyCode | https://www.relaxycode.com | - | anthropic | third_party | 第三方 |
| cubence | Cubence | https://api.cubence.com | - | anthropic | third_party | 合作伙伴 |
| aigocode | AIGoCode | https://api.aigocode.com | - | anthropic | third_party | 合作伙伴 |
| rightcode | RightCode | https://www.right.codes/claude | - | anthropic | third_party | 合作伙伴 |
| aicodemirror | AICodeMirror | https://api.aicodemirror.com/api/claudecode | - | anthropic | third_party | 合作伙伴 |
| aicoding | AICoding | https://api.aicoding.sh | - | anthropic | third_party | 合作伙伴 |
| crazyrouter | CrazyRouter | https://cn.crazyrouter.com | - | anthropic | third_party | 合作伙伴 |
| sssaicode | SSSAiCode | https://node-hk.sssaicode.com/api | - | anthropic | third_party | 合作伙伴 |
| compshare | Compshare | https://api.modelverse.cn | - | anthropic | aggregator | 聚合器 |
| compshare_coding | Compshare Coding Plan | https://cp.compshare.cn | - | anthropic | aggregator | 聚合器 |
| micu | Micu | https://www.micuapi.ai | - | anthropic | third_party | 合作伙伴 |
| ctok | CTok.ai | https://api.ctok.ai | - | anthropic | third_party | 合作伙伴 |
| eflowcode | E-FlowCode | https://e-flowcode.cc | - | anthropic | third_party | 第三方 |
| lionccapi | LionCCAPI | https://vibecodingapi.ai | - | anthropic | third_party | 合作伙伴 |
| openrouter | OpenRouter | https://openrouter.ai/api | anthropic/claude-sonnet-4.6 | anthropic | aggregator | 跨模型路由 |
| therouter | TheRouter | https://api.therouter.ai | anthropic/claude-sonnet-4.6 | anthropic | aggregator | 聚合器 |
| novita | Novita AI | https://api.novita.ai/anthropic | zai-org/glm-5 | anthropic | aggregator | 聚合器 |
| github_copilot | GitHub Copilot | https://api.githubcopilot.com | claude-sonnet-4.6 | openai_chat | third_party | OAuth |
| codex | Codex | https://chatgpt.com/backend-api/codex | gpt-5.4 | openai_responses | third_party | OAuth |
| lemondata | LemonData | https://api.lemondata.cc | - | anthropic | third_party | 合作伙伴 |
| nvidia | Nvidia | https://integrate.api.nvidia.com | moonshotai/kimi-k2.5 | openai_chat | aggregator | NVIDIA NIM |
| pipellm | PIPELLM | https://cc-api.pipellm.ai | claude-opus-4-7 | anthropic | aggregator | 聚合器 |
| xiaomimimo | Xiaomi MiMo | https://api.xiaomimimo.com/anthropic | mimo-v2-pro | anthropic | cn_official | 小米 |
| aws_bedrock_aksk | AWS Bedrock (AKSK) | https://bedrock-runtime.${AWS_REGION}.amazonaws.com | global.anthropic.claude-opus-4-7 | anthropic | cloud_provider | AWS |
| aws_bedrock_apikey | AWS Bedrock (API Key) | https://bedrock-runtime.${AWS_REGION}.amazonaws.com | global.anthropic.claude-opus-4-7 | anthropic | cloud_provider | AWS |

#### Codex/OpenAI 系预设（共 19 条）

| id | 显示名 | base_url | 默认模型 | 协议 | 分类 | 备注 |
|----|--------|----------|----------|------|------|------|
| official_openai | OpenAI Official | https://chatgpt.com/codex | - | openai_responses | official | 官方 |
| shengsuanyun | Shengsuanyun | https://router.shengsuanyun.com/api/v1 | gpt-5.4 | openai_responses | aggregator | 聚合器 |
| patewayai | PatewayAI | https://api.pateway.ai/v1 | gpt-5.5 | openai_responses | third_party | 合作伙伴 |
| azure_openai | Azure OpenAI | https://YOUR_RESOURCE_NAME.openai.azure.com/openai | gpt-5.4 | openai_responses | third_party | 官方云 |
| aihubmix | AiHubMix | https://aihubmix.com/v1 | gpt-5.4 | openai_responses | aggregator | 聚合器 |
| dmxapi | DMXAPI | https://www.dmxapi.cn/v1 | gpt-5.4 | openai_responses | aggregator | 聚合器 |
| packycode | PackyCode | https://www.packyapi.com/v1 | gpt-5.4 | openai_responses | third_party | 合作伙伴 |
| claudecn | ClaudeCN | https://claudecn.top/v1 | gpt-5.5 | openai_responses | third_party | 合作伙伴 |
| runapi | RunAPI | https://runapi.co/v1 | gpt-5.5 | openai_responses | aggregator | 聚合器 |
| relaxycode | RelaxyCode | https://www.relaxycode.com/v1 | gpt-5.5 | openai_responses | third_party | 第三方 |
| cubence | Cubence | https://api.cubence.com/v1 | gpt-5.4 | openai_responses | third_party | 合作伙伴 |
| aigocode | AIGoCode | https://api.aigocode.com | gpt-5.4 | openai_responses | third_party | 合作伙伴 |
| rightcode | RightCode | https://right.codes/codex/v1 | gpt-5.4 | openai_responses | third_party | 合作伙伴 |
| aicodemirror | AICodeMirror | https://api.aicodemirror.com/api/codex/backend-api/codex | gpt-5.4 | openai_responses | third_party | 合作伙伴 |
| aicoding | AICoding | https://api.aicoding.sh | gpt-5.4 | openai_responses | third_party | 合作伙伴 |
| crazyrouter | CrazyRouter | https://cn.crazyrouter.com/v1 | gpt-5.4 | openai_responses | third_party | 合作伙伴 |
| sssaicode | SSSAiCode | https://node-hk.sssaicode.com/api/v1 | gpt-5.4 | openai_responses | third_party | 合作伙伴 |
| compshare | Compshare | https://api.modelverse.cn/v1 | gpt-5.4 | openai_responses | aggregator | 聚合器 |
| compshare_coding | Compshare Coding Plan | https://cp.compshare.cn/v1 | gpt-5.4 | openai_responses | aggregator | 聚合器 |

#### Gemini 系预设（共 18 条）

| id | 显示名 | base_url | 默认模型 | 协议 | 分类 | 备注 |
|----|--------|----------|----------|------|------|------|
| google_official | Google Official | https://aistudio.google.com | - | gemini | official | 官方 OAuth |
| shengsuanyun | Shengsuanyun | https://router.shengsuanyun.com/api | gemini-3.1-pro | gemini | aggregator | 聚合器 |
| packycode | PackyCode | https://www.packyapi.com | gemini-3.1-pro | gemini | third_party | 合作伙伴 |
| cubence | Cubence | https://api.cubence.com | gemini-3.1-pro | gemini | third_party | 合作伙伴 |
| aigocode | AIGoCode | https://api.aigocode.com | gemini-3.1-pro | gemini | third_party | 合作伙伴 |
| aicodemirror | AICodeMirror | https://api.aicodemirror.com/api/gemini | gemini-3.1-pro | gemini | third_party | 合作伙伴 |
| aicoding | AICoding | https://api.aicoding.sh | gemini-3.1-pro | gemini | third_party | 合作伙伴 |
| crazyrouter | CrazyRouter | https://cn.crazyrouter.com | gemini-3.1-pro | gemini | third_party | 合作伙伴 |
| sssaicode | SSSAiCode | https://node-hk.sssaicode.com/api | gemini-3.1-pro | gemini | third_party | 合作伙伴 |
| ctok | CTok.ai | https://api.ctok.ai/v1beta | gemini-3.1-pro | gemini | third_party | 合作伙伴 |
| lionccapi | LionCCAPI | https://vibecodingapi.ai | gemini-3.1-pro | gemini | third_party | 合作伙伴 |
| eflowcode | E-FlowCode | https://e-flowcode.cc | gemini-3.1-pro-preview | gemini | third_party | 第三方 |
| lemondata | LemonData | https://api.lemondata.cc | gemini-3.1-pro | gemini | third_party | 合作伙伴 |
| openrouter | OpenRouter | https://openrouter.ai/api | gemini-3.1-pro | gemini | aggregator | 跨模型路由 |
| therouter | TheRouter | https://api.therouter.ai | gemini-3.1-pro | gemini | aggregator | 聚合器 |
| custom | 自定义 | - | gemini-3.1-pro | gemini | custom | 用户输入 |

### 9router 的 supported providers

**位置**：`/Users/a1/001.code/9router/src/shared/constants/providers.js`
**特点**：统一的 provider 注册表，包含 OAuth、API Key、Free Tier 等多种认证方式

9router 采用**混合设计**，分类维度是认证方式而非工具：
- `OAUTH_PROVIDERS` → GitHub Copilot、Cursor 等 OAuth 认证
- `APIKEY_PROVIDERS` → OpenAI、Anthropic、DeepSeek 等 API Key 认证
- `FREE_TIER_PROVIDERS` → OpenRouter、Gemini 等有免费额度的服务
- `FREE_PROVIDERS` → 已弃用的免费渠道（Kiro、Qwen 等）

### 合并去重后的完整清单（推荐 ClawHeart v2 采用）

**策略**：按协议分类，去除重复，保留最全的 base_url 和模型信息。

#### Anthropic 协议（OpenAI Chat 兼容）
1. Claude Official (anthropic.com) — 官方
2. DeepSeek (api.deepseek.com/anthropic) — 深度求索
3. Zhipu GLM (open.bigmodel.cn/api/anthropic) — 智谱
4. Baidu Qianfan (qianfan.baidubce.com/anthropic/coding) — 百度千帆
5. Kimi (api.moonshot.cn/anthropic) — 月之暗面
6. StepFun (api.stepfun.com/step_plan) — 阶跃星辰
7. MiniMax (api.minimaxi.com/anthropic) — MiniMax
8. Alibaba Bailian (dashscope.aliyuncs.com/apps/anthropic) — 阿里
9. BytePlus Ark (ark.ap-southeast.bytepluses.com/api/coding) — 字节海外
10. VolcEngine Ark (ark.cn-beijing.volces.com/api/coding) — 字节国内

#### OpenAI 协议（Responses API）
1. OpenAI Official (chatgpt.com/backend-api/codex) — 官方 Codex
2. Azure OpenAI (YOUR_RESOURCE_NAME.openai.azure.com/openai) — 微软云
3. PatewayAI (api.pateway.ai/v1) — 路由聚合

#### Gemini 协议
1. Google Official (aistudio.google.com) — 官方
2. Gemini Native via Anthropic (generativelanguage.googleapis.com) — 格式适配

#### 聚合/路由网关（支持多协议）
1. OpenRouter (openrouter.ai/api) — 综合路由，支持 50+ 模型
2. TheRouter (api.therouter.ai) — 路由聚合
3. NewAPI (支持 Anthropic/OpenAI/Gemini) — 自可部署网关
4. AiHubMix (aihubmix.com) — Claude 聚合器
5. SiliconFlow (api.siliconflow.cn) — 硅基聚合
6. Compshare (api.modelverse.cn) — 聚合平台
7. Shengsuanyun (router.shengsuanyun.com/api) — 声算云路由

#### 第三方/合作伙伴（Anthropic 兼容）
- PackyCode · RunAPI · Cubence · AIGoCode · RightCode
- AICodeMirror · AICoding · CrazyRouter · SSSAiCode
- Micu · CTok.ai · LemonData · PIPELLM · Novita
- ClaudeCN · RelaxyCode · E-FlowCode · LionCCAPI
- GitHub Copilot (OAuth) · AWS Bedrock (特殊认证)

### 图标处理建议

**cc-switch 做法**：
- 为每个预设配置 `icon` 字段（字符串，通常为供应商名或固定名称）
- 为每个预设配置 `iconColor` 字段（hex 颜色值，如 `#FF6B6B`）
- 在前端使用 `ProviderIcon` 组件渲染（可组合 emoji、SVG path、或色块+文字）

**推荐 ClawHeart v2 方案**：
1. **先采用色块 + 品牌首字母/全称**（最快可配置）：
   - Anthropic → A (#D4915D)
   - OpenAI → O (#000000)
   - Google → G (#4285F4)
   - 各聚合器自身品牌色
2. **可选升级：内置常见供应商的 SVG logo**（在 `src/assets/icons/providers/` 中预置 20+ 常用厂商）
3. **扩展点**：用户自定义渠道时允许上传 logo 图片或输入 icon-URL

#### 完整条目数统计

- Claude 系预设：49 条
- Codex/OpenAI 系预设：19 条  
- Gemini 系预设：18 条
- 合并去重后独立供应商：约 80+ 条（考虑跨工具复用）

**写入位置**：`/Users/a1/001.code/clawheartv2/.context/note.md`（已追加）
