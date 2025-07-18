use crate::board::BitBoard;
use crate::gui::app::Language;
use crate::player::Player;
use eframe::egui;

pub struct GameView {
    cell_size: f32,
}

impl GameView {
    pub fn new() -> Self {
        Self { cell_size: 50.0 }
    }

    pub fn show(
        &mut self,
        board: &BitBoard,
        current_player: Player,
        ui: &mut egui::Ui,
        language: Language,
    ) -> Option<(usize, usize)> {
        let legal_moves = board.get_legal_moves(current_player);
        let mut clicked_cell = None;

        ui.horizontal(|ui| {
            let board_size_label = match language {
                Language::Japanese => "盤面サイズ:",
                Language::English => "Board Size:",
            };
            ui.label(board_size_label);
            ui.add(egui::Slider::new(&mut self.cell_size, 30.0..=80.0).text("px"));
        });

        ui.add_space(10.0);

        // ボード描画
        let board_size = self.cell_size * 8.0;
        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(board_size + 20.0, board_size + 40.0),
            egui::Sense::click(),
        );

        let board_rect = egui::Rect::from_min_size(
            response.rect.min + egui::Vec2::new(10.0, 30.0),
            egui::Vec2::new(board_size, board_size),
        );

        // 背景
        painter.rect_filled(board_rect, 0.0, egui::Color32::from_rgb(34, 139, 34));

        // グリッド線とセル
        for row in 0..8 {
            for col in 0..8 {
                let cell_rect = egui::Rect::from_min_size(
                    board_rect.min
                        + egui::Vec2::new(col as f32 * self.cell_size, row as f32 * self.cell_size),
                    egui::Vec2::new(self.cell_size, self.cell_size),
                );

                // セルの境界線
                painter.rect_stroke(cell_rect, 0.0, egui::Stroke::new(1.0, egui::Color32::BLACK));

                // 石の描画
                let position = row * 8 + col;
                let black_pieces = board.black;
                let white_pieces = board.white;

                let center = cell_rect.center();
                let radius = self.cell_size * 0.35;

                if (black_pieces & (1u64 << position)) != 0 {
                    // 黒石
                    painter.circle_filled(center, radius, egui::Color32::BLACK);
                    painter.circle_stroke(
                        center,
                        radius,
                        egui::Stroke::new(1.0, egui::Color32::GRAY),
                    );
                } else if (white_pieces & (1u64 << position)) != 0 {
                    // 白石
                    painter.circle_filled(center, radius, egui::Color32::WHITE);
                    painter.circle_stroke(
                        center,
                        radius,
                        egui::Stroke::new(1.0, egui::Color32::BLACK),
                    );
                } else if (legal_moves & (1u64 << position)) != 0 {
                    // 合法手の表示
                    painter.circle_stroke(
                        center,
                        radius * 0.6,
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 215, 0)),
                    );

                    // 小さな点を中央に
                    painter.circle_filled(center, 3.0, egui::Color32::from_rgb(255, 215, 0));
                }
            }
        }

        // クリック処理
        if response.clicked() {
            if let Some(click_pos) = response.interact_pointer_pos() {
                // どのセルがクリックされたかを判定
                let rel_x = click_pos.x - board_rect.min.x;
                let rel_y = click_pos.y - board_rect.min.y;

                if rel_x >= 0.0 && rel_y >= 0.0 && rel_x < board_size && rel_y < board_size {
                    let col = (rel_x / self.cell_size) as usize;
                    let row = (rel_y / self.cell_size) as usize;

                    if row < 8 && col < 8 {
                        clicked_cell = Some((row, col));
                    }
                }
            }
        }

        // 座標ラベル
        for i in 0..8 {
            // 行番号（左側）
            let row_pos = egui::Pos2::new(
                board_rect.min.x - 15.0,
                board_rect.min.y + i as f32 * self.cell_size + self.cell_size / 2.0,
            );
            painter.text(
                row_pos,
                egui::Align2::CENTER_CENTER,
                i.to_string(),
                egui::FontId::proportional(12.0),
                egui::Color32::BLACK,
            );

            // 列番号（上側）
            let col_pos = egui::Pos2::new(
                board_rect.min.x + i as f32 * self.cell_size + self.cell_size / 2.0,
                board_rect.min.y - 15.0,
            );
            painter.text(
                col_pos,
                egui::Align2::CENTER_CENTER,
                i.to_string(),
                egui::FontId::proportional(12.0),
                egui::Color32::BLACK,
            );
        }

        // 現在のプレイヤー表示
        let player_text = match language {
            Language::Japanese => format!(
                "現在の手番: {} ({})",
                current_player.to_string(),
                current_player.to_char()
            ),
            Language::English => format!(
                "Current turn: {} ({})",
                current_player.to_string(),
                current_player.to_char()
            ),
        };

        let text_pos = egui::Pos2::new(board_rect.min.x, board_rect.max.y + 10.0);

        painter.text(
            text_pos,
            egui::Align2::LEFT_TOP,
            player_text,
            egui::FontId::proportional(14.0),
            egui::Color32::BLACK,
        );

        // 合法手の数を表示
        let legal_move_count = legal_moves.count_ones();
        if legal_move_count > 0 {
            let moves_text = match language {
                Language::Japanese => format!("打てる場所: {}箇所", legal_move_count),
                Language::English => format!("Legal moves: {} positions", legal_move_count),
            };
            let moves_pos = egui::Pos2::new(board_rect.min.x + 200.0, board_rect.max.y + 10.0);

            painter.text(
                moves_pos,
                egui::Align2::LEFT_TOP,
                moves_text,
                egui::FontId::proportional(14.0),
                egui::Color32::DARK_BLUE,
            );
        }

        clicked_cell
    }
}
