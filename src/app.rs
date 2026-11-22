use crate::discord::{
    DeletionEvent, DeletionJob, StopSignal, ValidatedUser, fetch_avatar_bytes, run_deletion,
    validate_token,
};
use crate::history::{AUTO_SAVE_INTERVAL, DeletionHistory};
use crate::i18n::I18n;
use crate::pack::{FilteredMessage, GroupType, PackData, filter_pack, parse_pack};
use crate::store::{Settings, Store};
use crate::theme::{self, format_number};
use crate::ui;
use chrono::Local;
use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Dashboard,
    Pack,
    Review,
    Progress,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Success,
    Error,
    Warn,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub time: String,
    pub text: String,
    pub level: LogLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Neutral,
    Ok,
    Err,
}

#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub text: String,
    pub kind: StatusKind,
}

#[derive(Debug, Clone)]
pub struct ReviewChannel {
    pub is_dm: bool,
    pub name: String,
    pub expanded: bool,
    pub message_indices: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct ReviewGroup {
    pub is_dm_group: bool,
    pub name: String,
    pub total_messages: usize,
    pub expanded: bool,
    pub channels: Vec<ReviewChannel>,
}

#[derive(Debug, Clone, Copy)]
pub enum TreeRow {
    Group(usize),
    Channel(usize, usize),
    Message(usize),
}

#[derive(Debug, Clone)]
pub enum ModalState {
    None,
    NoTokenAlert,
    ConfirmDeletion { count: usize },
}

pub enum AppMsg {
    FolderPicked(Option<PathBuf>),
    PackParsed {
        folder: String,
        result: Result<PackData, String>,
    },
    TokenValidated {
        token: String,
        result: Result<ValidatedUser, String>,
    },
    AvatarLoaded(ColorImage),
    HistoryPathPicked(Option<PathBuf>),
    HistoryFilePicked(Option<PathBuf>),
    Deletion(DeletionEvent),
}

pub struct ProgressState {
    pub running: bool,
    pub stopping: bool,
    pub finished: bool,
    pub total: usize,
    pub deleted: usize,
    pub errors: usize,
    pub progress: f32,
    pub status_text: String,
    pub logs: Vec<LogEntry>,
}

impl ProgressState {
    pub fn new(ready_label: &str) -> Self {
        Self {
            running: false,
            stopping: false,
            finished: false,
            total: 0,
            deleted: 0,
            errors: 0,
            progress: 0.0,
            status_text: ready_label.to_string(),
            logs: Vec::new(),
        }
    }
}

pub struct DisvokeApp {
    pub rt: Arc<Runtime>,
    pub http: reqwest::Client,
    pub store: Store,
    pub settings: Settings,
    pub draft_settings: Settings,
    pub i18n: I18n,
    pub page: Page,

    pub logo_texture: Option<TextureHandle>,
    pub avatar_texture: Option<TextureHandle>,
    pub validated_user: Option<ValidatedUser>,

    pub pack_data: Option<PackData>,
    pub pack_folder: Option<String>,
    pub pack_loading: bool,
    pub pack_error: Option<String>,
    pub filter_servers: bool,
    pub filter_dms: bool,
    pub filter_search: String,

    pub filtered_messages: Vec<FilteredMessage>,
    pub selected_ids: HashSet<String>,
    pub review_groups: Vec<ReviewGroup>,
    pub visible_rows: Vec<TreeRow>,

    pub history: DeletionHistory,
    pub progress: ProgressState,
    pub stop_signal: Option<Arc<StopSignal>>,

    pub show_token: bool,
    pub token_checking: bool,
    pub token_result: Option<StatusMessage>,
    pub history_result: Option<StatusMessage>,
    pub save_result: Option<(StatusMessage, Instant)>,
    pub modal: ModalState,

