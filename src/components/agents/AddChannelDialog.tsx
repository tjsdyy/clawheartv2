/**
 * 「新增模型渠道」抽屉式弹层。
 *
 * 参考 cc-switch UI：
 * 1. 顶部：Provider Preset 横排 chip（按 category 分组）
 * 2. 中部：色块图标 + Provider 名称
 * 3. 表单：Provider Name / Notes / Website URL（自动填充）/ API Key
 * 4. 底部：[Cancel] [+ Add]
 */
import { useEffect, useState } from "react";
import { X, Plus, Loader2, Save, KeyRound, Shield, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { cn } from "@/lib/utils";
import {
  PROVIDER_PRESETS,
  groupedPresets,
  type ProviderPreset,
} from "@/data/provider-presets";
import {
  useCreateProvider,
  useUpdateProvider,
  useSetProviderCredential,
  useDeleteProvider,
  type ProviderProfile,
} from "@/hooks/useProviders";
import { useAssignChannel } from "@/hooks/useChannelAssignments";
import { BrandIcon } from "./BrandIcon";
import { ConfirmDialog } from "@/components/ui/ConfirmDialog";

interface Props {
  onClose: () => void;
  /** 推荐的协议平台（如当前 tab 的 platform=claude）；会优先排序匹配的预设 */
  recommendedPlatform?: string;
  /** 传入则进入编辑模式（隐藏 Preset 网格，预填表单） */
  editingProfile?: ProviderProfile;
  /** 新建成功后自动分配给该 Agent（agent_id 格式：platform/agent_name） */
  autoAssignToAgent?: string;
}

export function AddChannelDialog({
  onClose,
  recommendedPlatform,
  editingProfile,
  autoAssignToAgent,
}: Props) {
  const isEditing = !!editingProfile;

  // 编辑模式：找到匹配的预设（按 base_url + protocol），找不到则 null
  const matchedPreset = isEditing
    ? PROVIDER_PRESETS.find(
        (p) =>
          p.base_url === editingProfile.base_url &&
          p.protocol === editingProfile.protocol,
      ) ?? null
    : null;

  const [selectedPreset, setSelectedPreset] = useState<ProviderPreset | null>(
    isEditing ? matchedPreset : PROVIDER_PRESETS[0] ?? null,
  );
  const [name, setName] = useState(isEditing ? editingProfile.name : "");
  const [notes, setNotes] = useState("");
  const [baseUrl, setBaseUrl] = useState(
    isEditing ? editingProfile.base_url : "",
  );
  const [apiKey, setApiKey] = useState("");

  const createProvider = useCreateProvider();
  const updateProvider = useUpdateProvider();
  const setCredential = useSetProviderCredential();
  const deleteProvider = useDeleteProvider();
  const assignChannel = useAssignChannel();

  const isOauth = selectedPreset?.auth_method === "oauth";
  const submitting =
    createProvider.isPending ||
    updateProvider.isPending ||
    setCredential.isPending ||
    deleteProvider.isPending;

  const [confirmDeleteOpen, setConfirmDeleteOpen] = useState(false);

  async function doDelete() {
    if (!editingProfile) return;
    try {
      await deleteProvider.mutateAsync(editingProfile.id);
      toast.success(`已删除「${editingProfile.name}」`);
      setConfirmDeleteOpen(false);
      onClose();
    } catch (e) {
      toast.error(`删除失败：${e}`);
    }
  }

  // 选中预设时自动填充（编辑模式下，仅在用户主动切换预设时才覆盖）
  useEffect(() => {
    if (!selectedPreset) return;
    if (isEditing) return; // 编辑模式：保留用户原值
    setName(selectedPreset.name);
    setBaseUrl(selectedPreset.base_url);
    setNotes(selectedPreset.note ?? "");
  }, [selectedPreset?.id, isEditing]); // eslint-disable-line react-hooks/exhaustive-deps

  const groups = groupedPresets();

  // 按 recommended_for 把推荐的置顶（只影响排序，不过滤）
  function isRecommended(p: ProviderPreset): boolean {
    if (!recommendedPlatform) return false;
    return p.recommended_for?.includes(recommendedPlatform) ?? false;
  }

  async function handleSubmit() {
    if (!selectedPreset && !isEditing) {
      toast.error("请选择预设");
      return;
    }
    if (!name.trim()) {
      toast.error("请填写 Provider 名称");
      return;
    }
    if (!baseUrl.trim()) {
      toast.error("请填写 Base URL");
      return;
    }
    // OAuth 类型暂未实现，提示后退回
    if (!isEditing && isOauth) {
      toast.info("OAuth 登录流程将在 v2.1 实现", {
        description: "暂时可手动获取 token 后选「自定义」配置",
      });
      return;
    }
    // 新建模式：必填 API Key；编辑模式：可留空保留原值
    if (!isEditing && !apiKey.trim()) {
      toast.error("请填写 API Key");
      return;
    }
    try {
      if (isEditing) {
        await updateProvider.mutateAsync({
          id: editingProfile.id,
          patch: {
            name: name.trim(),
            provider_kind:
              selectedPreset?.provider_kind ?? editingProfile.provider_kind,
            protocol:
              selectedPreset?.protocol ?? editingProfile.protocol,
            base_url: baseUrl.trim(),
            default_model:
              selectedPreset?.default_model ?? editingProfile.default_model,
            enabled: editingProfile.enabled,
          },
        });
        if (apiKey.trim()) {
          await setCredential.mutateAsync({
            profileId: editingProfile.id,
            apiKey: apiKey.trim(),
          });
        }
        toast.success(`已更新「${name}」`);
      } else {
        const created = await createProvider.mutateAsync({
          name: name.trim(),
          provider_kind: selectedPreset!.provider_kind,
          protocol: selectedPreset!.protocol,
          base_url: baseUrl.trim(),
          default_model: selectedPreset!.default_model ?? null,
          api_key: apiKey.trim(),
        });
        // 自动分配给当前 Agent
        if (autoAssignToAgent && created?.id) {
          try {
            await assignChannel.mutateAsync({
              agentId: autoAssignToAgent,
              profileId: created.id,
            });
            toast.success(`已创建并分配「${name}」`);
          } catch {
            toast.success(`已创建「${name}」（自动分配失败，请手动选择）`);
          }
        } else {
          toast.success(`已创建模型渠道「${name}」`);
        }
      }
      onClose();
    } catch (e) {
      console.error(e);
    }
  }

  return (
    <div className="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-6 animate-fadein">
      <div className="w-full max-w-3xl max-h-[90vh] bg-bg rounded-xl shadow-2xl border border-border flex flex-col overflow-hidden">
        {/* Header */}
        <header className="flex items-center justify-between px-5 py-3.5 border-b border-border">
          <h3 className="text-[14px] font-semibold tracking-tight flex items-center gap-2">
            {isEditing ? (
              <Save className="w-4 h-4 text-accent" />
            ) : (
              <Plus className="w-4 h-4 text-accent" />
            )}
            {isEditing ? `编辑「${editingProfile.name}」` : "新增模型渠道"}
          </h3>
          <button
            onClick={onClose}
            className="text-text-muted hover:text-text"
          >
            <X className="w-4 h-4" />
          </button>
        </header>

        <div className="flex-1 overflow-auto p-5 space-y-5">
          {/* Provider Preset 网格（仅新建模式显示完整网格） */}
          {!isEditing && (
            <section>
              <div className="text-[12px] font-medium text-text-dim mb-2">
                Provider Preset
              </div>
              <div className="space-y-3">
                {groups.map((group) => (
                  <div key={group.category}>
                    <div className="text-[10.5px] uppercase tracking-wider text-text-muted mb-1.5 font-mono">
                      {group.label}
                    </div>
                    <div className="flex flex-wrap gap-1.5">
                      {group.items
                        .slice()
                        .sort(
                          (a, b) =>
                            Number(isRecommended(b)) -
                            Number(isRecommended(a)),
                        )
                        .map((p) => {
                          const active = selectedPreset?.id === p.id;
                          const rec = isRecommended(p);
                          const oauth = p.auth_method === "oauth";
                          return (
                            <button
                              key={p.id}
                              onClick={() => setSelectedPreset(p)}
                              className={cn(
                                "relative flex items-center gap-1.5 px-2.5 py-1.5 rounded-md border text-[11.5px] font-medium transition-all",
                                active
                                  ? "border-transparent text-white shadow-sm"
                                  : "border-border bg-bg-elev hover:border-text-muted text-text-dim",
                              )}
                              style={active ? { background: p.color } : undefined}
                              title={p.note}
                            >
                              <BrandIcon
                                preset={p}
                                size={16}
                                rounded="sm"
                                inverted={active}
                              />
                              <span>{p.name}</span>
                              {oauth && (
                                <KeyRound
                                  className="w-2.5 h-2.5 opacity-70"
                                  aria-label="OAuth"
                                />
                              )}
                              {rec && !active && (
                                <span
                                  className="absolute -top-1 -right-1 w-3.5 h-3.5 rounded-full bg-accent text-white text-[8px] flex items-center justify-center"
                                  title="当前 Agent 推荐"
                                >
                                  ★
                                </span>
                              )}
                            </button>
                          );
                        })}
                    </div>
                  </div>
                ))}
              </div>
              <div className="text-[10.5px] text-text-muted mt-2 flex items-center gap-1.5">
                <span>💡</span>
                <span>仅需填 API Key，Base URL 已自动填充。OAuth 类型见 v2.1</span>
              </div>
            </section>
          )}

          {/* 编辑模式：用紧凑信息块替代预设网格 */}
          {isEditing && selectedPreset && (
            <section className="flex items-center gap-3 px-3 py-2 rounded-md bg-bg-elev border border-border-soft">
              <BrandIcon preset={selectedPreset} size={32} />
              <div className="flex-1 min-w-0">
                <div className="text-[12px] font-medium">
                  {selectedPreset.name}
                </div>
                <div className="text-[10.5px] text-text-muted">
                  {selectedPreset.note}
                </div>
              </div>
              <span className="text-[10.5px] text-text-muted">
                匹配预设
              </span>
            </section>
          )}
          {isEditing && !selectedPreset && (
            <section className="px-3 py-2 rounded-md bg-bg-elev border border-border-soft text-[11.5px] text-text-muted">
              当前渠道无匹配预设（自定义 base URL）
            </section>
          )}

          {/* 图标预览（仅新建模式） */}
          {!isEditing && selectedPreset && (
            <div className="flex justify-center py-2">
              <BrandIcon preset={selectedPreset} size={64} rounded="lg" />
            </div>
          )}

          {/* 表单 */}
          <section className="grid grid-cols-2 gap-3">
            <Field label="Provider 名称" required>
              <input
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="w-full px-3 py-1.5 text-[12.5px] rounded-md bg-bg-elev border border-border focus:border-accent outline-none"
                placeholder="如：MiniMax cn"
              />
            </Field>
            <Field label="备注（可选）">
              <input
                value={notes}
                onChange={(e) => setNotes(e.target.value)}
                className="w-full px-3 py-1.5 text-[12.5px] rounded-md bg-bg-elev border border-border focus:border-accent outline-none"
                placeholder="如：公司专用账号"
              />
            </Field>
            <div className="col-span-2">
              <Field label="Base URL" required>
                <input
                  value={baseUrl}
                  onChange={(e) => setBaseUrl(e.target.value)}
                  className="w-full px-3 py-1.5 text-[12.5px] rounded-md bg-bg-elev border border-border focus:border-accent outline-none font-mono"
                  placeholder="https://api.example.com"
                />
              </Field>
            </div>
            <div className="col-span-2">
              {isOauth && !isEditing ? (
                <Field label="API Key" hint="该供应商使用 OAuth 认证">
                  <div className="flex items-center gap-2 px-3 py-2 rounded-md border border-amber-500/30 bg-amber-500/5">
                    <Shield className="w-3.5 h-3.5 text-amber-600 dark:text-amber-400 flex-shrink-0" />
                    <div className="flex-1 text-[11.5px] text-amber-700 dark:text-amber-300 leading-snug">
                      OAuth 登录流程将在 v2.1 提供。当前可手动获取 token
                      后选「自定义」预设配置。
                    </div>
                  </div>
                </Field>
              ) : (
                <Field
                  label="API Key"
                  required={!isEditing}
                  hint={
                    isEditing
                      ? "已加密保存。留空保留原值，填入则覆盖"
                      : "加密存入系统 Keychain，本地不留明文"
                  }
                >
                  <input
                    type="password"
                    value={apiKey}
                    onChange={(e) => setApiKey(e.target.value)}
                    className="w-full px-3 py-1.5 text-[12.5px] rounded-md bg-bg-elev border border-border focus:border-accent outline-none font-mono"
                    placeholder={
                      isEditing
                        ? editingProfile.credential_set
                          ? "•••••••• (留空保留)"
                          : "sk-..."
                        : "sk-..."
                    }
                    autoComplete="off"
                  />
                </Field>
              )}
            </div>
          </section>
        </div>

        {/* Footer */}
        <footer className="px-5 py-3 border-t border-border bg-bg-elev/50 flex items-center gap-2">
          {/* 编辑模式：左下角删除按钮 */}
          {isEditing && (
            <button
              onClick={() => setConfirmDeleteOpen(true)}
              disabled={submitting}
              className="flex items-center gap-1 px-2.5 py-1.5 rounded-md text-[12.5px] text-critical hover:bg-critical/10 border border-critical/30 disabled:opacity-50"
            >
              <Trash2 className="w-3.5 h-3.5" />
              删除渠道
            </button>
          )}
          <div className="flex-1" />
          <button
            onClick={onClose}
            className="px-3 py-1.5 rounded-md text-[12.5px] text-text-dim hover:text-text hover:bg-bg-elev2"
          >
            取消
          </button>
          <button
            onClick={handleSubmit}
            disabled={submitting}
            className="flex items-center gap-1.5 px-4 py-1.5 rounded-md bg-accent text-white text-[12.5px] font-medium hover:bg-accent/90 disabled:opacity-50"
          >
            {submitting ? (
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
            ) : isEditing ? (
              <Save className="w-3.5 h-3.5" />
            ) : (
              <Plus className="w-3.5 h-3.5" />
            )}
            {isEditing ? "保存" : "添加"}
          </button>
        </footer>
      </div>

      {/* 删除确认（嵌套 modal） */}
      {confirmDeleteOpen && editingProfile && (
        <ConfirmDialog
          title={`删除渠道「${editingProfile.name}」？`}
          message={
            "此操作不可恢复。\n" +
            "Keychain 中的 API 凭据将一并移除，分配关系也会全部清除。"
          }
          confirmText="删除"
          dangerous
          loading={deleteProvider.isPending}
          onConfirm={doDelete}
          onCancel={() => setConfirmDeleteOpen(false)}
        />
      )}
    </div>
  );
}

function Field({
  label,
  required,
  hint,
  children,
}: {
  label: string;
  required?: boolean;
  hint?: string;
  children: React.ReactNode;
}) {
  return (
    <div>
      <label className="block text-[11.5px] text-text-dim mb-1">
        {label}
        {required && <span className="text-critical ml-0.5">*</span>}
      </label>
      {children}
      {hint && (
        <div className="text-[10.5px] text-text-muted mt-1 leading-snug">
          {hint}
        </div>
      )}
    </div>
  );
}
