use crate::board::BitBoard;
use crate::player::{Entry, NodeType, Player};
use fxhash::FxHashMap;
use rayon::prelude::*;
use std::sync::Arc;

// 置換表の設定を最適化
const MAX_TT_SIZE: usize = 2_000_000; // 適切なサイズに調整
const TT_CLEANUP_THRESHOLD: usize = 1_500_000; // クリーンアップ閾値を調整

// Null Move Pruning は削除（オセロには不適切）

// Late Move Reduction の設定を最適化
const LMR_DEPTH_THRESHOLD: u8 = 3;
const LMR_MOVE_THRESHOLD: usize = 3;

// Aspiration Window を調整
const ASPIRATION_WINDOW: i32 = 50;

// Futility Pruning の設定
const FUTILITY_MARGIN: [i32; 5] = [0, 200, 300, 500, 900];

// 安全な負数演算
#[inline(always)]
fn safe_neg(value: i32) -> i32 {
    if value == i32::MIN {
        i32::MAX
    } else {
        -value
    }
}

// 評価値の定数を最適化
const POSITION_SCORE: [[i32; 8]; 8] = [
    [100, -20, 10, 5, 5, 10, -20, 100],
    [-20, -50, -2, -2, -2, -2, -50, -20],
    [10, -2, -1, -1, -1, -1, -2, 10],
    [5, -2, -1, -1, -1, -1, -2, 5],
    [5, -2, -1, -1, -1, -1, -2, 5],
    [10, -2, -1, -1, -1, -1, -2, 10],
    [-20, -50, -2, -2, -2, -2, -50, -20],
    [100, -20, 10, 5, 5, 10, -20, 100],
];

// ゲーム段階の調整（より適切な閾値）
const EARLY_GAME_THRESHOLD: u32 = 25;
const MID_GAME_THRESHOLD: u32 = 50;

// 評価重みを最適化
const MOBILITY_WEIGHT: [i32; 3] = [25, 15, 8];
const PASS_BONUS: i32 = 30;
const DISC_DIFF_WEIGHT: [i32; 3] = [5, 20, 1000];
const CORNER_WEIGHT: i32 = 300;

// PV (Principal Variation) の管理
#[derive(Clone)]
struct PVTable {
    length: [usize; 64],
    moves: [[u8; 64]; 64],
}

impl PVTable {
    fn new() -> Self {
        Self {
            length: [0; 64],
            moves: [[0; 64]; 64],
        }
    }

    fn update(&mut self, ply: usize, best_move: u8, from_ply: usize) {
        self.moves[ply][0] = best_move;
        for i in 0..self.length[from_ply] {
            self.moves[ply][i + 1] = self.moves[from_ply][i];
        }
        self.length[ply] = self.length[from_ply] + 1;
    }

    fn get_pv_move(&self, ply: usize) -> Option<u8> {
        if self.length[ply] > 0 {
            Some(self.moves[ply][0])
        } else {
            None
        }
    }
}

// Killer Moves の最適化
#[derive(Clone)]
struct KillerMoves {
    moves: [[Option<u8>; 2]; 64],
}

impl KillerMoves {
    fn new() -> Self {
        Self {
            moves: [[None; 2]; 64],
        }
    }

    fn add_killer(&mut self, ply: usize, mv: u8) {
        if ply >= 64 {
            return;
        }

        if self.moves[ply][0] != Some(mv) {
            self.moves[ply][1] = self.moves[ply][0];
            self.moves[ply][0] = Some(mv);
        }
    }

    fn is_killer(&self, ply: usize, mv: u8) -> bool {
        if ply >= 64 {
            return false;
        }
        self.moves[ply][0] == Some(mv) || self.moves[ply][1] == Some(mv)
    }
}

// History Table の最適化
#[derive(Clone)]
struct HistoryTable {
    scores: [[[i32; 64]; 2]; 2], // [phase][player][move]
}

impl HistoryTable {
    fn new() -> Self {
        Self {
            scores: [[[0; 64]; 2]; 2],
        }
    }

    fn update(&mut self, phase: usize, player: usize, mv: u8, depth: u8, is_good: bool) {
        if mv >= 64 || phase >= 2 || player >= 2 {
            return;
        }

        let bonus = (depth as i32) * (depth as i32);
        if is_good {
            self.scores[phase][player][mv as usize] += bonus;
        } else {
            self.scores[phase][player][mv as usize] -= bonus / 2;
        }

        // オーバーフローを防ぐ
        self.scores[phase][player][mv as usize] =
            self.scores[phase][player][mv as usize].clamp(-10000, 10000);
    }

