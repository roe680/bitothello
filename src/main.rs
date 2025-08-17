mod ai;
mod board;
mod gui;
mod player;
mod stats;
mod test_graphs;

use board::BitBoard;
use player::{Player, PlayerType};
use stats::{plot_game_statistics, GameStats};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{self, Write};
use std::time::{Duration, Instant};

use crate::gui::japanese::setup_custom_fonts;

fn main() {
    // コマンドライン引数をチェック
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "test-graphs" {
        test_graphs::run_graph_test();
        return;
    }
    if args.len() > 1 && args[1] == "quick-game" {
        run_quick_ai_game();
        return;
    }
    if args.len() > 1 && args[1] == "cli" {
        run_cli_game();
        return;
    }

    // デフォルトでGUIを起動
    run_gui();
}

fn run_cli_game() {
    // タイトル表示
    println!("==========================");
    println!("    ビット オセロ");
    println!("==========================");

    // プレイヤータイプを選択
    let (black_player, white_player) = select_player_types();

    // ゲームの初期化
    let mut board = BitBoard::new();
    println!("\nゲーム開始！");
    println!("{}", board);

    // ゲーム統計情報の初期化
    let mut game_stats = GameStats::new();
    let mut _total_moves = 0;
    let mut thinking_time = Duration::new(0, 0);

    // ゲームループ
    let mut current_player = Player::Black;
    let mut pass_count = 0;

    while !board.is_game_over() {
        println!(
            "現在の手番: {}({})",
            current_player.to_string(),
            current_player.to_char()
        );

        // 合法手を高速に取得
        let legal_moves = board.get_legal_moves(current_player);
        if legal_moves == 0 {
            println!("打てる場所がありません。パスします。");
            current_player = current_player.opponent();
            pass_count += 1;

            // 連続パスでゲーム終了条件をチェック
            if pass_count >= 2 {
                println!("両者パスのためゲーム終了");
                break;
            }

            continue;
        }

        // パスカウントリセット
        pass_count = 0;

        // 合法手の数を素早くカウント
        let legal_move_count = legal_moves.count_ones() as usize;

        // 合法手の一覧を表示（最適化版 - 大量にある場合は省略）
        println!("打てる場所: {legal_move_count}箇所");
        if legal_move_count <= 12 {
            // 数が少ない場合のみ全表示
            let mut positions = Vec::with_capacity(legal_move_count);
            for pos in 0..64 {
                if (legal_moves & (1u64 << pos)) != 0 {
                    let row = pos / 8;
                    let col = pos % 8;
                    positions.push((row, col));
                }
            }
            print!("具体的な位置: ");
            for (row, col) in positions {
                print!("({},{}) ", row, col);
            }
            println!();
        }

        // プレイヤータイプに応じた処理
        let player_type = match current_player {
            Player::Black => &black_player,
            Player::White => &white_player,
        };
        // 時間計測
        let start = Instant::now();
        let (success, move_position, evaluation) =
            player_type.play_turn(&mut board, current_player);
        if success {
            // 成功したら盤面表示して手番交代
            let elapsed = start.elapsed();
            thinking_time += elapsed;
            _total_moves += 1;

            // 統計記録
            let (black_count, white_count) = board.count_all_discs();
            game_stats.record_move(
                current_player,
                move_position,
                elapsed,
                black_count,
                white_count,
                evaluation,
            );

            // 盤面表示
            println!("{}", board);

            // 手番交代
            current_player = current_player.opponent();
            println!("思考時間: {:.2?}", elapsed);
        } else {
            // パスの場合も記録
            let elapsed = start.elapsed();
            let (black_count, white_count) = board.count_all_discs();
            game_stats.record_move(
                current_player,
                None, // パス
                elapsed,
                black_count,
                white_count,
                None,
            );
        }
    }

    // ゲーム終了
    println!("\n==========================");
    println!("      ゲーム終了");
    println!("==========================");

    let (black_count, white_count) = board.count_all_discs();
    println!("黒(X): {} 白(O): {}", black_count, white_count);

    let winner = board.get_winner();
    match winner {
        Some(Player::Black) => println!("黒の勝ち！"),
        Some(Player::White) => println!("白の勝ち！"),
        None => println!("引き分け！"),
    }

    // ゲーム結果の最終化
    let game_result = game_stats.finalize_game(winner, black_count, white_count);

    println!("\n==========================");
    println!("      ゲーム統計");
    println!("==========================");
    println!("総手数: {}", game_result.total_moves);
    println!("総思考時間: {:.2?}", game_result.total_thinking_time);
    println!("ゲーム所要時間: {:.2?}", game_result.game_duration);
    if game_result.total_moves > 0 {
        println!(
            "1手あたりの平均思考時間: {:.2?}",
            game_result.total_thinking_time / game_result.total_moves as u32
        );
    }

    // 詳細統計の表示
    game_stats.print_summary(&game_result);

    // グラフの生成
    println!("\nグラフを生成中...");
    match plot_game_statistics(&game_stats, &game_result) {
        Ok(()) => println!("グラフ生成が完了しました！"),
        Err(e) => println!("グラフ生成エラー: {}", e),
    }
}

