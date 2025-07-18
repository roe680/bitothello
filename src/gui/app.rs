use crate::board::BitBoard;
use crate::gui::game_view::GameView;
use crate::gui::plot_viewer::PlotViewer;
use crate::player::{Player, PlayerType};
use crate::stats::{GameResult, GameStats};
use eframe::egui;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    Japanese,
    English,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    Menu,
    Playing,
    GameOver,
    ViewingStats,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayerTypeSelection {
    Human,
    AI1,
    AI3,
    AI5,
    AI7,
    AI9,
    AI11,
    AI13,
    Custom,
}

impl PlayerTypeSelection {
    fn to_string(&self) -> &'static str {
        match self {
            Self::Human => "人間",
            Self::AI1 => "AI レベル1 (初級)",
            Self::AI3 => "AI レベル3 (中級)",
            Self::AI5 => "AI レベル5 (上級)",
            Self::AI7 => "AI レベル7 (超上級)",
            Self::AI9 => "AI レベル9 (超超上級)",
            Self::AI11 => "AI レベル11 (超超超上級)",
            Self::AI13 => "AI レベル13 (超超超超上級)",
            Self::Custom => "カスタム",
        }
    }

    fn to_player_type(&self, custom_depth: usize) -> PlayerType {
        match self {
            Self::Human => PlayerType::Human,
            Self::AI1 => PlayerType::AI {
                level: 1,
                tt: RefCell::new(HashMap::default()),
            },
            Self::AI3 => PlayerType::AI {
                level: 3,
                tt: RefCell::new(HashMap::default()),
            },
            Self::AI5 => PlayerType::AI {
                level: 5,
                tt: RefCell::new(HashMap::default()),
            },
            Self::AI7 => PlayerType::AI {
                level: 7,
                tt: RefCell::new(HashMap::default()),
            },
            Self::AI9 => PlayerType::AI {
                level: 9,
                tt: RefCell::new(HashMap::default()),
            },
            Self::AI11 => PlayerType::AI {
                level: 11,
                tt: RefCell::new(HashMap::default()),
            },
            Self::AI13 => PlayerType::AI {
                level: 13,
                tt: RefCell::new(HashMap::default()),
            },
            Self::Custom => PlayerType::AI {
                level: custom_depth,
                tt: RefCell::new(HashMap::default()),
            },
        }
    }
}

pub struct OthelloApp {
    state: GameState,
    language: Language,

    // ゲーム設定
    black_player_type: PlayerTypeSelection,
    white_player_type: PlayerTypeSelection,
    custom_depth: usize,

    // ゲーム状態
    board: BitBoard,
    current_player: Player,
    black_player: Option<PlayerType>,
    white_player: Option<PlayerType>,
    pass_count: usize,

    // 統計
    game_stats: GameStats,
    thinking_time: Duration,

    // UI状態
    selected_position: Option<(usize, usize)>,
    status_message: String,

    // AI思考の非同期処理
    ai_thinking: bool,
    ai_move_receiver: Option<mpsc::Receiver<(bool, Option<(usize, usize)>, Option<i32>)>>,

    // ゲームビューアとプロットビューア
    game_view: GameView,
    plot_viewer: PlotViewer,

    // グラフ用データ保存
    stored_game_stats: Option<GameStats>,
    stored_game_result: Option<GameResult>,

    // ウィンドウ管理
    show_stats_window: bool,
    show_plot_window: bool,
}

impl Default for OthelloApp {
    fn default() -> Self {
        Self {
            state: GameState::Menu,
            language: Language::Japanese,
            black_player_type: PlayerTypeSelection::Human,
            white_player_type: PlayerTypeSelection::AI3,
            custom_depth: 5,
            board: BitBoard::new(),
            current_player: Player::Black,
            black_player: None,
            white_player: None,
            pass_count: 0,
            game_stats: GameStats::new(),
            thinking_time: Duration::new(0, 0),
            selected_position: None,
            status_message: String::new(),
            ai_thinking: false,
            ai_move_receiver: None,
            game_view: GameView::new(),
            plot_viewer: PlotViewer::new(),
            stored_game_stats: None,
            stored_game_result: None,
            show_stats_window: false,
            show_plot_window: false,
        }
    }
}

