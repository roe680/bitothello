use crate::board::BitBoard;
use crate::player::Player;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Mutex;

// 評価関数の重み付け定数
// 角の価値は非常に高い
const POSITION_SCORE: [i32; 64] = [
    500, -150, 30, 10, 10, 30, -150, 500, -150, -250, -15, -5, -5, -15, -250, -150, 30, -15, 15, 3,
    3, 15, -15, 30, 10, -5, 3, 3, 3, 3, -5, 10, 10, -5, 3, 3, 3, 3, -5, 10, 30, -15, 15, 3, 3, 15,
    -15, 30, -150, -250, -15, -5, -5, -15, -250, -150, 500, -150, 30, 10, 10, 30, -150, 500,
];

// 機動力（合法手の数）の重み
const MOBILITY_WEIGHT: i32 = 10;

// 相手にパスを強いる手のボーナス
const PASS_BONUS: i32 = 100;

// 石の枚数差の重み（終盤で重要になる）
const DISC_DIFF_WEIGHT: i32 = 1;

/// トランスポジションテーブルのキー
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BoardKey {
    black: u64,
    white: u64,
    player: Player,
    depth: usize,
}

/// トランスポジションテーブルのエントリ
#[derive(Debug, Clone)]
struct TableEntry {
    score: i32,
    depth: usize,
}

/// 手の候補
#[derive(Debug, Clone)]
struct Move {
    position: usize,
    score: i32,
}

impl Move {
    fn new(position: usize, score: i32) -> Self {
        Self { position, score }
    }
}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for Move {}

impl PartialOrd for Move {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Move {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl BitBoard {
    /// 最善手を探す
    /// level: 探索の深さ（1ならレベル1のアルゴリズム、それ以上はミニマックス法の探索深度）
    pub fn find_best_move(&self, player: Player, level: usize) -> Option<usize> {
        if level <= 1 {
            self.level1_best_move(player)
        } else {
            self.minimax_best_move(player, level)
        }
    }

    /// レベル1のAI：ひっくり返す石が最も多い手を選ぶ
    fn level1_best_move(&self, player: Player) -> Option<usize> {
        let legal_moves = self.get_legal_moves(player);
        if legal_moves == 0 {
            return None;
        }

        let mut best_score = 0;
        let mut best_moves = Vec::new();

        for pos in 0..64 {
            let pos_bit = 1u64 << pos;
            if (legal_moves & pos_bit) != 0 {
                let flips = self.compute_flips(pos, player);
                let score = flips.count_ones();

                if score > best_score {
                    best_score = score;
                    best_moves.clear();
                    best_moves.push(pos);
                } else if score == best_score {
                    best_moves.push(pos);
                }
            }
        }

        let mut rng = thread_rng();
        best_moves.choose(&mut rng).copied()
    }

    /// ミニマックス法による最善手の探索（トランスポジションテーブル付き）
    fn minimax_best_move(&self, player: Player, depth: usize) -> Option<usize> {
        let legal_moves = self.get_legal_moves(player);
        if legal_moves == 0 {
            return None;
        }

        // トランスポジションテーブルを初期化
        let tt = Mutex::new(HashMap::<BoardKey, TableEntry>::with_capacity(1_000_000));

        let mut best_moves = Vec::new();
        let mut best_score = i32::MIN;

        // 終盤であれば探索深度を増やす（空きマスが少ない場合）
        let empty_count = 64 - (self.black | self.white).count_ones() as usize;
        let adaptive_depth = if empty_count <= 10 && depth >= 5 {
            depth + 2 // 終盤は探索深度を増やす
        } else {
            depth
        };

        // 有効な手の評価値を計算してソート（Move Ordering）
        let mut moves = Vec::new();
        for pos in 0..64 {
            let pos_bit = 1u64 << pos;
            if (legal_moves & pos_bit) != 0 {
                let score = self.evaluate_move(pos, player);
                moves.push(Move::new(pos, score));
            }
        }
        moves.sort_by(|a, b| b.score.cmp(&a.score)); // 降順ソート

        // 並列化して探索
        let results: Vec<(i32, usize)> = moves
            .par_iter()
            .map(|m| {
                let pos = m.position;
                let mut board_clone = self.clone();
                if board_clone.make_move(pos, player) {
                    let score = board_clone.minimax_with_tt(
                        adaptive_depth - 1,
                        i32::MIN,
                        i32::MAX,
                        player.opponent(),
                        player,
                        &tt,
                    );
                    (score, pos)
                } else {
                    (i32::MIN, pos)
                }
            })
            .collect();

        // 最良スコアの手を集める
        for (score, pos) in results {
            if score > best_score {
                best_score = score;
                best_moves.clear();
                best_moves.push(pos);
            } else if score == best_score {
                best_moves.push(pos);
            }
        }

        let mut rng = thread_rng();
        best_moves.choose(&mut rng).copied()
    }

