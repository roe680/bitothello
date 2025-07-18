use crate::player::Player;
use std::time::{Duration, Instant};

/// 一手の記録
#[derive(Debug, Clone)]
pub struct MoveRecord {
    pub move_number: usize,
    pub player: Player,
    pub position: Option<(usize, usize)>, // None はパス
    pub thinking_time: Duration,
    pub black_count: u32,
    pub white_count: u32,
    pub evaluation: Option<i32>, // AI の評価値（人間の場合は None）
}

/// ゲーム結果
#[derive(Debug, Clone)]
pub struct GameResult {
    pub winner: Option<Player>,
    pub black_final_count: u32,
    pub white_final_count: u32,
    pub total_moves: usize,
    pub game_duration: Duration,
    pub total_thinking_time: Duration,
}

/// ゲーム統計を記録するクラス
#[derive(Debug)]
pub struct GameStats {
    pub moves: Vec<MoveRecord>,
    pub game_start_time: Instant,
    current_move_number: usize,
}

impl GameStats {
    pub fn new() -> Self {
        Self {
            moves: Vec::new(),
            game_start_time: Instant::now(),
            current_move_number: 0,
        }
    }

    /// 手を記録
    pub fn record_move(
        &mut self,
        player: Player,
        position: Option<(usize, usize)>,
        thinking_time: Duration,
        black_count: u32,
        white_count: u32,
        evaluation: Option<i32>,
    ) {
        if position.is_some() {
            self.current_move_number += 1;
        }

        let record = MoveRecord {
            move_number: self.current_move_number,
            player,
            position,
            thinking_time,
            black_count,
            white_count,
            evaluation,
        };

        self.moves.push(record);
    }

    /// ゲーム結果を生成
    pub fn finalize_game(
        &self,
        winner: Option<Player>,
        black_count: u32,
        white_count: u32,
    ) -> GameResult {
        let total_moves = self.current_move_number;
        let game_duration = self.game_start_time.elapsed();
        let total_thinking_time: Duration = self
            .moves
            .iter()
            .filter(|m| m.position.is_some()) // パスは除外
            .map(|m| m.thinking_time)
            .sum();

        GameResult {
            winner,
            black_final_count: black_count,
            white_final_count: white_count,
            total_moves,
            game_duration,
            total_thinking_time,
        }
    }

    /// 石数の推移を取得
    pub fn get_disc_count_history(&self) -> Vec<(usize, u32, u32)> {
        self.moves
            .iter()
            .filter(|m| m.position.is_some())
            .map(|m| (m.move_number, m.black_count, m.white_count))
            .collect()
    }

    /// 思考時間の推移を取得
    pub fn get_thinking_time_history(&self) -> Vec<(usize, f64)> {
        self.moves
            .iter()
            .filter(|m| m.position.is_some())
            .map(|m| (m.move_number, m.thinking_time.as_secs_f64()))
            .collect()
    }

    /// 評価値の推移を取得（AI のみ）
    pub fn get_evaluation_history(&self) -> Vec<(usize, Player, i32)> {
        self.moves
            .iter()
            .filter_map(|m| {
                if let (Some(_pos), Some(eval)) = (m.position, m.evaluation) {
                    Some((m.move_number, m.player, eval))
                } else {
                    None
                }
            })
            .collect()
    }

    /// 手数を取得
    pub fn get_move_count(&self) -> usize {
        self.current_move_number
    }

    /// プロット用のクローンを作成（Instantを現在時刻で置き換え）
    pub fn clone_for_plotting(&self) -> GameStats {
        GameStats {
            moves: self.moves.clone(),
            game_start_time: Instant::now(),
            current_move_number: self.current_move_number,
        }
    }

    /// 統計サマリーを表示
    pub fn print_summary(&self, game_result: &GameResult) {
        println!("\n==========================");
        println!("      詳細統計");
        println!("==========================");

        println!("手数分析:");
        println!("・総手数: {}", game_result.total_moves);
        println!("・総記録数: {} (パス含む)", self.moves.len());

        println!("\n時間分析:");
        println!("・ゲーム時間: {:.2?}", game_result.game_duration);
        println!("・総思考時間: {:.2?}", game_result.total_thinking_time);

        if game_result.total_moves > 0 {
            println!(
                "・1手平均思考時間: {:.2?}",
                game_result.total_thinking_time / game_result.total_moves as u32
            );
        }

        // 思考時間の統計
        let thinking_times: Vec<f64> = self
            .moves
            .iter()
            .filter(|m| m.position.is_some())
            .map(|m| m.thinking_time.as_secs_f64())
            .collect();

        if !thinking_times.is_empty() {
            let max_time = thinking_times.iter().fold(0.0f64, |a, &b| a.max(b));
            let min_time = thinking_times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            println!("・最長思考時間: {:.2}秒", max_time);
            println!("・最短思考時間: {:.2}秒", min_time);
        }

        // 石数の推移
        let disc_history = self.get_disc_count_history();
        if !disc_history.is_empty() {
            let (_, initial_black, initial_white) = disc_history[0];
            let (_, final_black, final_white) = disc_history[disc_history.len() - 1];

            println!("\n石数推移:");
            println!("・開始時: 黒{}個 白{}個", initial_black, initial_white);
            println!("・終了時: 黒{}個 白{}個", final_black, final_white);
            println!(
                "・黒の増減: {:+}個",
                final_black as i32 - initial_black as i32
            );
            println!(
                "・白の増減: {:+}個",
                final_white as i32 - initial_white as i32
            );
        }
    }
}