    pub tx: mpsc::UnboundedSender<AppMsg>,
    rx: mpsc::UnboundedReceiver<AppMsg>,
}

impl DisvokeApp {
    pub fn new(cc: &eframe::CreationContext<'_>, rt: Arc<Runtime>, logo_image: Option<ColorImage>) -> Self {
        theme::configure_fonts(&cc.egui_ctx);
        theme::apply_theme(&cc.egui_ctx);

        let store = Store::new();
        let settings = store.load();
        let i18n = I18n::new(&settings.lang);
        let ready_text = i18n.t("progress.ready").to_string();

        let history = DeletionHistory {
            history_path: settings.history_path.clone(),
            ..Default::default()
        };

        let logo_texture = logo_image.map(|img| {
            cc.egui_ctx
                .load_texture("disvoke_logo", img, TextureOptions::LINEAR)
        });

        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            rt,
            http: reqwest::Client::new(),
            store,
            draft_settings: settings.clone(),
            settings,
            i18n,
            page: Page::Dashboard,
            logo_texture,
            avatar_texture: None,
            validated_user: None,
            pack_data: None,
            pack_folder: None,
            pack_loading: false,
            pack_error: None,
            filter_servers: true,
            filter_dms: true,
            filter_search: String::new(),
            filtered_messages: Vec::new(),
            selected_ids: HashSet::new(),
            review_groups: Vec::new(),
            visible_rows: Vec::new(),
            history,
            progress: ProgressState::new(&ready_text),
            stop_signal: None,
            show_token: false,
            token_checking: false,
            token_result: None,
            history_result: None,
            save_result: None,
            modal: ModalState::None,
            tx,
            rx,
        }
    }

    pub fn add_log(&mut self, text: impl Into<String>, level: LogLevel) {
        let time = Local::now().format("%H:%M:%S").to_string();
        self.progress.logs.push(LogEntry {
            time,
            text: text.into(),
            level,
        });
    }

    pub fn pick_pack_folder(&self, ctx: &Context) {
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        self.rt.spawn(async move {
            let picked = rfd::AsyncFileDialog::new()
                .set_title("Select extracted Discord data pack folder")
                .pick_folder()
                .await
                .map(|h| h.path().to_path_buf());
            let _ = tx.send(AppMsg::FolderPicked(picked));
            ctx.request_repaint();
        });
    }

    pub fn load_pack_path(&mut self, path: PathBuf, ctx: &Context) {
        let folder = path.display().to_string();
        self.pack_loading = true;
        self.pack_error = None;
        self.pack_folder = Some(folder.clone());

        let tx = self.tx.clone();
        let ctx = ctx.clone();
        self.rt.spawn(async move {
            let result = tokio::task::spawn_blocking(move || parse_pack(&path))
                .await
                .unwrap_or_else(|e| Err(e.to_string()));
            let _ = tx.send(AppMsg::PackParsed { folder, result });
            ctx.request_repaint();
        });
    }

    pub fn apply_filter_and_review(&mut self) {
        let Some(pack) = &self.pack_data else {
            return;
        };
        self.filtered_messages = filter_pack(
            pack,
            self.filter_servers,
            self.filter_dms,
            &self.filter_search,
        );
        self.selected_ids = self
            .filtered_messages
            .iter()
            .filter(|m| !self.history.deleted_ids.contains(&m.msg.id))
            .map(|m| m.msg.id.clone())
            .collect();

        self.rebuild_review_tree();
        self.page = Page::Review;

        let skipped = self.filtered_messages.len().saturating_sub(self.selected_ids.len());
        if skipped > 0 {
            self.add_log(
                format!(
                    "History: {} message(s) already deleted in a previous session — deselected automatically.",
                    format_number(skipped)
                ),
                LogLevel::Info,
            );
        }
    }