impl OthelloApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // eframeのデフォルトフォントはUnicodeをサポートしているため
        // 日本語も表示可能
        Self::default()
    }

    fn t(language: Language, key: &str) -> String {
        match (language, key) {
            // Game titles
            (Language::Japanese, "title") => "ビット オセロ".to_string(),
            (Language::English, "title") => "Bit Othello".to_string(),

            // Player types
            (Language::Japanese, "human") => "人間".to_string(),
            (Language::English, "human") => "Human".to_string(),
            (Language::Japanese, "ai_level1") => "AI レベル1 (初級)".to_string(),
            (Language::English, "ai_level1") => "AI Level 1 (Beginner)".to_string(),
            (Language::Japanese, "ai_level3") => "AI レベル3 (中級)".to_string(),
            (Language::English, "ai_level3") => "AI Level 3 (Intermediate)".to_string(),
            (Language::Japanese, "ai_level5") => "AI レベル5 (上級)".to_string(),
            (Language::English, "ai_level5") => "AI Level 5 (Advanced)".to_string(),
            (Language::Japanese, "ai_level7") => "AI レベル7 (超上級)".to_string(),
            (Language::English, "ai_level7") => "AI Level 7 (Expert)".to_string(),
            (Language::Japanese, "ai_level9") => "AI レベル9 (超超上級)".to_string(),
            (Language::English, "ai_level9") => "AI Level 9 (Master)".to_string(),
            (Language::Japanese, "ai_level11") => "AI レベル11 (超超超上級)".to_string(),
            (Language::English, "ai_level11") => "AI Level 11 (Grandmaster)".to_string(),
            (Language::Japanese, "ai_level13") => "AI レベル13 (超超超超上級)".to_string(),
            (Language::English, "ai_level13") => "AI Level 13 (Ultimate)".to_string(),
            (Language::Japanese, "custom") => "カスタム".to_string(),
            (Language::English, "custom") => "Custom".to_string(),

            // Menu
            (Language::Japanese, "player_settings") => "プレイヤー設定".to_string(),
            (Language::English, "player_settings") => "Player Settings".to_string(),
            (Language::Japanese, "black_player") => "黒(先手): ".to_string(),
            (Language::English, "black_player") => "Black (First): ".to_string(),
            (Language::Japanese, "white_player") => "白(後手): ".to_string(),
            (Language::English, "white_player") => "White (Second): ".to_string(),
            (Language::Japanese, "custom_depth") => "カスタム深さ: ".to_string(),
            (Language::English, "custom_depth") => "Custom Depth: ".to_string(),
            (Language::Japanese, "start_game") => "ゲーム開始".to_string(),
            (Language::English, "start_game") => "Start Game".to_string(),
            (Language::Japanese, "language") => "言語 / Language".to_string(),
            (Language::English, "language") => "Language / 言語".to_string(),

            // Game
            (Language::Japanese, "game_info") => "ゲーム情報".to_string(),
            (Language::English, "game_info") => "Game Info".to_string(),
            (Language::Japanese, "ai_thinking") => "AI思考中...".to_string(),
            (Language::English, "ai_thinking") => "AI thinking...".to_string(),
            (Language::Japanese, "return_to_menu") => "メニューに戻る".to_string(),
            (Language::English, "return_to_menu") => "Return to Menu".to_string(),
            (Language::Japanese, "show_stats_graphs") => "統計・グラフ表示".to_string(),
            (Language::English, "show_stats_graphs") => "Show Stats & Graphs".to_string(),
            (Language::Japanese, "new_game") => "新しいゲーム".to_string(),
            (Language::English, "new_game") => "New Game".to_string(),
            (Language::Japanese, "stats_window") => "統計ウィンドウ".to_string(),
            (Language::English, "stats_window") => "Statistics Window".to_string(),

            // Statistics
            (Language::Japanese, "game_statistics") => "ゲーム統計".to_string(),
            (Language::English, "game_statistics") => "Game Statistics".to_string(),

            // Graphs
            (Language::Japanese, "graph_viewer") => "グラフ表示".to_string(),
            (Language::English, "graph_viewer") => "Graph Viewer".to_string(),

            // Board
            (Language::Japanese, "board_size") => "盤面サイズ:".to_string(),
            (Language::English, "board_size") => "Board Size:".to_string(),

            // Fallback
            _ => key.to_string(),
        }
    }

    fn start_new_game(&mut self) {
        self.board = BitBoard::new();
        self.current_player = Player::Black;
        self.pass_count = 0;
        self.game_stats = GameStats::new();
        self.thinking_time = Duration::new(0, 0);
        self.selected_position = None;
        self.ai_thinking = false;
        self.ai_move_receiver = None;

        // プレイヤータイプを設定
        self.black_player = Some(self.black_player_type.to_player_type(self.custom_depth));
        self.white_player = Some(self.white_player_type.to_player_type(self.custom_depth));

        self.state = GameState::Playing;
        self.status_message = match self.language {
            Language::Japanese => format!("{}の手番です", self.current_player.to_string()),
            Language::English => format!("{}'s turn", self.current_player.to_string()),
        };
    }

    fn handle_human_move(&mut self, row: usize, col: usize) -> bool {
        let position = row * 8 + col;
        let legal_moves = self.board.get_legal_moves(self.current_player);

        if (legal_moves & (1u64 << position)) != 0 {
            let start = Instant::now();
            if self.board.make_move(position, self.current_player) {
                let elapsed = start.elapsed();
                self.thinking_time += elapsed;

                let (black_count, white_count) = self.board.count_all_discs();
                self.game_stats.record_move(
                    self.current_player,
                    Some((row, col)),
                    elapsed,
                    black_count,
                    white_count,
                    None,
                );

                self.current_player = self.current_player.opponent();
                self.pass_count = 0;
                return true;
            }
        }
        false
    }

    fn start_ai_thinking(&mut self) {
        if self.ai_thinking {
            return;
        }

        let player_type = match self.current_player {
            Player::Black => self.black_player.as_ref(),
            Player::White => self.white_player.as_ref(),
        };

        if let Some(PlayerType::AI { level, tt: _ }) = player_type {
            self.ai_thinking = true;
            let mut board_copy = self.board.clone();
            let current_player = self.current_player;
            let level = *level;

            let (tx, rx) = mpsc::channel();
            self.ai_move_receiver = Some(rx);

            thread::spawn(move || {
                let start = Instant::now();
                let mut tt = HashMap::default();
                let (best_move, evaluation) =
                    board_copy.find_best_move_with_tt(current_player, level, &mut tt);
                let _elapsed = start.elapsed();

                if let Some(position) = best_move {
                    let row = position / 8;
                    let col = position % 8;
                    let success = board_copy.make_move(position, current_player);
                    tx.send((success, Some((row, col)), evaluation)).ok();
                } else {
                    tx.send((false, None, evaluation)).ok();
                }
            });
        }
    }

    fn check_ai_move(&mut self) {
        if let Some(ref receiver) = self.ai_move_receiver {
            if let Ok((success, move_position, evaluation)) = receiver.try_recv() {
                self.ai_thinking = false;
                self.ai_move_receiver = None;

                let start = Instant::now();

                if success {
                    if let Some((row, col)) = move_position {
                        let position = row * 8 + col;
                        self.board.make_move(position, self.current_player);

                        let elapsed = start.elapsed();
                        self.thinking_time += elapsed;

                        let (black_count, white_count) = self.board.count_all_discs();
                        self.game_stats.record_move(
                            self.current_player,
                            Some((row, col)),
                            elapsed,
                            black_count,
                            white_count,
                            evaluation,
                        );

                        self.current_player = self.current_player.opponent();
                        self.pass_count = 0;
                    }
                } else {
                    // パス
                    let elapsed = start.elapsed();
                    let (black_count, white_count) = self.board.count_all_discs();
                    self.game_stats.record_move(
                        self.current_player,
                        None,
                        elapsed,
                        black_count,
                        white_count,
                        evaluation,
                    );

                    self.current_player = self.current_player.opponent();
                    self.pass_count += 1;
                }
            }
        }
    }

    fn check_game_over(&mut self) {
        if self.board.is_game_over() || self.pass_count >= 2 {
            self.state = GameState::GameOver;

            let (black_count, white_count) = self.board.count_all_discs();
            let winner = self.board.get_winner();

            self.status_message = match (winner, self.language) {
                (Some(Player::Black), Language::Japanese) => {
                    format!("黒の勝ち！ (黒:{} 白:{})", black_count, white_count)
                }
                (Some(Player::Black), Language::English) => {
                    format!("Black wins! (Black:{} White:{})", black_count, white_count)
                }
                (Some(Player::White), Language::Japanese) => {
                    format!("白の勝ち！ (黒:{} 白:{})", black_count, white_count)
                }
                (Some(Player::White), Language::English) => {
                    format!("White wins! (Black:{} White:{})", black_count, white_count)
                }
                (None, Language::Japanese) => {
                    format!("引き分け！ (黒:{} 白:{})", black_count, white_count)
                }
                (None, Language::English) => {
                    format!("Draw! (Black:{} White:{})", black_count, white_count)
                }
            };
        }
    }

    fn generate_and_show_graphs(&mut self) {
        let (black_count, white_count) = self.board.count_all_discs();
        let winner = self.board.get_winner();
        let game_result = self
            .game_stats
            .finalize_game(winner, black_count, white_count);

        // Store data for plot viewer
        self.stored_game_stats = Some(self.game_stats.clone_for_plotting());
        self.stored_game_result = Some(game_result);
        self.plot_viewer.mark_data_available();

        self.show_plot_window = true;
        self.status_message = match self.language {
            Language::Japanese => "グラフを表示しました！".to_string(),
            Language::English => "Graphs displayed!".to_string(),
        };
    }
}