    fn get_score(&self, phase: usize, player: usize, mv: u8) -> i32 {
        if mv >= 64 || phase >= 2 || player >= 2 {
            return 0;
        }
        self.scores[phase][player][mv as usize]
    }

    fn age(&mut self) {
        for phase in 0..2 {
            for player in 0..2 {
                for mv in 0..64 {
                    self.scores[phase][player][mv] = (self.scores[phase][player][mv] * 7) / 8;
                }
            }
        }
    }
}

// 手の情報を格納する構造体
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Move {
    position: u8,
    score: i32,
}

impl Move {
    fn new(position: u8, score: i32) -> Self {
        Self { position, score }
    }
}

impl PartialOrd for Move {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Move {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.score.cmp(&self.score) // 降順ソート
    }
}

// ゲーム段階の定義
#[derive(Debug, Clone, Copy, PartialEq)]
enum GamePhase {
    Early,
    Mid,
    End,
}

impl GamePhase {
    fn from_empty_count(empty_count: u32) -> Self {
        if empty_count > EARLY_GAME_THRESHOLD {
            GamePhase::Early
        } else if empty_count > (64 - MID_GAME_THRESHOLD) {
            GamePhase::Mid
        } else {
            GamePhase::End
        }
    }

    fn index(&self) -> usize {
        match self {
            GamePhase::Early => 0,
            GamePhase::Mid => 0,
            GamePhase::End => 1,
        }
    }
}

impl BitBoard {
    /// Transposition Table を使用した最善手探索のメインエントリーポイント
    pub fn find_best_move_with_tt(
        &mut self,
        player: Player,
        depth: usize,
        tt: &mut FxHashMap<(u64, u64, u8), Entry>,
    ) -> (Option<usize>, Option<i32>) {
        if depth == 0 {
            return (None, None);
        }

        // Transposition Table のサイズ管理
        if tt.len() > TT_CLEANUP_THRESHOLD {
            self.cleanup_tt(tt);
        }

        // 反復深化探索を使用
        self.iterative_deepening_search(player, depth, tt)
    }

    /// 反復深化探索（時間管理付き）
    fn iterative_deepening_search(
        &mut self,
        player: Player,
        max_depth: usize,
        tt: &mut FxHashMap<(u64, u64, u8), Entry>,
    ) -> (Option<usize>, Option<i32>) {
        let mut best_move = None;
        let mut best_eval = None;
        let mut pv_table = PVTable::new();
        let mut killer_moves = KillerMoves::new();
        let mut history_table = HistoryTable::new();

        let start_time = std::time::Instant::now();
        let time_limit = std::time::Duration::from_millis(match max_depth {
            1..=3 => 100,
            4..=6 => 500,
            7..=10 => 2000,
            11..=15 => 5000,
            _ => 10000,
        });

        // 反復深化
        for current_depth in 1..=max_depth {
            if start_time.elapsed() > time_limit && current_depth > 3 {
                break;
            }

            let result = self.aspiration_window_search(
                player,
                current_depth,
                tt,
                &mut pv_table,
                &mut killer_moves,
                &mut history_table,
                best_eval.unwrap_or(0),
            );

            if let Some((mv, eval)) = result {
                best_move = Some(mv);
                best_eval = Some(eval);

                // 時間制限チェック
                if start_time.elapsed() > time_limit {
                    break;
                }
            }
        }

        // History Table の老化
        history_table.age();

        (best_move, best_eval)
    }