    pub fn rebuild_review_tree(&mut self) {
        self.review_groups.clear();
        self.visible_rows.clear();
        if self.filtered_messages.is_empty() {
            return;
        }

        let mut server_order: Vec<String> = Vec::new();
        let mut server_channels: HashMap<String, Vec<String>> = HashMap::new();
        let mut channel_buckets: HashMap<(String, String), Vec<usize>> = HashMap::new();
        let mut dm_order: Vec<String> = Vec::new();
        let mut dm_buckets: HashMap<String, Vec<usize>> = HashMap::new();

        for (idx, item) in self.filtered_messages.iter().enumerate() {
            match item.group_type {
                GroupType::Server => {
                    let srv = &item.group_name;
                    let ch = &item.msg.channel_name;
                    if !server_channels.contains_key(srv) {
                        server_order.push(srv.clone());
                        server_channels.insert(srv.clone(), Vec::new());
                    }
                    let key = (srv.clone(), ch.clone());
                    let entry = channel_buckets.entry(key).or_insert_with(|| {
                        server_channels.get_mut(srv).unwrap().push(ch.clone());
                        Vec::new()
                    });
                    entry.push(idx);
                }
                GroupType::Dm => {
                    let dm = &item.group_name;
                    let entry = dm_buckets.entry(dm.clone()).or_insert_with(|| {
                        dm_order.push(dm.clone());
                        Vec::new()
                    });
                    entry.push(idx);
                }
            }
        }

        for srv_name in server_order {
            let mut channels = Vec::new();
            let mut total = 0usize;
            if let Some(ch_names) = server_channels.remove(&srv_name) {
                for ch_name in ch_names {
                    let indices = channel_buckets
                        .remove(&(srv_name.clone(), ch_name.clone()))
                        .unwrap_or_default();
                    total += indices.len();
                    channels.push(ReviewChannel {
                        is_dm: false,
                        name: ch_name,
                        expanded: false,
                        message_indices: indices,
                    });
                }
            }
            self.review_groups.push(ReviewGroup {
                is_dm_group: false,
                name: srv_name,
                total_messages: total,
                expanded: true,
                channels,
            });
        }

        if !dm_order.is_empty() {
            let mut channels = Vec::new();
            let mut total = 0usize;
            for dm_name in dm_order {
                let indices = dm_buckets.remove(&dm_name).unwrap_or_default();
                total += indices.len();
                channels.push(ReviewChannel {
                    is_dm: true,
                    name: dm_name,
                    expanded: false,
                    message_indices: indices,
                });
            }
            self.review_groups.push(ReviewGroup {
                is_dm_group: true,
                name: String::new(),
                total_messages: total,
                expanded: true,
                channels,
            });
        }

        self.rebuild_visible_rows();
    }

    pub fn rebuild_visible_rows(&mut self) {
        self.visible_rows.clear();
        for (g_idx, grp) in self.review_groups.iter().enumerate() {
            self.visible_rows.push(TreeRow::Group(g_idx));
            if grp.expanded {
                for (c_idx, ch) in grp.channels.iter().enumerate() {
                    self.visible_rows.push(TreeRow::Channel(g_idx, c_idx));
                    if ch.expanded {
                        for &msg_idx in &ch.message_indices {
                            self.visible_rows.push(TreeRow::Message(msg_idx));
                        }
                    }
                }
            }
        }
    }

    pub fn select_all_filtered(&mut self) {
        for item in &self.filtered_messages {
            if !self.history.deleted_ids.contains(&item.msg.id) {
                self.selected_ids.insert(item.msg.id.clone());
            }
        }
    }

    pub fn deselect_all(&mut self) {
        self.selected_ids.clear();
    }

    pub fn verify_token(&mut self, ctx: &Context) {
        let token = self.draft_settings.token.trim().to_string();
        if token.is_empty() {
            self.token_result = Some(StatusMessage {
                text: self.i18n.t("settings.noToken").to_string(),
                kind: StatusKind::Err,
            });
            return;
        }
        self.token_checking = true;
        self.token_result = Some(StatusMessage {
            text: self.i18n.t("settings.tokenChecking").to_string(),
            kind: StatusKind::Neutral,
        });

        let client = self.http.clone();
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        self.rt.spawn(async move {
            let result = validate_token(&client, &token).await;
            let _ = tx.send(AppMsg::TokenValidated { token, result });
            ctx.request_repaint();
        });
    }

    pub fn pick_history_save_path(&self, ctx: &Context) {
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        self.rt.spawn(async move {
            let picked = rfd::AsyncFileDialog::new()
                .set_title("Choose history file location")
                .set_file_name("disvoke-history.json")
                .add_filter("JSON", &["json"])
                .save_file()
                .await
                .map(|h| h.path().to_path_buf());
            let _ = tx.send(AppMsg::HistoryPathPicked(picked));
            ctx.request_repaint();
        });
    }