/// プレイヤータイプを選択する関数（最適化版）
fn select_player_types() -> (PlayerType, PlayerType) {
    println!("プレイヤー設定を行います。");

    let black_player = select_single_player_type("黒(先手)");
    let white_player = select_single_player_type("白(後手)");

    println!("\n対局設定:");
    println!("・黒(X): {}", player_type_to_string(&black_player));
    println!("・白(O): {}", player_type_to_string(&white_player));

    (black_player, white_player)
}

/// プレイヤータイプを文字列に変換
fn player_type_to_string(player_type: &PlayerType) -> String {
    match player_type {
        PlayerType::Human => String::from("人間"),
        PlayerType::AI { level, tt: _ } => {
            let difficulty = match level {
                1 => "初級",
                3 => "中級",
                5 => "上級",
                7 => "超上級",
                9 => "超超上級",
                11 => "超超超上級",
                13 => "超超超超上級",
                _ => "カスタム",
            };
            format!("AI (レベル{} - {})", level, difficulty)
        }
    }
}

/// 単一プレイヤーのタイプを選択する関数
fn select_single_player_type(player_name: &str) -> PlayerType {
    loop {
        println!("\n{}のプレイヤータイプを選択:", player_name);
        println!("1: 人間");
        println!("2: AI レベル1（初級）");
        println!("3: AI レベル3（中級）");
        println!("4: AI レベル5（上級）");
        println!("5: AI レベル7（超上級）");
        println!("6: AI レベル9（超超上級）");
        println!("7: AI レベル11（超超超上級）");
        println!("8: AI レベル13（超超超超上級）");
        println!("9: カスタム（任意の深さを指定）");
        print!("選択 (1-9): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                // 入力を処理
                match input.trim() {
                    "1" => return PlayerType::Human,
                    "2" => {
                        return PlayerType::AI {
                            level: 1,
                            tt: RefCell::new(HashMap::default()),
                        }
                    }
                    "3" => {
                        return PlayerType::AI {
                            level: 3,
                            tt: RefCell::new(HashMap::default()),
                        }
                    }
                    "4" => {
                        return PlayerType::AI {
                            level: 5,
                            tt: RefCell::new(HashMap::default()),
                        }
                    }
                    "5" => {
                        return PlayerType::AI {
                            level: 7,
                            tt: RefCell::new(HashMap::default()),
                        }
                    }
                    "6" => {
                        return PlayerType::AI {
                            level: 9,
                            tt: RefCell::new(HashMap::default()),
                        }
                    }
                    "7" => {
                        return PlayerType::AI {
                            level: 11,
                            tt: RefCell::new(HashMap::default()),
                        }
                    }
                    "8" => {
                        return PlayerType::AI {
                            level: 13,
                            tt: RefCell::new(HashMap::default()),
                        }
                    }
                    "9" => {
                        // カスタム深さの入力
                        loop {
                            print!("AI の深さを入力してください (1-20): ");
                            io::stdout().flush().unwrap();

                            let mut depth_input = String::new();
                            match io::stdin().read_line(&mut depth_input) {
                                Ok(_) => match depth_input.trim().parse::<usize>() {
                                    Ok(depth) if depth >= 1 && depth <= 20 => {
                                        println!("カスタム AI (深さ {}) を選択しました", depth);
                                        return PlayerType::AI {
                                            level: depth + 1,
                                            tt: RefCell::new(HashMap::default()),
                                        };
                                    }
                                    Ok(_) => println!("深さは 1-20 の範囲で入力してください。"),
                                    Err(_) => println!("無効な入力です。数字を入力してください。"),
                                },
                                Err(_) => println!("入力エラー。もう一度入力してください。"),
                            }
                        }
                    }
                    "q" | "quit" | "exit" => {
                        println!("プログラムを終了します。");
                        std::process::exit(0);
                    }
                    _ => println!("無効な選択です。1-9の数字を入力してください。"),
                }
            }
            Err(_) => {
                println!("入力エラー。もう一度選択してください。");
                continue;
            }
        }
    }
}