impl eframe::App for OthelloApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // AI思考のチェック
        if self.ai_thinking {
            self.check_ai_move();
        }

        // ゲーム状態の更新
        if self.state == GameState::Playing && !self.ai_thinking {
            self.check_game_over();

            if self.state == GameState::Playing {
                // 現在のプレイヤーがAIで、まだ思考中でない場合は思考開始
                let is_ai = match self.current_player {
                    Player::Black => matches!(self.black_player, Some(PlayerType::AI { .. })),
                    Player::White => matches!(self.white_player, Some(PlayerType::AI { .. })),
                };

                if is_ai {
                    self.start_ai_thinking();
                }

                // 合法手をチェック
                let legal_moves = self.board.get_legal_moves(self.current_player);
                if legal_moves == 0 && !self.ai_thinking {
                    self.status_message = match self.language {
                        Language::Japanese => {
                            format!("{}はパスします", self.current_player.to_string())
                        }
                        Language::English => format!("{} passes", self.current_player.to_string()),
                    };
                    self.current_player = self.current_player.opponent();
                    self.pass_count += 1;
                } else if !is_ai {
                    self.status_message = match self.language {
                        Language::Japanese => {
                            format!("{}の手番です", self.current_player.to_string())
                        }
                        Language::English => format!("{}'s turn", self.current_player.to_string()),
                    };
                }
            }
        }

        // メインUI
        egui::CentralPanel::default().show(ctx, |ui| match self.state {
            GameState::Menu => self.show_menu(ui),
            GameState::Playing | GameState::GameOver => self.show_game(ui, ctx),
            GameState::ViewingStats => self.show_stats(ui),
        });

        // 統計ウィンドウ
        if self.show_stats_window {
            let mut show_stats = self.show_stats_window;
            egui::Window::new(Self::t(self.language, "game_statistics"))
                .open(&mut show_stats)
                .show(ctx, |ui| {
                    let move_count = self.game_stats.get_move_count();
                    ui.label(Self::t(self.language, "game_statistics"));
                    ui.separator();
                    match self.language {
                        Language::Japanese => {
                            ui.label(format!("総手数: {}", move_count));
                            ui.label(format!("思考時間: {:.2?}", self.thinking_time));
                            if move_count > 0 {
                                ui.label(format!(
                                    "平均思考時間: {:.2?}",
                                    self.thinking_time / move_count as u32
                                ));
                            }
                        }
                        Language::English => {
                            ui.label(format!("Total moves: {}", move_count));
                            ui.label(format!("Thinking time: {:.2?}", self.thinking_time));
                            if move_count > 0 {
                                ui.label(format!(
                                    "Average thinking time: {:.2?}",
                                    self.thinking_time / move_count as u32
                                ));
                            }
                        }
                    }
                });
            self.show_stats_window = show_stats;
        }

        // プロット表示ウィンドウ
        if self.show_plot_window {
            egui::Window::new(Self::t(self.language, "graph_viewer"))
                .open(&mut self.show_plot_window)
                .default_size([900.0, 700.0])
                .show(ctx, |ui| {
                    if let (Some(ref stats), Some(ref result)) =
                        (&self.stored_game_stats, &self.stored_game_result)
                    {
                        self.plot_viewer.show(ui, self.language, stats, result);
                    } else {
                        let no_data_text = match self.language {
                            Language::Japanese => "グラフデータがありません。",
                            Language::English => "No graph data available.",
                        };
                        ui.label(no_data_text);
                    }
                });
        }

        // 継続的な更新
        ctx.request_repaint();
    }
}