    /// Aspiration Window を使った探索
    fn aspiration_window_search(
        &mut self,
        player: Player,
        depth: usize,
        tt: &mut FxHashMap<(u64, u64, u8), Entry>,
        pv_table: &mut PVTable,
        killer_moves: &mut KillerMoves,
        history_table: &mut HistoryTable,
        prev_score: i32,
    ) -> Option<(usize, i32)> {
        if depth <= 3 {
            return self.minimax_best_move_with_tt(
                player,
                depth,
                tt,
                pv_table,
                killer_moves,
                history_table,
            );
        }

        let mut alpha = prev_score - ASPIRATION_WINDOW;
        let mut beta = prev_score + ASPIRATION_WINDOW;
        let mut window_size = ASPIRATION_WINDOW;

        loop {
            pv_table.length[0] = 0; // PV をリセット

            let score = self.minimax_with_tt_internal(
                player,
                depth as u8,
                alpha,
                beta,
                0,
                false,
                tt,
                pv_table,
                killer_moves,
                history_table,
            );

            if score <= alpha {
                // Fail low - alpha を下げる
                alpha = score - window_size;
                window_size *= 2;
            } else if score >= beta {
                // Fail high - beta を上げる
                beta = score + window_size;
                window_size *= 2;
            } else {
                // 正常な範囲内
                if let Some(best_move) = pv_table.get_pv_move(0) {
                    return Some((best_move as usize, score));
                }
                break;
            }

            // ウィンドウが大きくなりすぎたら通常探索に切り替え
            if window_size > 1000 {
                return self.minimax_best_move_with_tt(
                    player,
                    depth,
                    tt,
                    pv_table,
                    killer_moves,
                    history_table,
                );
            }
        }

        None
    }

    /// レベル1用の高速な最善手探索
    fn level1_best_move(&self, player: Player) -> Option<usize> {
        let legal_moves = self.get_legal_moves(player);
        if legal_moves == 0 {
            return None;
        }

        let mut best_move = None;
        let mut best_score = i32::MIN;

        for pos in 0..64 {
            let bit = 1u64 << pos;
            if (legal_moves & bit) != 0 {
                let score = self.evaluate_move_fast(pos, player);
                if score > best_score {
                    best_score = score;
                    best_move = Some(pos);
                }
            }
        }

        best_move
    }

    /// Transposition Table を使った最善手探索
    fn minimax_best_move_with_tt(
        &mut self,
        player: Player,
        depth: usize,
        tt: &mut FxHashMap<(u64, u64, u8), Entry>,
        pv_table: &mut PVTable,
        killer_moves: &mut KillerMoves,
        history_table: &mut HistoryTable,
    ) -> Option<(usize, i32)> {
        if depth == 1 {
            if let Some(pos) = self.level1_best_move(player) {
                return Some((pos, 0));
            }
        }

        let legal_moves = self.get_legal_moves(player);
        if legal_moves == 0 {
            return None;
        }

        pv_table.length[0] = 0;

        let score = self.minimax_with_tt_internal(
            player,
            depth as u8,
            i32::MIN + 1,
            i32::MAX - 1,
            0,
            false,
            tt,
            pv_table,
            killer_moves,
            history_table,
        );

        if let Some(best_move) = pv_table.get_pv_move(0) {
            Some((best_move as usize, score))
        } else {
            None
        }
    }

    /// 内部的な Minimax 実装（高度な最適化版）
    fn minimax_best_move_with_tt_internal(
        &mut self,
        player: Player,
        depth: usize,
        tt: &mut FxHashMap<(u64, u64, u8), Entry>,
        pv_table: &mut PVTable,
        killer_moves: &mut KillerMoves,
        history_table: &mut HistoryTable,
    ) -> Option<(usize, i32)> {
        if depth >= 8 {
            // 並列探索を使用
            self.parallel_search(player, depth, tt, pv_table, killer_moves, history_table)
        } else {
            // 逐次探索を使用
            self.sequential_search(player, depth, tt, pv_table, killer_moves, history_table)
        }
    }

    /// 並列探索の実装（簡略化）
    fn parallel_search(
        &mut self,
        player: Player,
        depth: usize,
        tt: &mut FxHashMap<(u64, u64, u8), Entry>,
        pv_table: &mut PVTable,
        killer_moves: &mut KillerMoves,
        history_table: &mut HistoryTable,
    ) -> Option<(usize, i32)> {
        // 深い探索でも通常の逐次探索を使用（並列処理のオーバーヘッドを避ける）
        self.sequential_search(player, depth, tt, pv_table, killer_moves, history_table)
    }

    /// 逐次探索の実装
    fn sequential_search(
        &mut self,
        player: Player,
        depth: usize,
        tt: &mut FxHashMap<(u64, u64, u8), Entry>,
        pv_table: &mut PVTable,
        killer_moves: &mut KillerMoves,
        history_table: &mut HistoryTable,
    ) -> Option<(usize, i32)> {
        let legal_moves = self.get_legal_moves(player);
        if legal_moves == 0 {
            return None;
        }

        pv_table.length[0] = 0;

        let best_score = self.minimax_with_tt_internal(
            player,
            depth as u8,
            i32::MIN + 1,
            i32::MAX - 1,
            0,
            false,
            tt,
            pv_table,
            killer_moves,
            history_table,
        );

        if let Some(best_move) = pv_table.get_pv_move(0) {
            Some((best_move as usize, best_score))
        } else {
            None
        }
    }