/// クイックAI対戦（グラフ生成テスト用）
fn run_quick_ai_game() {
    println!("==========================");
    println!("  クイックAI対戦テスト");
    println!("==========================");

    // AI レベル20 vs AI レベル20 の短い試合
    let black_player = PlayerType::AI {
        level: 20,
        tt: RefCell::new(HashMap::default()),
    };
    let white_player = PlayerType::AI {
        level: 20,
        tt: RefCell::new(HashMap::default()),
    };

    println!("AI (レベル20) vs AI (レベル20) で対戦します...");

    let mut board = BitBoard::new();
    let mut game_stats = GameStats::new();
    let mut current_player = Player::Black;
    let mut pass_count = 0;
    let mut move_count = 0;
    const MAX_MOVES: usize = 30; // より長い試合でAIの性能をテスト

    while !board.is_game_over() && move_count < MAX_MOVES {
        let legal_moves = board.get_legal_moves(current_player);
        if legal_moves == 0 {
            pass_count += 1;
            if pass_count >= 2 {
                break;
            }
            current_player = current_player.opponent();
            continue;
        }

        pass_count = 0;
        let player_type = match current_player {
            Player::Black => &black_player,
            Player::White => &white_player,
        };

        let start = Instant::now();
        let (success, move_position, evaluation) =
            player_type.play_turn(&mut board, current_player);

        if success {
            let elapsed = start.elapsed();
            let (black_count, white_count) = board.count_all_discs();

            game_stats.record_move(
                current_player,
                move_position,
                elapsed,
                black_count,
                white_count,
                evaluation,
            );

            move_count += 1;
            current_player = current_player.opponent();

            if move_count % 5 == 0 {
                println!(
                    "{}手目完了 (黒:{}個 白:{}個) - 思考時間: {:.3}s",
                    move_count,
                    black_count,
                    white_count,
                    elapsed.as_secs_f64()
                );
            }
        }
    }

    let (black_count, white_count) = board.count_all_discs();
    let winner = if black_count > white_count {
        Some(Player::Black)
    } else if white_count > black_count {
        Some(Player::White)
    } else {
        None
    };

    println!("\nクイックゲーム終了！");
    println!("最終スコア - 黒: {} 白: {}", black_count, white_count);

    let game_result = game_stats.finalize_game(winner, black_count, white_count);

    // グラフの生成
    println!("\nグラフを生成中...");
    match plot_game_statistics(&game_stats, &game_result) {
        Ok(()) => println!("✓ グラフ生成が完了しました！"),
        Err(e) => println!("❌ グラフ生成エラー: {}", e),
    }
}

/// GUI版のゲームを実行
fn run_gui() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("ビット オセロ"),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "ビット オセロ",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Box::new(gui::OthelloApp::new(cc))
        }),
    ) {
        eprintln!("GUI実行エラー: {}", e);
    }
}