    pub fn pick_history_open_file(&self, ctx: &Context) {
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        self.rt.spawn(async move {
            let picked = rfd::AsyncFileDialog::new()
                .set_title("Open history file")
                .add_filter("JSON", &["json"])
                .pick_file()
                .await
                .map(|h| h.path().to_path_buf());
            let _ = tx.send(AppMsg::HistoryFilePicked(picked));
            ctx.request_repaint();
        });
    }

    pub fn clear_history(&mut self) {
        self.history.deleted_ids.clear();
        self.history.auto_save_counter = 0;
        let msg = self.i18n.t("settings.historyCleared");
        self.history_result = Some(StatusMessage {
            text: if msg.is_empty() {
                "History cleared from memory.".to_string()
            } else {
                msg.to_string()
            },
            kind: StatusKind::Ok,
        });
    }

    pub fn save_settings(&mut self) {
        self.draft_settings.token = self.draft_settings.token.trim().to_string();
        self.draft_settings.history_path = self.draft_settings.history_path.trim().to_string();
        self.draft_settings.delay = self.draft_settings.delay.clamp(1, 30);
        self.draft_settings.batch_mode.min_batch = self.draft_settings.batch_mode.min_batch.clamp(1, 9999);
        self.draft_settings.batch_mode.max_batch = self.draft_settings.batch_mode.max_batch.clamp(1, 9999);
        self.draft_settings.batch_mode.min_break_min = self.draft_settings.batch_mode.min_break_min.clamp(1, 9999);
        self.draft_settings.batch_mode.max_break_min = self.draft_settings.batch_mode.max_break_min.clamp(1, 9999);

        let lang_changed = self.draft_settings.lang != self.settings.lang;
        self.settings = self.draft_settings.clone();
        self.history.history_path = self.settings.history_path.clone();
        self.store.save(&self.settings);

        if lang_changed {
            self.i18n.set_lang(&self.settings.lang);
            if !self.progress.running && !self.progress.finished {
                self.progress.status_text = self.i18n.t("progress.ready").to_string();
            }
        }

        self.save_result = Some((
            StatusMessage {
                text: self.i18n.t("settings.saved").to_string(),
                kind: StatusKind::Ok,
            },
            Instant::now(),
        ));
    }

    pub fn request_start_deletion(&mut self) {
        let count = self.selected_ids.len();
        if count == 0 || self.progress.running {
            return;
        }
        if self.settings.token.trim().is_empty() {
            self.modal = ModalState::NoTokenAlert;
            return;
        }
        self.modal = ModalState::ConfirmDeletion { count };
    }

    pub fn start_deletion(&mut self, ctx: &Context) {
        let messages: Vec<_> = self
            .filtered_messages
            .iter()
            .filter(|m| self.selected_ids.contains(&m.msg.id))
            .map(|m| m.msg.clone())
            .collect();
        if messages.is_empty() {
            return;
        }

        let total = messages.len();
        self.page = Page::Progress;
        self.history.auto_save_counter = 0;

        self.progress.running = true;
        self.progress.stopping = false;
        self.progress.finished = false;
        self.progress.total = total;
        self.progress.deleted = 0;
        self.progress.errors = 0;
        self.progress.progress = 0.0;
        self.progress.logs.clear();
        self.progress.status_text = self.i18n.fmt(
            "progress.deleting",
            &[("current", "0"), ("total", &total.to_string())],
        );

        let start_log = self.i18n.fmt(
            "progress.logStart",
            &[
                ("count", &total.to_string()),
                ("delay", &self.settings.delay.to_string()),
            ],
        );
        self.add_log(start_log, LogLevel::Info);

        let bm = &self.settings.batch_mode;
        if bm.enabled {
            let batch_log = self.i18n.fmt(
                "progress.logBatchMode",
                &[
                    ("min", &bm.min_batch.to_string()),
                    ("max", &bm.max_batch.to_string()),
                    ("breakMin", &bm.min_break_min.to_string()),
                    ("breakMax", &bm.max_break_min.to_string()),
                ],
            );
            self.add_log(batch_log, LogLevel::Info);
        }

        if !self.history.history_path.trim().is_empty() {
            let path = self.history.history_path.clone();
            self.add_log(
                format!("History file: {path}  (autosave every {AUTO_SAVE_INTERVAL} messages)"),
                LogLevel::Info,
            );
        } else {
            self.add_log(
                "No history file configured — deleted IDs stored in memory only.",
                LogLevel::Warn,
            );
        }

        let stop = Arc::new(StopSignal::new());
        self.stop_signal = Some(stop.clone());

        let (del_tx, mut del_rx) = mpsc::unbounded_channel();
        let app_tx = self.tx.clone();
        let repaint_ctx = ctx.clone();
        let forward_ctx = ctx.clone();

        self.rt.spawn(async move {
            while let Some(ev) = del_rx.recv().await {
                let _ = app_tx.send(AppMsg::Deletion(ev));
                forward_ctx.request_repaint();
            }
        });

        let client = self.http.clone();
        let token = self.settings.token.clone();
        let delay = self.settings.delay;
        let batch_mode = if self.settings.batch_mode.enabled {
            Some(self.settings.batch_mode.clone())
        } else {
            None
        };
        let history_snapshot = self.history.deleted_ids.clone();

        self.rt.spawn(async move {
            run_deletion(
                DeletionJob {
                    client,
                    messages,
                    token,
                    delay_secs: delay,
                    batch_mode,
                    history_set: history_snapshot,
                    stop,
                    tx: del_tx,
                },
                move || repaint_ctx.request_repaint(),
            )
            .await;
        });
    }