    /// 手の並び替え（高度な最適化版）
    fn order_moves(
        &self,
        legal_moves: u64,
        player: Player,
        ply: usize,
        pv_table: &PVTable,
        killer_moves: &KillerMoves,
        history_table: &HistoryTable,
    ) -> Vec<Move> {
        let mut moves = Vec::new();
        let phase = GamePhase::from_empty_count(64 - (self.black | self.white).count_ones());
        let phase_idx = phase.index();
        let player_idx = match player {
            Player::Black => 0,
            Player::White => 1,
        };

        for pos in 0..64 {
            let bit = 1u64 << pos;
            if (legal_moves & bit) == 0 {
                continue;
            }

            let mut score = 0;

            // PV move が最優先
            if let Some(pv_move) = pv_table.get_pv_move(ply) {
                if pv_move == pos as u8 {
                    score += 10000;
                }
            }

            // Killer moves
            if killer_moves.is_killer(ply, pos as u8) {
                score += 5000;
            }

            // History heuristic
            score += history_table.get_score(phase_idx, player_idx, pos as u8);

            // 位置の価値
            let row = pos / 8;
            let col = pos % 8;
            score += POSITION_SCORE[row][col];

            // 角の特別ボーナス
            if pos == 0 || pos == 7 || pos == 56 || pos == 63 {
                score += CORNER_WEIGHT;
            }

            // モビリティの評価
            let flips = self.compute_flips(pos, player);
            score += flips.count_ones() as i32 * 10;

            moves.push(Move::new(pos as u8, score));
        }

        moves.sort_unstable();
        moves
    }

