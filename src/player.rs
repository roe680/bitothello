use crate::board::BitBoard;
use fxhash::FxHashMap;
use std::cell::RefCell;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Player {
    Black,
    White,
}

impl Player {
    /// 相手のプレイヤーを返す
    pub fn opponent(&self) -> Self {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }

    /// 文字列表現を返す
    pub fn to_string(&self) -> &'static str {
        match self {
            Player::Black => "黒",
            Player::White => "白",
        }
    }

    /// 文字表現を返す
    pub fn to_char(&self) -> char {
        match self {
            Player::Black => 'X',
            Player::White => 'O',
        }
    }
}

pub enum PlayerType {
    Human,
    AI {
        level: usize,
        tt: RefCell<FxHashMap<(u64, u64, u8), Entry>>, //black, white, playerの順
    },
}

impl Clone for PlayerType {
    fn clone(&self) -> Self {
        match self {
            PlayerType::Human => PlayerType::Human,
            PlayerType::AI { level, tt } => PlayerType::AI {
                level: *level,
                tt: RefCell::new(tt.borrow().clone()),
            },
        }
    }
}

#[derive(Clone, Copy)]
pub struct Entry {
    pub score: i32,            //このボードの評価
    pub depth: u8,             //この評価を出すために何手先まで読んだか（再利用可否に使う）
    pub flag: NodeType,        // Exact / LowerBound / UpperBound
    pub best_move: Option<u8>, //最善手（あれば）例：0〜63 で盤面の場所を表す
}

#[derive(Clone, Copy)]
pub enum NodeType {
    Exact,
    LowerBound,
    UpperBound,
}

/*Exact
この局面のスコアは「完全に正確に読んだやつ」→ 再利用してOK
LowerBound
「これ以上良い評価がある可能性がある」（例：βカットで途中終了）
UpperBound
「これ以下の評価しかない」（例：αカットで途中終了）
 */