    pub fn stop_deletion(&mut self) {
        if let Some(sig) = &self.stop_signal {
            sig.request_stop();
        }
        if !self.progress.stopping {
            self.progress.stopping = true;
            let msg = self.i18n.t("progress.logStopping").to_string();
            self.add_log(msg, LogLevel::Warn);
        }
    }

    fn process_messages(&mut self, ctx: &Context) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AppMsg::FolderPicked(Some(path)) => {
                    self.load_pack_path(path, ctx);
                }
                AppMsg::FolderPicked(None) => {}
                AppMsg::PackParsed { folder, result } => {
                    self.pack_loading = false;
                    match result {
                        Ok(data) => {
                            self.pack_folder = Some(folder);
                            self.pack_error = None;
                            self.pack_data = Some(data);
                        }
                        Err(err) => {
                            self.pack_error = Some(err);
                            self.pack_data = None;
                        }
                    }
                }
                AppMsg::TokenValidated { token, result } => {
                    self.token_checking = false;
                    match result {
                        Ok(user) => {
                            let ok_text = self.i18n.fmt(
                                "settings.tokenOk",
                                &[("username", &user.username), ("id", &user.id)],
                            );
                            self.token_result = Some(StatusMessage {
                                text: ok_text,
                                kind: StatusKind::Ok,
                            });
                            self.settings.token = token;
                            self.avatar_texture = None;
                            if let Some(avatar_hash) = user.avatar.clone() {
                                let client = self.http.clone();
                                let uid = user.id.clone();
                                let tx = self.tx.clone();
                                let ctx = ctx.clone();
                                self.rt.spawn(async move {
                                    if let Some(bytes) =
                                        fetch_avatar_bytes(&client, &uid, &avatar_hash).await
                                        && let Some(img) = decode_circle_avatar(&bytes)
                                    {
                                        let _ = tx.send(AppMsg::AvatarLoaded(img));
                                        ctx.request_repaint();
                                    }
                                });
                            }
                            self.validated_user = Some(user);
                        }
                        Err(err) => {
                            let err_text = self.i18n.fmt("settings.tokenErr", &[("error", &err)]);
                            self.token_result = Some(StatusMessage {
                                text: err_text,
                                kind: StatusKind::Err,
                            });
                        }
                    }
                }
                AppMsg::AvatarLoaded(img) => {
                    self.avatar_texture =
                        Some(ctx.load_texture("user_avatar", img, TextureOptions::LINEAR));
                }
                AppMsg::HistoryPathPicked(Some(path)) => {
                    let s = path.display().to_string();
                    self.history.history_path = s.clone();
                    self.settings.history_path = s.clone();
                    self.draft_settings.history_path = s;
                }
                AppMsg::HistoryPathPicked(None) => {}
                AppMsg::HistoryFilePicked(Some(path)) => {
                    match DeletionHistory::load_from_path(&path) {
                        Some(ids) => {
                            let count = ids.len();
                            self.history.deleted_ids = ids.into_iter().collect();
                            let s = path.display().to_string();
                            self.history.history_path = s.clone();
                            self.settings.history_path = s.clone();
                            self.draft_settings.history_path = s;

                            for id in &self.history.deleted_ids {
                                self.selected_ids.remove(id);
                            }

                            let text = self.i18n.fmt(
                                "settings.historyLoaded",
                                &[("count", &count.to_string())],
                            );
                            self.history_result = Some(StatusMessage {
                                text: if text.is_empty() {
                                    format!("Loaded {count} deleted IDs.")
                                } else {
                                    text
                                },
                                kind: StatusKind::Ok,
                            });
                        }
                        None => {
                            let text = self.i18n.t("settings.historyLoadErr");
                            self.history_result = Some(StatusMessage {
                                text: if text.is_empty() {
                                    "Invalid history file.".to_string()
                                } else {
                                    text.to_string()
                                },
                                kind: StatusKind::Err,
                            });
                        }
                    }
                }
                AppMsg::HistoryFilePicked(None) => {}
                AppMsg::Deletion(ev) => self.handle_deletion_event(ev),
            }
        }
    }

    fn handle_deletion_event(&mut self, ev: DeletionEvent) {
        let total = self.progress.total.max(1);
        match ev {
            DeletionEvent::BatchStart {
                batch_index,
                batch_total,
                batch_size,
            } => {
                let text = self.i18n.fmt(
                    "progress.logBatchStart",
                    &[
                        ("index", &batch_index.to_string()),
                        ("total", &batch_total.to_string()),
                        ("size", &batch_size.to_string()),
                    ],
                );
                self.add_log(text, LogLevel::Info);
            }
            DeletionEvent::Success {
                msg_id,
                deleted,
                errors,
            } => {
                self.history.deleted_ids.insert(msg_id.clone());
                self.selected_ids.remove(&msg_id);
                self.history.auto_save_counter += 1;

                self.progress.deleted = deleted;
                self.progress.errors = errors;
                let done_count = deleted + errors;
                self.progress.progress = (done_count as f32 / total as f32).clamp(0.0, 1.0);
                self.progress.status_text = self.i18n.fmt(
                    "progress.deleting",
                    &[
                        ("current", &done_count.to_string()),
                        ("total", &self.progress.total.to_string()),
                    ],
                );

                let log = self.i18n.fmt(
                    "progress.logSuccess",
                    &[("id", &msg_id), ("count", &deleted.to_string())],
                );
                self.add_log(log, LogLevel::Success);

                if self.history.auto_save_counter.is_multiple_of(AUTO_SAVE_INTERVAL)
                    && self.history.save_to_disk()
                {
                    let total_ids = self.history.deleted_ids.len();
                    self.add_log(
                        format!("History autosaved ({total_ids} IDs total)."),
                        LogLevel::Info,
                    );
                }
            }
            DeletionEvent::Error {
                msg_id,
                deleted,
                errors,
                status,
                reason,
            } => {
                self.progress.deleted = deleted;
                self.progress.errors = errors;
                let done_count = deleted + errors;
                self.progress.progress = (done_count as f32 / total as f32).clamp(0.0, 1.0);

                let detail = if !status.is_empty() {
                    if !reason.is_empty() {
                        format!(" [HTTP {status}: {reason}]")
                    } else {
                        format!(" [HTTP {status}]")
                    }
                } else {
                    String::new()
                };
                let log = self
                    .i18n
                    .fmt("progress.logError", &[("id", &msg_id), ("detail", &detail)]);
                self.add_log(log, LogLevel::Error);
            }
            DeletionEvent::RateLimit { retry_after_ms } => {
                let secs = format!("{:.1}", retry_after_ms as f64 / 1000.0);
                let log = self
                    .i18n
                    .fmt("progress.logRateLimit", &[("seconds", &secs)]);
                self.add_log(log, LogLevel::Warn);
            }
            DeletionEvent::BatchBreak {
                break_ms,
                batch_index,
                batch_total,
            } => {
                let mins = ((break_ms as f64 / 60_000.0).round() as u64).to_string();
                let log = self.i18n.fmt(
                    "progress.logBatchBreak",
                    &[
                        ("index", &batch_index.to_string()),
                        ("total", &batch_total.to_string()),
                        ("mins", &mins),
                    ],
                );
                self.add_log(log, LogLevel::Warn);
                self.progress.status_text = self
                    .i18n
                    .fmt("progress.batchBreakStatus", &[("mins", &mins)]);
            }
            DeletionEvent::BatchBreakTick { remaining_ms } => {
                let mins = ((remaining_ms as f64 / 60_000.0).ceil() as u64).to_string();
                self.progress.status_text = self
                    .i18n
                    .fmt("progress.batchBreakStatus", &[("mins", &mins)]);
            }
            DeletionEvent::Stopped { deleted, errors } => {
                let saved = self.history.save_to_disk();
                self.progress.running = false;
                self.progress.stopping = false;
                self.progress.finished = true;
                self.progress.deleted = deleted;
                self.progress.errors = errors;
                self.stop_signal = None;

                let status = self.i18n.fmt(
                    "progress.stopped",
                    &[
                        ("deleted", &deleted.to_string()),
                        ("errors", &errors.to_string()),
                    ],
                );
                self.progress.status_text = if status.is_empty() {
                    format!("Stopped. Deleted: {deleted}, Errors: {errors}")
                } else {
                    status
                };

                let suffix = if saved { " History saved." } else { "" };
                self.add_log(
                    format!("Stopped by user. Deleted: {deleted}, Errors: {errors}.{suffix}"),
                    LogLevel::Warn,
                );
            }
            DeletionEvent::Done { deleted, errors } => {
                self.history.save_to_disk();
                self.progress.running = false;
                self.progress.stopping = false;
                self.progress.finished = true;
                self.progress.deleted = deleted;
                self.progress.errors = errors;
                self.progress.progress = 1.0;
                self.stop_signal = None;

                self.progress.status_text = self.i18n.fmt(
                    "progress.done",
                    &[
                        ("deleted", &deleted.to_string()),
                        ("errors", &errors.to_string()),
                    ],
                );
                let log = self.i18n.fmt(
                    "progress.logDone",
                    &[
                        ("deleted", &deleted.to_string()),
                        ("errors", &errors.to_string()),
                    ],
                );
                self.add_log(log, LogLevel::Success);
            }
        }
    }

    fn handle_dropped_files(&mut self, ctx: &Context) {
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect()
        });
        if let Some(dir) = dropped.into_iter().find(|p| p.is_dir()) {
            self.page = Page::Pack;
            self.load_pack_path(dir, ctx);
        }
    }
}