    /// Minimax アルゴリズムの内部実装（最適化版）
    fn minimax_with_tt_internal(
        &mut self,
        player: Player,
        depth: u8,
        mut alpha: i32,
        beta: i32,
        ply: usize,
        null_move: bool,
        tt: &mut FxHashMap<(u64, u64, u8), Entry>,
        pv_table: &mut PVTable,
        killer_moves: &mut KillerMoves,
        history_table: &mut HistoryTable,
    ) -> i32 {
        let original_alpha = alpha;
        pv_table.length[ply] = 0;

        // Transposition Table の確認
        let tt_key = (self.black, self.white, player as u8);
        if let Some(entry) = tt.get(&tt_key) {
            if entry.depth >= depth {
                match entry.flag {
                    NodeType::Exact => return entry.score,
                    NodeType::LowerBound => {
                        if entry.score >= beta {
                            return entry.score;
                        }
                        alpha = alpha.max(entry.score);
                    }
                    NodeType::UpperBound => {
                        if entry.score <= alpha {
                            return entry.score;
                        }
                    }
                }
            }
        }

        // 終端ノード
        if depth == 0 {
            let score = self.evaluate_board_optimized(player);
            tt.insert(
                tt_key,
                Entry {
                    score,
                    depth,
                    flag: NodeType::Exact,
                    best_move: None,
                },
            );
            return score;
        }

        // ゲーム終了チェック
        if self.is_game_over() {
            let score = self.evaluate_game_end(player);
            tt.insert(
                tt_key,
                Entry {
                    score,
                    depth,
                    flag: NodeType::Exact,
                    best_move: None,
                },
            );
            return score;
        }

        let legal_moves = self.get_legal_moves(player);

        // パスの処理
        if legal_moves == 0 {
            let mut pass_board = *self;
            let score = safe_neg(pass_board.minimax_with_tt_internal(
                player.opponent(),
                depth - 1,
                safe_neg(beta),
                safe_neg(alpha),
                ply + 1,
                false,
                tt,
                pv_table,
                killer_moves,
                history_table,
            ));

            tt.insert(
                tt_key,
                Entry {
                    score,
                    depth,
                    flag: NodeType::Exact,
                    best_move: None,
                },
            );

            return score;
        }

        // Null Move Pruning は削除（オセロには適用不可）

        // Futility Pruning
        let futility_prune = depth < 5 && !self.is_endgame();
        let static_eval = if futility_prune {
            self.evaluate_board_optimized(player)
        } else {
            0
        };

        let moves = self.order_moves(
            legal_moves,
            player,
            ply,
            pv_table,
            killer_moves,
            history_table,
        );
        let mut best_score = i32::MIN;
        let mut best_move = None;
        let phase = GamePhase::from_empty_count(64 - (self.black | self.white).count_ones());
        let phase_idx = phase.index();
        let player_idx = match player {
            Player::Black => 0,
            Player::White => 1,
        };

        for (move_count, &mv) in moves.iter().enumerate() {
            let pos = mv.position as usize;

            // Futility Pruning
            if futility_prune && move_count > 0 {
                if static_eval + FUTILITY_MARGIN[depth as usize] <= alpha {
                    continue;
                }
            }

            let mut new_board = *self;
            if !new_board.make_move(pos, player) {
                continue;
            }

            let mut score;

            // PVS (Principal Variation Search)
            if move_count == 0 {
                // 最初の手は full window で探索
                score = safe_neg(new_board.minimax_with_tt_internal(
                    player.opponent(),
                    depth - 1,
                    safe_neg(beta),
                    safe_neg(alpha),
                    ply + 1,
                    false,
                    tt,
                    pv_table,
                    killer_moves,
                    history_table,
                ));
            } else {
                // Late Move Reduction
                let reduction = if depth >= LMR_DEPTH_THRESHOLD
                    && move_count >= LMR_MOVE_THRESHOLD
                    && !killer_moves.is_killer(ply, mv.position)
                {
                    1
                } else {
                    0
                };

                let search_depth = depth.saturating_sub(1 + reduction);

                // Null window で探索
                score = safe_neg(new_board.minimax_with_tt_internal(
                    player.opponent(),
                    search_depth,
                    safe_neg(alpha) - 1,
                    safe_neg(alpha),
                    ply + 1,
                    false,
                    tt,
                    pv_table,
                    killer_moves,
                    history_table,
                ));

                // Re-search が必要な場合
                if score > alpha && (reduction > 0 || score < beta) {
                    score = safe_neg(new_board.minimax_with_tt_internal(
                        player.opponent(),
                        depth - 1,
                        safe_neg(beta),
                        safe_neg(alpha),
                        ply + 1,
                        false,
                        tt,
                        pv_table,
                        killer_moves,
                        history_table,
                    ));
                }
            }

            if score > best_score {
                best_score = score;
                best_move = Some(mv.position);

                // PV の更新
                pv_table.update(ply, mv.position, ply + 1);

                if score > alpha {
                    alpha = score;

                    // History heuristic の更新
                    history_table.update(phase_idx, player_idx, mv.position, depth, true);

                    if score >= beta {
                        // Killer move の追加
                        killer_moves.add_killer(ply, mv.position);

                        // 残りの手の history を減点
                        for &remaining_move in moves.iter().skip(move_count + 1) {
                            history_table.update(
                                phase_idx,
                                player_idx,
                                remaining_move.position,
                                depth,
                                false,
                            );
                        }

                        break; // Beta cutoff
                    }
                }
            }
        }

        // Transposition Table への保存
        let flag = if best_score <= original_alpha {
            NodeType::UpperBound
        } else if best_score >= beta {
            NodeType::LowerBound
        } else {
            NodeType::Exact
        };

        tt.insert(
            tt_key,
            Entry {
                score: best_score,
                depth,
                flag,
                best_move,
            },
        );

        best_score
    }

    /// 終盤判定の最適化
    #[inline(always)]
    fn is_endgame(&self) -> bool {
        (self.black | self.white).count_ones() >= 55
    }

    /// Transposition Table のクリーンアップ（改良版）
    fn cleanup_tt(&self, tt: &mut FxHashMap<(u64, u64, u8), Entry>) {
        if tt.len() <= MAX_TT_SIZE {
            return;
        }

        // より効率的なクリーンアップ：深度の低いエントリから削除
        let target_size = MAX_TT_SIZE * 3 / 4;
        let mut to_remove = Vec::new();

        for (key, entry) in tt.iter() {
            if entry.depth <= 2 {
                to_remove.push(*key);
                if to_remove.len() + target_size >= tt.len() {
                    break;
                }
            }
        }

        for key in to_remove {
            tt.remove(&key);
        }
    }

