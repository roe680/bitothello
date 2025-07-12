use crate::player::Player;
use std::fmt;

const DEFAULT_BLACK: u64 = 0x0000000810000000; // 初期配置の黒石
const DEFAULT_WHITE: u64 = 0x0000001008000000; // 初期配置の白石

#[derive(Copy, Clone, Debug)]
pub struct BitBoard {
    pub black: u64,
    pub white: u64,
}

impl BitBoard {
    // 各方向でのシフト量とマスク (shift, mask, is_forward)
    // シフト量: ビットシフト量
    // マスク: 盤面の端からのはみ出しを防ぐマスク
    // is_forward: trueなら左上から右下へ（<<）、falseなら右下から左上へ（>>）
    const SHIFTS: [(u32, u64, bool); 8] = [
        (1, 0x7f7f7f7f7f7f7f7f, false),  // 左
        (1, 0xfefefefefefefefe, true),   // 右
        (8, 0xffffffffffffff00, false),  // 上
        (8, 0x00ffffffffffffff, true),   // 下
        (9, 0x7f7f7f7f7f7f7f00, false),  // 左上
        (7, 0xfefefefefefefe00, false),  // 右上
        (7, 0x007f7f7f7f7f7f7f, true),   // 左下
        (9, 0x00fefefefefefefefe, true), // 右下
    ];

    // キャッシュ用の定数
    const CORNER_MASK: u64 = 0x8100000000000081; // 角のマスク
    const EDGE_MASK: u64 = 0xFF818181818181FF; // 辺のマスク

    /// 新しいビットボードを初期配置で作成
    pub fn new() -> Self {
        BitBoard {
            black: DEFAULT_BLACK,
            white: DEFAULT_WHITE,
        }
    }

    /// 指定位置にビットを設定する
    #[inline(always)]
    fn set_bit(&mut self, pos: usize, player: Player) {
        debug_assert!(pos < 64, "ビット位置が範囲外です");
        let bit = 1u64 << pos;

        match player {
            Player::Black => {
                self.black |= bit;
                self.white &= !bit;
            }
            Player::White => {
                self.white |= bit;
                self.black &= !bit;
            }
        }
    }

    /// 複数の石を一度にひっくり返す
    #[inline(always)]
    fn flip_bits(&mut self, bits: u64, player: Player) {
        match player {
            Player::Black => {
                self.black |= bits; // 黒石を置く
                self.white &= !bits; // 白石を取り除く
            }
            Player::White => {
                self.white |= bits; // 白石を置く
                self.black &= !bits; // 黒石を取り除く
            }
        }
    }

    /// 石を置いてひっくり返す（最適化版）
    #[inline(always)]
    pub fn make_move(&mut self, pos: usize, player: Player) -> bool {
        debug_assert!(pos < 64, "ビット位置が範囲外です");

        let pos_bit = 1u64 << pos;

        // 既に石が置かれているかチェック
        if (self.black | self.white) & pos_bit != 0 {
            return false;
        }

        let flips = self.compute_flips(pos, player);

        // ひっくり返せる石がなければ不正な手
        if flips == 0 {
            return false;
        }

        // 石を置き、ひっくり返す（ビット演算のみで高速化）
        match player {
            Player::Black => {
                self.black |= pos_bit | flips;
                self.white &= !flips;
            }
            Player::White => {
                self.white |= pos_bit | flips;
                self.black &= !flips;
            }
        }

        true
    }

    /// ひっくり返し計算（修正版）
    #[inline(always)]
    pub fn compute_flips(&self, pos: usize, player: Player) -> u64 {
        let (my, opp) = match player {
            Player::Black => (self.black, self.white),
            Player::White => (self.white, self.black),
        };

        let mut flips = 0u64;
        let row = pos / 8;
        let col = pos % 8;

        // 8方向をチェック
        let directions = [
            (-1, -1),
            (-1, 0),
            (-1, 1), // 上左、上、上右
            (0, -1),
            (0, 1), // 左、右
            (1, -1),
            (1, 0),
            (1, 1), // 下左、下、下右
        ];

        for &(dr, dc) in &directions {
            let mut direction_flips = 0u64;
            let mut found_opponent = false;
            let mut r = row as i32 + dr;
            let mut c = col as i32 + dc;

            while r >= 0 && r < 8 && c >= 0 && c < 8 {
                let current_pos = (r * 8 + c) as usize;
                let current_bit = 1u64 << current_pos;

                if (opp & current_bit) != 0 {
                    // 相手の石を発見
                    direction_flips |= current_bit;
                    found_opponent = true;
                } else if (my & current_bit) != 0 {
                    // 自分の石を発見
                    if found_opponent {
                        flips |= direction_flips; // この方向の石をひっくり返す
                    }
                    break;
                } else {
                    // 空きマス
                    break;
                }

                r += dr;
                c += dc;
            }
        }

        flips
    }

