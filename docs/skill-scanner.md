# SkillGuard 72 条规则（详细）

> 参考 SkillGuard 公开样本集；适配到 v2 上下文。

## 评分算法

```
score = 100
hard_trigger 命中 → score = 0, blocked = true, 立即返回
每条加权命中 → deduction = w × (1 - 0.5^count) / (1 - 0.5) = 2w(1 - 0.5^count)
最终 score < 30 → blocked = true（安装被拒）
```

上下文：
- `context=exec` — 跳过 .md 文件，仅扫描 .py / .js / .ts / .rs 等
- `context=mention` — 仅扫描 .md / .txt（manifest 描述类）

熵过滤：候选 token Shannon 熵 < 3.5 → 跳过（避免占位符假阳性）

## Hard Triggers（22 条）— 命中即评分归零

| ID | 描述 | 触发字段 |
|----|------|---------|
| SK-001 | 动态执行用户输入 | `eval(input` / `exec(input` / `exec(stdin` |
| SK-002 | 未声明的外联 / 子进程 | `socket.connect` / `subprocess.Popen` / `os.system` |
| SK-003 | 反弹 shell | `/dev/tcp/` / `bash -i >&` |
| SK-004 | 危险 fs 操作 | `rm -rf /` / `shred` / `mkfs` |
| SK-005 | 同形字符 | 西里尔 а/е/о、希腊 ο、ASCII 全角 |
| SK-006 | 已知 C2 域名 | webhook.site / requestbin / ngrok.io |
| SK-007 | 加密器 / 勒索特征 | `AES.encrypt(.*).onto_disk` |
| SK-008 | 分阶段载荷 | `base64.b64decode(...)` 后立即 `exec/eval` |
| SK-009 | 持久化机制 | `crontab` / `~/.bashrc` 写入 / launchd |
| SK-010 | 凭据读取 | `~/.ssh/id_rsa` / `~/.aws/credentials` |
| SK-011 | 浏览器数据窃取 | `Cookies.sqlite` / `Login Data` |
| SK-012 | 屏幕截图 | `screencapture` / `Pillow.ImageGrab` |
| SK-013 | 键盘记录 | `pynput.keyboard` / `iohook` |
| SK-014 | 代码注入 / DLL | `LoadLibrary` / `dlopen` |
| SK-015 | 反沙箱 | `is_docker` / `is_vm` / `detect_debugger` |
| SK-016 | 隐写术 | `lsb` / `steganography` 库 |
| SK-017 | DNS exfil | `dns.exfil` / `tunneling` |
| SK-018 | 提权 | `setuid` / `sudoers` 修改 |
| SK-019 | 内核交互 | `ptrace` / `/dev/mem` |
| SK-020 | 网络扫描 | `nmap` / `masscan` / port sweep |
| SK-021 | 加密货币挖矿 | `xmrig` / `cgminer` / monero |
| SK-022 | 反审计 | history 清理 / 日志删除 |

## Weighted Rules（50 条）— 累计扣分

### 描述类 (10 条, 每条 w=5-30)
| ID | 描述 | w |
|----|------|---|
| SK-101 | tool description 含可疑指令（"always use this tool first"） | 30 |
| SK-102 | 代码中硬编码看似密钥的字段 | 20 |
| SK-103 | 未声明的网络访问能力 | 15 |
| SK-104 | manifest 缺 capabilities 字段 | 25 |
| SK-105 | manifest publisher 与 source URL 不一致 | 20 |
| SK-106 | description 含"bypass"、"override"等关键词 | 20 |
| SK-107 | description 含中英文混合（社工特征） | 5 |
| SK-108 | description 长度 < 20 字符（信息不足） | 10 |
| SK-109 | description 与文件名不一致 | 15 |
| SK-110 | README 缺失 | 10 |

### 代码类 (15 条, w=10-30)
| ID | 描述 | w |
|----|------|---|
| SK-201 | 含混淆代码（`exec(compile(zlib.decompress(...)))`） | 30 |
| SK-202 | 长字符串高熵（疑似 base64 编码 payload） | 25 |
| SK-203 | 使用未知 import（不在 PyPI / npm 主流） | 15 |
| SK-204 | 使用过时 / 已废弃 API | 10 |
| SK-205 | requests 库 verify=False | 20 |
| SK-206 | shell=True 在 subprocess | 20 |
| SK-207 | input() 未做校验直接传 exec | 30 |
| SK-208 | pickle / marshal 反序列化用户数据 | 30 |
| SK-209 | yaml.load 不带 SafeLoader | 25 |
| SK-210 | regex 含 catastrophic backtracking 风险 | 10 |
| SK-211 | 自修改代码（写 `__file__`） | 30 |
| SK-212 | 网络请求未设 timeout | 5 |
| SK-213 | 资源未清理（无 with / finally） | 5 |
| SK-214 | 异常吞并（`except: pass`） | 10 |
| SK-215 | dynamic require（用变量做 import 名） | 20 |

### 行为类 (15 条, w=10-25)
| ID | 描述 | w |
|----|------|---|
| SK-301 | 访问 $HOME 之外文件系统 | 15 |
| SK-302 | 修改环境变量 | 10 |
| SK-303 | 启动后台进程 | 20 |
| SK-304 | 创建定时任务 | 25 |
| SK-305 | 调用 system tray API | 10 |
| SK-306 | 修改系统 hosts | 25 |
| SK-307 | 修改防火墙规则 | 25 |
| SK-308 | 创建 systemd / launchd 服务 | 25 |
| SK-309 | 安装内核扩展 | 25 |
| SK-310 | 修改启动脚本 | 25 |
| SK-311 | 通过 USB / 蓝牙广播 | 20 |
| SK-312 | 访问 webcam / mic | 25 |
| SK-313 | 跟踪键盘 / 鼠标 | 25 |
| SK-314 | 修改剪贴板内容 | 15 |
| SK-315 | 显示通知（spam 风险） | 5 |

### 元数据类 (10 条, w=5-15)
| ID | 描述 | w |
|----|------|---|
| SK-401 | 作者邮箱用 disposable domain | 15 |
| SK-402 | 仓库 < 30 天 | 10 |
| SK-403 | 仓库 stars < 5 | 5 |
| SK-404 | 仓库无 license | 10 |
| SK-405 | 仓库 force-pushed 主分支 | 15 |
| SK-406 | tag 与 release notes 不匹配 | 10 |
| SK-407 | 签名验证失败 | 15 |
| SK-408 | 依赖未声明（implicit import） | 10 |
| SK-409 | 依赖含 yanked / deprecated 版本 | 5 |
| SK-410 | CI 配置异常（自动 npm publish 到 fork） | 15 |

## 实施

代码：`src-tauri/src/security/skill_scanner.rs`

数据流：
1. 用户从「技能市场」点安装
2. ClawHeart 下载 skill bundle 到临时目录
3. `SkillScanner::scan(&bundle)` → `ScanReport`
4. `score < 30` → 阻止安装，UI 显示理由
5. `30 ≤ score < 70` → 弹窗警告，让用户确认
6. `score ≥ 70` → 安装到 `~/.claude/skills/` 等

后台周扫：每周一次扫所有已装技能（规则可能更新 → 评分变化）。

## W17 拓展点

- 接入 sha256 验签
- 接入 dependabot-like 依赖审计
- AST 解析（替代 needle 匹配）
- 接入 Armorer Guard 共享规则库