impl PlayerType {
    /// 指定されたプレイヤータイプでゲームを実行する
    /// 戻り値: (成功したかどうか, 手の位置, AI評価値)
    pub fn play_turn(
        &self,
        board: &mut BitBoard,
        player: Player,
    ) -> (bool, Option<(usize, usize)>, Option<i32>) {
        match self {
            PlayerType::Human => {
                println!("行(0-7) 列(0-7) の形式で入力。例: 3 2");
                println!("ヘルプ: 'h'または'help', ゲーム終了: 'q'または'quit'");

                // 合法手の位置リストを用意（ヘルプ表示用）
                let legal_pos_list: Vec<(usize, usize)> = (0..64)
                    .filter(|&pos| (board.get_legal_moves(player) & (1u64 << pos)) != 0)
                    .map(|pos| (pos / 8, pos % 8))
                    .collect();

                loop {
                    let mut input = String::new();
                    match std::io::stdin().read_line(&mut input) {
                        Ok(_) => {
                            let input = input.trim().to_lowercase();

                            // 特殊コマンドの処理
                            match input.as_str() {
                                "q" | "quit" | "exit" => {
                                    println!("ゲームを終了します。");
                                    std::process::exit(0);
                                }
                                "h" | "help" | "?" => {
                                    println!("--ヘルプ--");
                                    println!("・行と列の番号を半角スペースで区切って入力します。");
                                    println!("・例: '2 3' は行2, 列3に石を置きます。");
                                    println!("・現在の合法手リスト:");
                                    for (i, &(row, col)) in legal_pos_list.iter().enumerate() {
                                        print!("({},{}) ", row, col);
                                        if (i + 1) % 8 == 0 {
                                            println!();
                                        }
                                    }
                                    if legal_pos_list.len() % 8 != 0 {
                                        println!();
                                    }
                                    continue;
                                }
                                _ => {}
                            }

                            // 通常の手の入力を解析
                            let parts: Vec<&str> = input.split_whitespace().collect();
                            if parts.len() != 2 {
                                println!(
                                    "無効な入力形式です。行(0-7) 列(0-7) の形式で入力してください。"
                                );
                                continue;
                            }

                            let row: Result<usize, _> = parts[0].parse();
                            let col: Result<usize, _> = parts[1].parse();

                            if let (Ok(row), Ok(col)) = (row, col) {
                                if row >= 8 || col >= 8 {
                                    println!(
                                        "無効な座標です。行と列は0-7の範囲で指定してください。"
                                    );
                                    continue;
                                }

                                let pos = row * 8 + col;
                                if board.is_legal_move(pos, player) {
                                    println!("{}を({},{})に置きます", player.to_string(), row, col);
                                    board.make_move(pos, player);
                                    return (true, Some((row, col)), None);
                                } else {
                                    println!("そこには置けません。別の場所を選んでください。");
                                    println!(
                                        "'h'または'help'と入力すると合法手の一覧を表示します。"
                                    );
                                    continue;
                                }
                            } else {
                                println!("無効な入力です。数字を入力してください。");
                                continue;
                            }
                        }
                        Err(_) => {
                            println!("入力エラー。もう一度入力してください。");
                            continue;
                        }
                    }
                }
            }
            PlayerType::AI { level, tt } => {
                let start_thinking = std::time::Instant::now();

                // 適応的深度調整（最適化版）
                let empty_count = 64 - (board.black | board.white).count_ones() as usize;
                let total_moves = 64 - empty_count;

                let adaptive_level = match empty_count {
                    0..=8 => {
                        // 超終盤：完全読み
                        std::cmp::min(empty_count + 4, *level + 6)
                    }
                    9..=16 => {
                        // 終盤：深く読む
                        std::cmp::min(*level + 3, 20)
                    }
                    17..=40 => {
                        // 中盤：標準的な深度
                        *level
                    }
                    _ => {
                        // 序盤：効率重視
                        std::cmp::max(*level - 1, 1)
                    }
                };

                // メモリクリーンアップの頻度を調整
                {
                    let mut tt_borrowed = tt.borrow_mut();
                    if tt_borrowed.len() > 5_000_000 && total_moves % 8 == 0 {
                        // 8手ごとにクリーンアップ
                        let retain_count = 2_000_000;
                        let mut entries: Vec<_> =
                            tt_borrowed.iter().map(|(k, v)| (*k, *v)).collect();
                        entries.sort_by_key(|(_, entry)| std::cmp::Reverse(entry.depth));

                        tt_borrowed.clear();
                        for (key, entry) in entries.into_iter().take(retain_count) {
                            tt_borrowed.insert(key, entry);
                        }
                    }
                }

                // 最善手探索
                let (pos, evaluation) = {
                    let mut tt_borrowed = tt.borrow_mut();
                    board.find_best_move_with_tt(player, adaptive_level, &mut *tt_borrowed)
                };

                if let Some(pos) = pos {
                    // 思考時間の調整（レベルに応じて）
                    let elapsed = start_thinking.elapsed();
                    let min_thinking_time = match *level {
                        1..=3 => std::time::Duration::from_millis(200),
                        4..=6 => std::time::Duration::from_millis(300),
                        7..=10 => std::time::Duration::from_millis(500),
                        _ => std::time::Duration::from_millis(1000),
                    };

                    if elapsed < min_thinking_time {
                        std::thread::sleep(min_thinking_time - elapsed);
                    }

                    let row = pos / 8;
                    let col = pos % 8;

                    // 詳細情報の表示（デバッグ用）
                    if *level >= 8 {
                        println!(
                            "{}(AI Lv.{})は({},{})に置きました [深度:{}, 評価:{:?}, 思考時間:{:.2}s]",
                            player.to_string(),
                            adaptive_level,
                            row,
                            col,
                            adaptive_level,
                            evaluation,
                            start_thinking.elapsed().as_secs_f64()
                        );
                    } else {
                        println!("{}(AI)は({},{})に置きました", player.to_string(), row, col);
                    }

                    board.make_move(pos, player);
                    (true, Some((row, col)), evaluation)
                } else {
                    println!("{}(AI)はパスします", player.to_string());
                    (false, None, None)
                }
            }
        }
    }
}
