mod ai;
mod board;
mod player;

use board::BitBoard;
use player::{Player, PlayerType};
use std::io::{self, Write};
use std::time::{Duration, Instant};

fn main() {
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
    let game_start_time = Instant::now();
    let mut total_moves = 0;
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
        if player_type.play_turn(&mut board, current_player) {
            // 成功したら盤面表示して手番交代
            let elapsed = start.elapsed();
            thinking_time += elapsed;
            total_moves += 1;

            // 盤面表示
            println!("{}", board);

            // 手番交代
            current_player = current_player.opponent();
            println!("思考時間: {:.2?}", elapsed);
        }
    }

    // ゲーム終了
    // ゲーム終了統計
    let game_duration = game_start_time.elapsed();

    println!("\n==========================");
    println!("      ゲーム終了");
    println!("==========================");

    let (black_count, white_count) = board.count_all_discs();
    println!("黒(X): {} 白(O): {}", black_count, white_count);

    match board.get_winner() {
        Some(Player::Black) => println!("黒の勝ち！"),
        Some(Player::White) => println!("白の勝ち！"),
        None => println!("引き分け！"),
    }

    println!("\n==========================");
    println!("      ゲーム統計");
    println!("==========================");
    println!("総手数: {}", total_moves);
    println!("総思考時間: {:.2?}", thinking_time);
    println!("ゲーム所要時間: {:.2?}", game_duration);
    if total_moves > 0 {
        println!(
            "1手あたりの平均思考時間: {:.2?}",
            thinking_time / total_moves as u32
        );
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
        PlayerType::AI { level } => {
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
        print!("選択 (1-8): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                // 入力を処理
                match input.trim() {
                    "1" => return PlayerType::Human,
                    "2" => return PlayerType::AI { level: 1 },
                    "3" => return PlayerType::AI { level: 3 },
                    "4" => return PlayerType::AI { level: 5 },
                    "5" => return PlayerType::AI { level: 7 },
                    "6" => return PlayerType::AI { level: 9 },
                    "7" => return PlayerType::AI { level: 11 },
                    "8" => return PlayerType::AI { level: 13 },
                    "q" | "quit" | "exit" => {
                        println!("プログラムを終了します。");
                        std::process::exit(0);
                    }
                    _ => println!("無効な選択です。1-8の数字を入力してください。"),
                }
            }
            Err(_) => {
                println!("入力エラー。もう一度選択してください。");
                continue;
            }
        }
    }
}

// この部分を空にする - モジュールに移動済み