impl OthelloApp {
    fn show_menu(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading(Self::t(self.language, "title"));
            ui.add_space(20.0);

            // Language selector
            ui.horizontal(|ui| {
                ui.label(Self::t(self.language, "language"));
                if ui.button("日本語").clicked() {
                    self.language = Language::Japanese;
                }
                if ui.button("English").clicked() {
                    self.language = Language::English;
                }
            });

            ui.add_space(30.0);

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(Self::t(self.language, "player_settings"));
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label(Self::t(self.language, "black_player"));
                        egui::ComboBox::from_id_source("black_player")
                            .selected_text(Self::get_player_type_text(
                                self.language,
                                self.black_player_type,
                            ))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.black_player_type,
                                    PlayerTypeSelection::Human,
                                    Self::t(self.language, "human"),
                                );
                                ui.selectable_value(
                                    &mut self.black_player_type,
                                    PlayerTypeSelection::AI1,
                                    Self::t(self.language, "ai_level1"),
                                );
                                ui.selectable_value(
                                    &mut self.black_player_type,
                                    PlayerTypeSelection::AI3,
                                    Self::t(self.language, "ai_level3"),
                                );
                                ui.selectable_value(
                                    &mut self.black_player_type,
                                    PlayerTypeSelection::AI5,
                                    Self::t(self.language, "ai_level5"),
                                );
                                ui.selectable_value(
                                    &mut self.black_player_type,
                                    PlayerTypeSelection::AI7,
                                    Self::t(self.language, "ai_level7"),
                                );
                                ui.selectable_value(
                                    &mut self.black_player_type,
                                    PlayerTypeSelection::AI9,
                                    Self::t(self.language, "ai_level9"),
                                );
                                ui.selectable_value(
                                    &mut self.black_player_type,
                                    PlayerTypeSelection::AI11,
                                    Self::t(self.language, "ai_level11"),
                                );
                                ui.selectable_value(
                                    &mut self.black_player_type,
                                    PlayerTypeSelection::AI13,
                                    Self::t(self.language, "ai_level13"),
                                );
                                ui.selectable_value(
                                    &mut self.black_player_type,
                                    PlayerTypeSelection::Custom,
                                    Self::t(self.language, "custom"),
                                );
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label(Self::t(self.language, "white_player"));
                        egui::ComboBox::from_id_source("white_player")
                            .selected_text(Self::get_player_type_text(
                                self.language,
                                self.white_player_type,
                            ))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.white_player_type,
                                    PlayerTypeSelection::Human,
                                    Self::t(self.language, "human"),
                                );
                                ui.selectable_value(
                                    &mut self.white_player_type,
                                    PlayerTypeSelection::AI1,
                                    Self::t(self.language, "ai_level1"),
                                );
                                ui.selectable_value(
                                    &mut self.white_player_type,
                                    PlayerTypeSelection::AI3,
                                    Self::t(self.language, "ai_level3"),
                                );
                                ui.selectable_value(
                                    &mut self.white_player_type,
                                    PlayerTypeSelection::AI5,
                                    Self::t(self.language, "ai_level5"),
                                );
                                ui.selectable_value(
                                    &mut self.white_player_type,
                                    PlayerTypeSelection::AI7,
                                    Self::t(self.language, "ai_level7"),
                                );
                                ui.selectable_value(
                                    &mut self.white_player_type,
                                    PlayerTypeSelection::AI9,
                                    Self::t(self.language, "ai_level9"),
                                );
                                ui.selectable_value(
                                    &mut self.white_player_type,
                                    PlayerTypeSelection::AI11,
                                    Self::t(self.language, "ai_level11"),
                                );
                                ui.selectable_value(
                                    &mut self.white_player_type,
                                    PlayerTypeSelection::AI13,
                                    Self::t(self.language, "ai_level13"),
                                );
                                ui.selectable_value(
                                    &mut self.white_player_type,
                                    PlayerTypeSelection::Custom,
                                    Self::t(self.language, "custom"),
                                );
                            });
                    });

                    if self.black_player_type == PlayerTypeSelection::Custom
                        || self.white_player_type == PlayerTypeSelection::Custom
                    {
                        ui.horizontal(|ui| {
                            ui.label(Self::t(self.language, "custom_depth"));
                            ui.add(egui::Slider::new(&mut self.custom_depth, 1..=15));
                        });
                    }
                });
            });

            ui.add_space(30.0);

            if ui.button(Self::t(self.language, "start_game")).clicked() {
                self.start_new_game();
            }
        });
    }

    fn get_player_type_text(language: Language, player_type: PlayerTypeSelection) -> String {
        match player_type {
            PlayerTypeSelection::Human => Self::t(language, "human"),
            PlayerTypeSelection::AI1 => Self::t(language, "ai_level1"),
            PlayerTypeSelection::AI3 => Self::t(language, "ai_level3"),
            PlayerTypeSelection::AI5 => Self::t(language, "ai_level5"),
            PlayerTypeSelection::AI7 => Self::t(language, "ai_level7"),
            PlayerTypeSelection::AI9 => Self::t(language, "ai_level9"),
            PlayerTypeSelection::AI11 => Self::t(language, "ai_level11"),
            PlayerTypeSelection::AI13 => Self::t(language, "ai_level13"),
            PlayerTypeSelection::Custom => Self::t(language, "custom"),
        }
    }

    fn show_game(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.horizontal(|ui| {
            // ゲームボード
            ui.vertical(|ui| {
                ui.label(&self.status_message);
                ui.add_space(10.0);

                let is_human = match self.current_player {
                    Player::Black => {
                        matches!(self.black_player, Some(PlayerType::Human))
                    }
                    Player::White => {
                        matches!(self.white_player, Some(PlayerType::Human))
                    }
                };

                if let Some((row, col)) =
                    self.game_view
                        .show(&self.board, self.current_player, ui, self.language)
                {
                    if self.state == GameState::Playing && !self.ai_thinking && is_human {
                        self.handle_human_move(row, col);
                    }
                }
            });

            ui.separator();

            // サイドパネル
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(Self::t(self.language, "game_info"));
                        ui.add_space(5.0);

                        let (black_count, white_count) = self.board.count_all_discs();
                        match self.language {
                            Language::Japanese => {
                                ui.label(format!("黒: {} 個", black_count));
                                ui.label(format!("白: {} 個", white_count));
                            }
                            Language::English => {
                                ui.label(format!("Black: {} pieces", black_count));
                                ui.label(format!("White: {} pieces", white_count));
                            }
                        }

                        if self.ai_thinking {
                            ui.label(Self::t(self.language, "ai_thinking"));
                            ui.spinner();
                        }
                    });
                });

                ui.add_space(10.0);

                if ui
                    .button(Self::t(self.language, "return_to_menu"))
                    .clicked()
                {
                    self.state = GameState::Menu;
                }

                if self.state == GameState::GameOver {
                    ui.add_space(10.0);
                    if ui
                        .button(Self::t(self.language, "show_stats_graphs"))
                        .clicked()
                    {
                        self.generate_and_show_graphs();
                    }

                    if ui.button(Self::t(self.language, "new_game")).clicked() {
                        self.start_new_game();
                    }
                }

                if ui.button(Self::t(self.language, "stats_window")).clicked() {
                    self.show_stats_window = true;
                }
            });
        });
    }

    fn show_stats(&mut self, ui: &mut egui::Ui) {
        match self.language {
            Language::Japanese => ui.label("統計表示（開発中）"),
            Language::English => ui.label("Statistics display (under development)"),
        };
        match self.language {
            Language::Japanese => {
                if ui.button("ゲームに戻る").clicked() {
                    self.state = GameState::Playing;
                }
            }
            Language::English => {
                if ui.button("Return to Game").clicked() {
                    self.state = GameState::Playing;
                }
            }
        }
    }
}