    /// 最適化された盤面評価関数
    fn evaluate_board_optimized(&self, player: Player) -> i32 {
        let empty_count = 64 - (self.black | self.white).count_ones();
        let phase = GamePhase::from_empty_count(empty_count);

        // ゲーム終了チェックを慎重に
        let black_legal = self.get_legal_moves(Player::Black);
        let white_legal = self.get_legal_moves(Player::White);

        if black_legal == 0 && white_legal == 0 {
            return self.evaluate_game_end(player);
        }

        let mut score = 0;

        match phase {
            GamePhase::Early => {
                // 序盤はモビリティと位置を重視、石数差は控えめ
                score += self.evaluate_mobility(player) * MOBILITY_WEIGHT[0];
                score += self.evaluate_position_value(player);
                score += self.evaluate_disc_count(player) * DISC_DIFF_WEIGHT[0];
            }
            GamePhase::Mid => {
                // 中盤はバランス重視
                score += self.evaluate_mobility(player) * MOBILITY_WEIGHT[1];
                score += self.evaluate_position_value(player);
                score += self.evaluate_corners_optimized(player);
                score += self.evaluate_stability(player);
                score += self.evaluate_disc_count(player) * DISC_DIFF_WEIGHT[1];
            }
            GamePhase::End => {
                // 終盤は石数と確定石を重視
                score += self.evaluate_disc_count(player) * DISC_DIFF_WEIGHT[2];
                score += self.evaluate_corners_optimized(player);
                score += self.evaluate_stability(player) * 2;
                score += self.evaluate_parity(player);
                score += self.evaluate_mobility(player) * MOBILITY_WEIGHT[2];
            }
        }

        score
    }

    /// ゲーム終了時の評価
    #[inline]
    fn evaluate_game_end(&self, player: Player) -> i32 {
        let black_count = self.black.count_ones() as i32;
        let white_count = self.white.count_ones() as i32;
        let total_discs = black_count + white_count;

        // 序盤の調整を削除（実際のオセロでは石が10個未満になることは稀）

        let diff = match player {
            Player::Black => black_count - white_count,
            Player::White => white_count - black_count,
        };

        if diff > 0 {
            10000 + diff
        } else if diff < 0 {
            -10000 + diff
        } else {
            0
        }
    }

    /// モビリティ評価の最適化
    #[inline]
    fn evaluate_mobility(&self, player: Player) -> i32 {
        let my_moves = self.get_legal_moves(player).count_ones() as i32;
        let opp_moves = self.get_legal_moves(player.opponent()).count_ones() as i32;

        let mobility_diff = my_moves - opp_moves;

        // パスを強制する場合のボーナス
        if opp_moves == 0 && my_moves > 0 {
            mobility_diff + PASS_BONUS
        } else if my_moves == 0 && opp_moves > 0 {
            // 自分がパスする場合のペナルティ
            mobility_diff - PASS_BONUS
        } else {
            mobility_diff
        }
    }

    /// 位置価値の評価
    #[inline]
    fn evaluate_position_value(&self, player: Player) -> i32 {
        let mut score = 0;
        let (my_board, opp_board) = match player {
            Player::Black => (self.black, self.white),
            Player::White => (self.white, self.black),
        };

        for pos in 0..64 {
            let bit = 1u64 << pos;
            let row = pos / 8;
            let col = pos % 8;
            let value = POSITION_SCORE[row][col];

            if (my_board & bit) != 0 {
                score += value;
            } else if (opp_board & bit) != 0 {
                score -= value;
            }
        }

        score
    }

    /// 石数差の評価
    #[inline]
    fn evaluate_disc_count(&self, player: Player) -> i32 {
        let my_count = self.count_discs(player) as i32;
        let opp_count = self.count_discs(player.opponent()) as i32;
        my_count - opp_count
    }

    /// 角の評価の最適化
    fn evaluate_corners_optimized(&self, player: Player) -> i32 {
        const CORNERS: [usize; 4] = [0, 7, 56, 63];
        let mut score = 0;

        for &corner in &CORNERS {
            let bit = 1u64 << corner;
            if (self.black & bit) != 0 {
                score += if player == Player::Black {
                    CORNER_WEIGHT
                } else {
                    -CORNER_WEIGHT
                };
            } else if (self.white & bit) != 0 {
                score += if player == Player::White {
                    CORNER_WEIGHT
                } else {
                    -CORNER_WEIGHT
                };
            }
        }

        score
    }

