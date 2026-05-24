// ClawHeart Desktop v2 — Rust 内核入口
//
// 模块编入：
//   security/* — 安全引擎核心（无外部依赖纯逻辑）
//   proxy/*    — 协议归一化 + 检测器（hudsucker 接入 W5+，feature = proxy_real）
//   agents/*   — Agent 发现 + 平台扫描器
//   sync/*     — 云端同步 worker 骨架
//   storage/*  — schema + queries（feature = storage）

pub mod agents;
#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "desktop")]
pub mod commands;
pub mod error;
pub mod proxy;
pub mod security;
pub mod skills;
pub mod state;
pub mod storage;
pub mod sync;

#[cfg(feature = "desktop")]
use state::AppState;
#[cfg(feature = "desktop")]
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[cfg(feature = "desktop")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

    let state = build_state();

    // 启动 fetch_server（W12）—— 反向代理监听 127.0.0.1:19112
    // 不能放在 setup 闭包里（state 已被 .manage 消费），所以提前 clone
    #[cfg(feature = "fetch_server")]
    let state_for_fetch = state.clone();

    // 启动 hudsucker MITM 代理（W5）—— 正向代理监听 127.0.0.1:19111
    #[cfg(feature = "proxy_real")]
    let state_for_proxy = state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .setup(move |app| {
            #[cfg(desktop)]
            {
                tracing::info!(version = env!("CARGO_PKG_VERSION"), "ClawHeart starting");
                let _ = app;
            }

            #[cfg(feature = "fetch_server")]
            {
                let port = state_for_fetch
                    .fetch_server_port
                    .load(std::sync::atomic::Ordering::Acquire);
                let st = state_for_fetch.clone();
                tauri::async_runtime::spawn(async move {
                    let server = proxy::fetch_server::FetchServer::new(st, port);
                    if let Err(e) = server.run().await {
                        tracing::error!(error = %e, "fetch_server stopped with error");
                    }
                });
                tracing::info!(port, "fetch_server spawn requested");
            }

            #[cfg(feature = "proxy_real")]
            {
                let port = state_for_proxy
                    .proxy_server_port
                    .load(std::sync::atomic::Ordering::Acquire);
                let st = state_for_proxy.clone();
                let ks = st.kill_switch.clone();
                tauri::async_runtime::spawn(async move {
                    let server = proxy::server::ProxyServer::new(port, ks, st);
                    if let Err(e) = server.start().await {
                        tracing::error!(error = %e, "hudsucker proxy_server stopped");
                    }
                });
                tracing::info!(port, "proxy_server (hudsucker) spawn requested");
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // status
            commands::status::get_status,
            commands::status::get_proxy_status,
            // settings
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::set_theme,
            // tools matrix
            commands::tools::list_tools,
            commands::tools::list_recent_events,
            commands::tools::trigger_kill_switch,
            // auth
            commands::auth::login,
            commands::auth::logout,
            commands::auth::refresh_token,
            // proxy
            commands::proxy::proxy_pause,
            commands::proxy::proxy_resume,
            commands::proxy::proxy_get_ca_cert,
            commands::proxy::proxy_install_ca,
            // intercept
            commands::intercept::list_intercept_events,
            commands::intercept::get_intercept_event,
            commands::intercept::list_request_logs,
            commands::intercept::export_request_logs,
            // skills
            commands::skills::list_skills,
            commands::skills::toggle_skill,
            commands::skills::set_skill_safety,
            commands::skills::scan_skill,
            commands::skills::sync_skills,
            commands::skills::discover_local_skills,
            commands::skills::get_local_skill_detail,
            commands::skills::scan_local_skill,
            commands::skills::backup_local_skills,
            commands::skills::list_skill_backups,
            commands::skills::delete_skill_backup,
            // skills SSOT 管理（Phase B）
            commands::skills::get_ssot_config,
            commands::skills::get_skill_backup_dir,
            commands::skills::ensure_ssot,
            commands::skills::move_skill_to_ssot,
            commands::skills::toggle_skill_binding,
            commands::skills::uninstall_skill,
            commands::skills::repair_skill_binding,
            // danger (W13 接 DB)
            commands::danger::list_danger_commands,
            commands::danger::toggle_danger_command,
            commands::danger::sync_danger_commands,
            // budget
            commands::budget::list_budget_rules,
            commands::budget::set_budget_rule,
            commands::budget::get_token_usage,
            // usage (CC-Switch 借鉴)
            commands::usage::get_usage_summary,
            commands::usage::get_usage_trends,
            commands::usage::get_usage_by_provider,
            commands::usage::get_usage_by_model,
            // agents
            commands::agents::list_agents,
            commands::agents::discover_agents_now,
            commands::agents::list_mcp_servers,
            // agent decisions (候选 Agent 决策持久化)
            commands::agent_decisions::confirm_unknown_agent,
            commands::agent_decisions::ignore_unknown_agent,
            commands::agent_decisions::reset_unknown_agent_decision,
            commands::agent_decisions::list_unknown_agent_decisions,
            // agent ↔ channel 分配（N:M）
            commands::assignments::list_agent_channels,
            commands::assignments::list_channel_agents,
            commands::assignments::list_all_assignments,
            commands::assignments::assign_channel,
            commands::assignments::unassign_channel,
            commands::assignments::replace_agent_channels,
            // 从 Agent 配置反向导入渠道
            commands::import_channels::list_importable_platforms,
            commands::import_channels::scan_importable_channels,
            commands::import_channels::import_channels_batch,
            // scan
            commands::scan::get_scan_items,
            commands::scan::start_scan_run,
            commands::scan::list_scan_history,
            commands::scan::get_scan_progress,
            commands::scan::get_scan_run,
            commands::security_rules::list_security_rules,
            commands::security_rules::toggle_security_rule,
            commands::security_rules::set_rule_action,
            commands::security_rules::reset_rule,
            commands::security_rules::reset_rule_kind,
            // advisory
            commands::advisory::list_advisories,
            commands::advisory::acknowledge_advisory,
            commands::advisory::subscribe_feed,
            // kill switch
            commands::killswitch::kill_switch_activate,
            commands::killswitch::kill_switch_reset,
            commands::killswitch::kill_switch_status,
            // agent config one-click overwrite (W7 + W8)
            commands::agent_config::scan_agent_configs,
            commands::agent_config::plan_overwrite,
            commands::agent_config::apply_overwrite,
            commands::agent_config::list_apply_batches,
            commands::agent_config::list_batch_snapshots,
            commands::agent_config::rollback_batch,
            commands::agent_config::rollback_snapshot,
            commands::agent_config::get_apply_real_status,
            commands::agent_config::set_apply_real_enabled,
            // providers (第三方 LLM 中转 API 集中管理)
            commands::providers::list_provider_profiles,
            commands::providers::create_provider_profile,
            commands::providers::update_provider_profile,
            commands::providers::delete_provider_profile,
            commands::providers::set_default_provider_profile,
            commands::providers::set_provider_credential,
            commands::providers::clear_provider_credential,
            commands::providers::test_provider_connection,
            commands::providers::scan_import_candidates,
            commands::providers::scan_unmanaged_agents,
            commands::providers::bulk_import_profiles,
            commands::providers::list_active_routings,
            commands::providers::set_agent_routing,
            // access mode (三种监控模式)
            commands::access_mode::get_access_mode,
            commands::access_mode::get_fetch_server_status,
            commands::access_mode::set_access_mode,
            commands::access_mode::update_proxy_port,
            commands::access_mode::list_protocol_adapters,
            commands::access_mode::toggle_protocol_adapter,
            commands::access_mode::check_ca_status,
            commands::access_mode::install_ca,
            commands::access_mode::uninstall_ca,
            commands::access_mode::generate_sandbox_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(all(feature = "desktop", feature = "storage"))]
fn build_state() -> AppState {
    let db_path = storage::conn::default_path();
    tracing::info!("Opening DB at {:?}", db_path);
    match storage::conn::open(&db_path) {
        Ok(db) => {
            tracing::info!("Database opened successfully");
            match storage::seed::run_if_needed(&db) {
                Ok(true) => tracing::info!("Seeded initial data (30 danger rules + budget + settings)"),
                Ok(false) => tracing::debug!("Seed already applied"),
                Err(e) => tracing::warn!("Seed failed (non-fatal): {}", e),
            }
            AppState::new().with_db(db)
        }
        Err(e) => {
            tracing::error!("Failed to open DB at {:?}: {}", db_path, e);
            AppState::new()
        }
    }
}

#[cfg(all(feature = "desktop", not(feature = "storage")))]
fn build_state() -> AppState {
    AppState::new()
}

#[cfg(feature = "desktop")]
fn init_tracing() {
    let filter = EnvFilter::try_from_env("CLAWHEART_LOG")
        .unwrap_or_else(|_| EnvFilter::new("info,clawheart=debug"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().compact())
        .init();
}