    /// 古いミニマックス関数（互換性のために残す）
    fn minimax(
        &self,
        depth: usize,
        alpha: i32,
        beta: i32,
        current_player: Player,
        original_player: Player,
    ) -> i32 {
        let tt = Mutex::new(HashMap::<BoardKey, TableEntry>::with_capacity(1000));
        self.minimax_with_tt(depth, alpha, beta, current_player, original_player, &tt)
    }

    /// トランスポジションテーブル付きミニマックス法＋アルファベータ枝刈り
    fn minimax_with_tt(
        &self,
        depth: usize,
        mut alpha: i32,
        mut beta: i32,
        current_player: Player,
        original_player: Player,
        tt: &Mutex<HashMap<BoardKey, TableEntry>>,
    ) -> i32 {
        // 終了条件
        if depth == 0 || self.is_game_over() {
            return self.evaluate_board(original_player);
        }

        // トランスポジションテーブルをチェック
        let key = BoardKey {
            black: self.black,
            white: self.white,
            player: current_player,
            depth,
        };

        // テーブルヒットを確認（スレッドセーフに）
        let tt_lookup = tt.lock().unwrap().get(&key).cloned();
        if let Some(entry) = tt_lookup {
            if entry.depth >= depth {
                return entry.score;
            }
        }

        let legal_moves = self.get_legal_moves(current_player);

        // パスが必要な場合
        if legal_moves == 0 {
            // 相手もパスが必要ならゲーム終了
            if self.get_legal_moves(current_player.opponent()) == 0 {
                return self.evaluate_board(original_player);
            }
            // パスして相手の手番
            return self.minimax_with_tt(
                depth,
                -beta,
                -alpha,
                current_player.opponent(),
                original_player,
                tt,
            );
        }

        let is_maximizing = current_player == original_player;
        let mut best_score;
        let mut moves = Vec::new();

        // 手を列挙して事前評価する（Move Ordering）
        for pos in 0..64 {
            let pos_bit = 1u64 << pos;
            if (legal_moves & pos_bit) != 0 {
                let score = self.evaluate_move(pos, current_player);
                moves.push(Move::new(pos, score));
            }
        }

        // ソート（最大化と最小化で順序を変える）
        if is_maximizing {
            moves.sort_by(|a, b| b.score.cmp(&a.score)); // 降順
            best_score = i32::MIN;
        } else {
            moves.sort_by(|a, b| a.score.cmp(&b.score)); // 昇順
            best_score = i32::MAX;
        }

        // 手を試す
        for m in &moves {
            let pos = m.position;
            let mut board_clone = self.clone();

            if board_clone.make_move(pos, current_player) {
                let score = board_clone.minimax_with_tt(
                    depth - 1,
                    -beta,
                    -alpha,
                    current_player.opponent(),
                    original_player,
                    tt,
                );

                // ネガマックス形式で評価
                let negated_score = -score;

                if is_maximizing {
                    best_score = best_score.max(negated_score);
                    alpha = alpha.max(negated_score);
                } else {
                    best_score = best_score.min(negated_score);
                    beta = beta.min(negated_score);
                }

                // アルファベータカット
                if beta <= alpha {
                    break;
                }
            }
        }

        // 結果をトランスポジションテーブルに保存
        tt.lock().unwrap().insert(
            key,
            TableEntry {
                score: best_score,
                depth,
            },
        );

        best_score
    }