    /// 確定石の評価
    fn evaluate_stability(&self, player: Player) -> i32 {
        let my_stable = self.compute_stable_discs(player);
        let opp_stable = self.compute_stable_discs(player.opponent());

        (my_stable.count_ones() as i32) - (opp_stable.count_ones() as i32)
    }

    /// 確定石の計算（簡略版）
    fn compute_stable_discs(&self, player: Player) -> u64 {
        let my_board = match player {
            Player::Black => self.black,
            Player::White => self.white,
        };

        let occupied = self.black | self.white;
        let mut stable = 0u64;

        // 角の確定石
        const CORNERS: [usize; 4] = [0, 7, 56, 63];
        for &corner in &CORNERS {
            let bit = 1u64 << corner;
            if (my_board & bit) != 0 {
                stable |= bit;
                stable = self.expand_stability(stable, my_board, occupied);
            }
        }

        stable
    }

    /// 確定石の拡張
    fn expand_stability(&self, mut stable: u64, my_board: u64, occupied: u64) -> u64 {
        let mut changed = true;

        while changed {
            changed = false;
            let _old_stable = stable;

            // 8方向をチェックして、両端が確定または盤面端の石を確定石とする
            for pos in 0..64 {
                let bit = 1u64 << pos;
                if (my_board & bit) != 0 && (stable & bit) == 0 {
                    let row = pos / 8;
                    let col = pos % 8;

                    let mut is_stable = true;

                    // 水平、垂直、対角線方向をチェック
                    let directions = [
                        (0, 1),
                        (0, -1), // 水平
                        (1, 0),
                        (-1, 0), // 垂直
                        (1, 1),
                        (-1, -1), // 対角線1
                        (1, -1),
                        (-1, 1), // 対角線2
                    ];

                    for chunk in directions.chunks(2) {
                        let (dr1, dc1) = chunk[0];
                        let (dr2, dc2) = chunk[1];

                        let stable1 = self.check_direction_stability(
                            row as i32, col as i32, dr1, dc1, my_board, stable, occupied,
                        );
                        let stable2 = self.check_direction_stability(
                            row as i32, col as i32, dr2, dc2, my_board, stable, occupied,
                        );

                        if !stable1 && !stable2 {
                            is_stable = false;
                            break;
                        }
                    }

                    if is_stable {
                        stable |= bit;
                        changed = true;
                    }
                }
            }
        }

        stable
    }

    /// 方向の安定性をチェック
    fn check_direction_stability(
        &self,
        row: i32,
        col: i32,
        dr: i32,
        dc: i32,
        my_board: u64,
        stable: u64,
        _occupied: u64,
    ) -> bool {
        let mut r = row + dr;
        let mut c = col + dc;

        while r >= 0 && r < 8 && c >= 0 && c < 8 {
            let pos = (r * 8 + c) as usize;
            let bit = 1u64 << pos;

            if (stable & bit) != 0 {
                return true; // 確定石に到達
            }

            if (my_board & bit) == 0 {
                return false; // 自分の石でない
            }

            r += dr;
            c += dc;
        }

        true // 盤面端に到達
    }

    /// パリティの評価
    fn evaluate_parity(&self, player: Player) -> i32 {
        let empty_count = 64 - (self.black | self.white).count_ones();

        if empty_count % 2 == 0 {
            // 偶数なら後手有利
            if player == Player::White {
                10
            } else {
                -10
            }
        } else {
            // 奇数なら先手有利
            if player == Player::Black {
                10
            } else {
                -10
            }
        }
    }

    /// 高速な手の評価（レベル1用）
    fn evaluate_move_fast(&self, pos: usize, player: Player) -> i32 {
        let row = pos / 8;
        let col = pos % 8;
        let mut score = POSITION_SCORE[row][col];

        // 角のボーナス
        if pos == 0 || pos == 7 || pos == 56 || pos == 63 {
            score += CORNER_WEIGHT;
        }

        // ひっくり返す石の数
        let flips = self.compute_flips(pos, player);
        score += flips.count_ones() as i32 * 10;

        score
    }

    /// 通常の手の評価（レガシー）
    pub fn evaluate_move(&self, _pos: usize, _player: Player) -> i32 {
        0 // 現在は使用していない
    }
}
