use crate::board::BitBoard;

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

#[derive(Debug, Clone)]
pub enum PlayerType {
    Human,
    AI { level: usize },
}

impl PlayerType {
    /// 指定されたプレイヤータイプでゲームを実行する
    pub fn play_turn(&self, board: &mut BitBoard, player: Player) -> bool {
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
                                    return true;
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
            PlayerType::AI { level } => {
                // 思考処理（AIレベルによって動的に調整）
                let start_thinking = std::time::Instant::now();

                // 終盤ではより深く読む（適応型深度調整）
                let empty_count = 64 - (board.black | board.white).count_ones() as usize;
                let adaptive_level = if empty_count <= 12 && *level >= 5 {
                    // 終盤は読みを深くする（石数が少ない時）
                    *level + 2
                } else {
                    *level
                };

                // 最善手を探索
                if let Some(pos) = board.find_best_move(player, adaptive_level) {
                    // 思考時間表示のためにスリープ時間を調整（早すぎるとユーザーが混乱するのを防ぐ）
                    let elapsed = start_thinking.elapsed();
                    if elapsed < std::time::Duration::from_millis(300) && *level < 9 {
                        std::thread::sleep(std::time::Duration::from_millis(300) - elapsed);
                    }

                    let row = pos / 8;
                    let col = pos % 8;
                    println!("{}(AI)は({},{})に置きました", player.to_string(), row, col);
                    board.make_move(pos, player);
                    true
                } else {
                    println!("{}(AI)はパスします", player.to_string());
                    false
                }
            }
        }
    }
}