    /// ボードの評価関数（最適化版）
    fn evaluate_board(&self, player: Player) -> i32 {
        let opponent = player.opponent();

        // ゲームが終了している場合は石の数で評価（より明確な勝敗判定）
        if self.is_game_over() {
            let my_discs = self.count_discs(player);
            let opp_discs = self.count_discs(opponent);

            if my_discs > opp_discs {
                return 100000 + (my_discs - opp_discs) as i32; // より詳細な勝ち評価
            } else if my_discs < opp_discs {
                return -100000 - (opp_discs - my_discs) as i32; // より詳細な負け評価
            } else {
                return 0; // 引き分け
            }
        }

        // 空きマスの数に基づいてゲームフェーズを判断
        let empty_count = 64 - (self.black | self.white).count_ones() as i32;
        let is_endgame = empty_count < 12;

        // 1. 位置評価（最適化版）
        let (my_bits, opp_bits) = match player {
            Player::Black => (self.black, self.white),
            Player::White => (self.white, self.black),
        };

        let mut position_score = 0;

        // 自分の石の位置評価
        let mut bits = my_bits;
        while bits != 0 {
            let lsb = bits & (!bits + 1);
            let pos = lsb.trailing_zeros() as usize;
            position_score += POSITION_SCORE[pos];
            bits &= bits - 1; // 最下位ビットをクリア
        }

        // 相手の石の位置評価
        bits = opp_bits;
        while bits != 0 {
            let lsb = bits & (!bits + 1);
            let pos = lsb.trailing_zeros() as usize;
            position_score -= POSITION_SCORE[pos];
            bits &= bits - 1; // 最下位ビットをクリア
        }

        // 2. 機動力（合法手の数）- 終盤では重要度が下がる
        let mobility_weight = if is_endgame {
            MOBILITY_WEIGHT / 2
        } else {
            MOBILITY_WEIGHT
        };
        let my_mobility = self.get_legal_moves(player).count_ones() as i32;
        let opp_mobility = self.get_legal_moves(opponent).count_ones() as i32;
        let mobility_score = (my_mobility - opp_mobility) * mobility_weight;

        // 3. パスを強いる手へのボーナス（終盤でより重要）
        let pass_bonus = if is_endgame {
            PASS_BONUS * 2
        } else {
            PASS_BONUS
        };
        let pass_score = if opp_mobility == 0 && my_mobility > 0 {
            pass_bonus
        } else {
            0
        };

        // 4. 石数の差（終盤で重要になる）
        let disc_weight = if is_endgame {
            DISC_DIFF_WEIGHT * 3
        } else {
            DISC_DIFF_WEIGHT
        };
        let my_discs = self.count_discs(player) as i32;
        let opp_discs = self.count_discs(opponent) as i32;
        let disc_diff = (my_discs - opp_discs) * disc_weight;

        // 5. 角の確保（非常に重要）
        let corner_score = self.evaluate_corners(player) * 50;

        // 総合評価
        position_score + mobility_score + pass_score + disc_diff + corner_score
    }

    /// 角の確保状況を評価
    fn evaluate_corners(&self, player: Player) -> i32 {
        let corners = [0, 7, 56, 63]; // 四隅の位置
        let (my_bits, opp_bits) = match player {
            Player::Black => (self.black, self.white),
            Player::White => (self.white, self.black),
        };

        let mut score = 0;
        for &corner in &corners {
            let corner_bit = 1u64 << corner;
            if (my_bits & corner_bit) != 0 {
                score += 1;
            } else if (opp_bits & corner_bit) != 0 {
                score -= 1;
            }
        }

        score
    }

    /// 特定の手の評価値を計算（Move Ordering用）- 最適化版
    fn evaluate_move(&self, pos: usize, player: Player) -> i32 {
        // 位置の価値
        let position_value = POSITION_SCORE[pos];

        // ひっくり返す石の数
        let flips = self.compute_flips(pos, player);
        let flips_count = flips.count_ones() as i32;

        // 角かどうか（最も重要）
        let corner_bonus = match pos {
            0 | 7 | 56 | 63 => 1000, // 角の価値を大幅に上げる
            // 角の隣のマスは危険なので大幅にペナルティ
            1 | 8 | 6 | 15 | 48 | 57 | 55 | 62 => -500,
            // 角の斜め隣も危険
            9 | 14 | 49 | 54 => -300,
            _ => 0,
        };

        // 安定石になるかどうかをチェック
        // ここでは完全なチェックではなく簡易版
        let stable_bonus = if self.is_potentially_stable(pos, player) {
            200
        } else {
            0
        };

        // 合法手の減少/増加を評価
        let mut board_clone = self.clone();
        if !board_clone.make_move(pos, player) {
            return i32::MIN; // 無効な手
        }

        // 相手の合法手の数
        let opponent = player.opponent();
        let opponent_moves = board_clone.get_legal_moves(opponent);
        let opponent_moves_count = opponent_moves.count_ones() as i32;

        // 相手がパスするか
        let pass_bonus = if opponent_moves == 0 {
            PASS_BONUS * 2
        } else {
            0
        };

        // 次の自分の合法手の数
        let my_next_moves_count = if opponent_moves == 0 {
            // パスの場合、自分がまた打てる手の数
            board_clone.get_legal_moves(player).count_ones() as i32
        } else {
            0
        };

        // 総合評価
        position_value
            + flips_count * 5
            + pass_bonus
            + corner_bonus
            + stable_bonus
            + my_next_moves_count * 10
            - opponent_moves_count * 5
    }

    /// そのマスが潜在的に安定石になるかを評価（簡易版）
    fn is_potentially_stable(&self, pos: usize, _player: Player) -> bool {
        // 角は常に安定
        if pos == 0 || pos == 7 || pos == 56 || pos == 63 {
            return true;
        }

        // 端のマスは比較的安定になる可能性が高い
        let row = pos / 8;
        let col = pos % 8;

        if row == 0 || row == 7 || col == 0 || col == 7 {
            return true;
        }

        // それ以外は不安定とみなす
        false
    }
}