    /// 合法手かどうかをチェック（最適化版）
    #[inline(always)]
    pub fn is_legal_move(&self, pos: usize, player: Player) -> bool {
        debug_assert!(pos < 64, "ビット位置が範囲外です");

        // すでに石が置かれていたら不正
        let pos_bit = 1u64 << pos;
        if (self.black | self.white) & pos_bit != 0 {
            return false;
        }

        // 事前計算を活用して高速判定
        // - 隣接する相手の石がなければ不正
        let (_, opp) = match player {
            Player::Black => (self.black, self.white),
            Player::White => (self.white, self.black),
        };

        // 周囲8方向に相手の石があるかチェック（一度に計算）
        let adjacent_mask = self.get_adjacent_mask(pos);
        if adjacent_mask & opp == 0 {
            return false;
        }

        // 実際にひっくり返せるか詳細チェック
        self.compute_flips(pos, player) != 0
    }

    /// 指定位置の周囲8方向のマスクを計算
    #[inline(always)]
    fn get_adjacent_mask(&self, pos: usize) -> u64 {
        let pos_bit = 1u64 << pos;
        let mut mask = 0;

        // すべての方向を SHIFTS からループで処理
        for &(shift, dir_mask, is_forward) in Self::SHIFTS.iter() {
            if is_forward {
                mask |= (pos_bit << shift) & dir_mask;
            } else {
                mask |= (pos_bit >> shift) & dir_mask;
            }
        }

        mask
    }

    /// 合法手の一覧をビットボードとして取得（修正版）
    #[inline(always)]
    pub fn get_legal_moves(&self, player: Player) -> u64 {
        let mut legal_moves = 0u64;
        let occupied = self.black | self.white;

        // 全ての空きマスをチェック
        for pos in 0..64 {
            let pos_bit = 1u64 << pos;

            // 既に石が置かれていればスキップ
            if (occupied & pos_bit) != 0 {
                continue;
            }

            // この位置でひっくり返せる石があるかチェック
            if self.compute_flips(pos, player) != 0 {
                legal_moves |= pos_bit;
            }
        }

        legal_moves
    }

    /// 合法手の一覧を座標のベクターとして取得
    pub fn get_legal_move_positions(&self, player: Player) -> Vec<usize> {
        let legal_moves = self.get_legal_moves(player);
        let mut positions = Vec::new();

        for pos in 0..64 {
            let bit = 1u64 << pos;
            if (legal_moves & bit) != 0 {
                positions.push(pos);
            }
        }

        positions
    }

    /// 指定位置の石を取得
    #[inline]
    pub fn get_disc(&self, pos: usize) -> Option<Player> {
        if pos >= 64 {
            return None;
        }

        let bit = 1u64 << pos;

        if (self.black & bit) != 0 {
            Some(Player::Black)
        } else if (self.white & bit) != 0 {
            Some(Player::White)
        } else {
            None
        }
    }

    /// 指定位置の石を行と列の形式で取得
    pub fn get_disc_at(&self, row: usize, col: usize) -> Option<Player> {
        if row >= 8 || col >= 8 {
            return None;
        }
        self.get_disc(row * 8 + col)
    }

    /// 石の数をカウント
    #[inline]
    pub fn count_discs(&self, player: Player) -> u32 {
        match player {
            Player::Black => self.black.count_ones(),
            Player::White => self.white.count_ones(),
        }
    }

    /// 両プレイヤーの石の数を取得
    pub fn count_all_discs(&self) -> (u32, u32) {
        (self.black.count_ones(), self.white.count_ones())
    }

    /// パス判定
    pub fn is_pass_required(&self, player: Player) -> bool {
        self.get_legal_moves(player) == 0
    }

    /// ゲーム終了判定（最適化版）
    #[inline]
    pub fn is_game_over(&self) -> bool {
        // 空きマスがなければ終了
        if self.black | self.white == !0u64 {
            return true;
        }

        // 両者にとって合法手がなければ終了
        self.get_legal_moves(Player::Black) == 0 && self.get_legal_moves(Player::White) == 0
    }

    /// 勝者を返す
    pub fn get_winner(&self) -> Option<Player> {
        let black_count = self.count_discs(Player::Black);
        let white_count = self.count_discs(Player::White);

        if black_count == white_count {
            None // 引き分け
        } else if black_count > white_count {
            Some(Player::Black)
        } else {
            Some(Player::White)
        }
    }
}

impl Default for BitBoard {
    fn default() -> Self {
        BitBoard::new()
    }
}

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  0 1 2 3 4 5 6 7")?;

        for row in 0..8 {
            write!(f, "{}|", row)?;

            for col in 0..8 {
                match self.get_disc_at(row, col) {
                    Some(Player::Black) => write!(f, "X|")?,
                    Some(Player::White) => write!(f, "O|")?,
                    None => write!(f, " |")?,
                }
            }

            writeln!(f, "")?;
        }

        let black_count = self.count_discs(Player::Black);
        let white_count = self.count_discs(Player::White);
        writeln!(f, "黒(X): {} 白(O): {}", black_count, white_count)
    }
}