impl eframe::App for DisvokeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.process_messages(ctx);
        self.handle_dropped_files(ctx);

        if let Some((_, shown_at)) = &self.save_result {
            let elapsed = shown_at.elapsed();
            if elapsed >= Duration::from_secs(2) {
                self.save_result = None;
            } else {
                ctx.request_repaint_after(Duration::from_secs(2) - elapsed);
            }
        }

        ui::render_app(self, ctx);
    }
}

fn decode_circle_avatar(bytes: &[u8]) -> Option<ColorImage> {
    let img = image::load_from_memory(bytes)
        .ok()?
        .resize_exact(64, 64, image::imageops::FilterType::Triangle)
        .to_rgba8();
    let (w, h) = img.dimensions();
    let mut pixels = img.into_raw();
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let radius = cx.min(cy) - 0.5;

    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            let idx = ((y * w + x) * 4) as usize;
            if dist > radius + 0.5 {
                pixels[idx + 3] = 0;
            } else if dist > radius - 0.5 {
                let alpha = ((radius + 0.5 - dist).clamp(0.0, 1.0) * pixels[idx + 3] as f32) as u8;
                pixels[idx + 3] = alpha;
            }
        }
    }

    Some(ColorImage::from_rgba_unmultiplied(
        [w as usize, h as usize],
        &pixels,
    ))
}
