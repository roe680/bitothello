use crate::gui::app::Language;
use crate::player::Player;
use crate::stats::{GameResult, GameStats};
use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlotType {
    DiscCount,
    ThinkingTime,
    Evaluation,
    Overview,
}

pub struct PlotViewer {
    selected_plot: PlotType,
    has_data: bool,
}

impl PlotViewer {
    pub fn new() -> Self {
        Self {
            selected_plot: PlotType::DiscCount,
            has_data: false,
        }
    }

    pub fn mark_data_available(&mut self) {
        self.has_data = true;
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        language: Language,
        stats: &GameStats,
        result: &GameResult,
    ) {
        if !self.has_data {
            let no_data_text = match language {
                Language::Japanese => "グラフデータがありません。ゲームを完了してください。",
                Language::English => "No graph data available. Please complete a game.",
            };
            ui.label(no_data_text);
            return;
        }

        // Plot type selector
        ui.horizontal(|ui| {
            let plot_type_label = match language {
                Language::Japanese => "グラフタイプ:",
                Language::English => "Plot Type:",
            };
            ui.label(plot_type_label);

            let disc_count_text = match language {
                Language::Japanese => "石数推移",
                Language::English => "Disc Count",
            };
            if ui
                .selectable_label(self.selected_plot == PlotType::DiscCount, disc_count_text)
                .clicked()
            {
                self.selected_plot = PlotType::DiscCount;
            }

            let thinking_time_text = match language {
                Language::Japanese => "思考時間",
                Language::English => "Thinking Time",
            };
            if ui
                .selectable_label(
                    self.selected_plot == PlotType::ThinkingTime,
                    thinking_time_text,
                )
                .clicked()
            {
                self.selected_plot = PlotType::ThinkingTime;
            }

            let evaluation_text = match language {
                Language::Japanese => "AI評価値",
                Language::English => "AI Evaluation",
            };
            if ui
                .selectable_label(self.selected_plot == PlotType::Evaluation, evaluation_text)
                .clicked()
            {
                self.selected_plot = PlotType::Evaluation;
            }

            let overview_text = match language {
                Language::Japanese => "総合表示",
                Language::English => "Overview",
            };
            if ui
                .selectable_label(self.selected_plot == PlotType::Overview, overview_text)
                .clicked()
            {
                self.selected_plot = PlotType::Overview;
            }
        });

        ui.separator();

        // Display selected plot
        match self.selected_plot {
            PlotType::DiscCount => self.show_disc_count_plot(ui, language, stats, result),
            PlotType::ThinkingTime => self.show_thinking_time_plot(ui, language, stats, result),
            PlotType::Evaluation => self.show_evaluation_plot(ui, language, stats, result),
            PlotType::Overview => self.show_overview_plots(ui, language, stats, result),
        }
    }

    fn show_disc_count_plot(
        &self,
        ui: &mut egui::Ui,
        language: Language,
        stats: &GameStats,
        result: &GameResult,
    ) {
        let disc_history = stats.get_disc_count_history();

        if disc_history.is_empty() {
            let no_data_text = match language {
                Language::Japanese => "石数データがありません。",
                Language::English => "No disc count data available.",
            };
            ui.label(no_data_text);
            return;
        }

        let _title = match language {
            Language::Japanese => "石数の推移",
            Language::English => "Disc Count History",
        };

        let x_label = match language {
            Language::Japanese => "手数",
            Language::English => "Move Number",
        };

        let y_label = match language {
            Language::Japanese => "石数",
            Language::English => "Piece Count",
        };

        // Prepare data
        let black_points: PlotPoints = disc_history
            .iter()
            .map(|(move_num, black, _)| [*move_num as f64, *black as f64])
            .collect();

        let white_points: PlotPoints = disc_history
            .iter()
            .map(|(move_num, _, white)| [*move_num as f64, *white as f64])
            .collect();

        Plot::new("disc_count_plot")
            .legend(egui_plot::Legend::default())
            .x_axis_label(x_label)
            .y_axis_label(y_label)
            .show(ui, |plot_ui| {
                let black_label = match language {
                    Language::Japanese => "黒",
                    Language::English => "Black",
                };
                plot_ui.line(
                    Line::new(black_points)
                        .color(egui::Color32::RED)
                        .name(black_label),
                );

                let white_label = match language {
                    Language::Japanese => "白",
                    Language::English => "White",
                };
                plot_ui.line(
                    Line::new(white_points)
                        .color(egui::Color32::BLUE)
                        .name(white_label),
                );
            });

        // Add game result summary
        ui.add_space(10.0);
        self.show_game_result_summary(ui, language, result);
    }

    fn show_thinking_time_plot(
        &self,
        ui: &mut egui::Ui,
        language: Language,
        stats: &GameStats,
        result: &GameResult,
    ) {
        let time_history = stats.get_thinking_time_history();

        if time_history.is_empty() {
            let no_data_text = match language {
                Language::Japanese => "思考時間データがありません。",
                Language::English => "No thinking time data available.",
            };
            ui.label(no_data_text);
            return;
        }

        let _title = match language {
            Language::Japanese => "思考時間の推移",
            Language::English => "Thinking Time History",
        };

        let x_label = match language {
            Language::Japanese => "手数",
            Language::English => "Move Number",
        };

        let y_label = match language {
            Language::Japanese => "思考時間 (秒)",
            Language::English => "Thinking Time (seconds)",
        };

        // Prepare data
        let time_points: PlotPoints = time_history
            .iter()
            .map(|(move_num, time)| [*move_num as f64, *time])
            .collect();

        // Calculate average
        let avg_time =
            time_history.iter().map(|(_, time)| time).sum::<f64>() / time_history.len() as f64;

        Plot::new("thinking_time_plot")
            .legend(egui_plot::Legend::default())
            .x_axis_label(x_label)
            .y_axis_label(y_label)
            .show(ui, |plot_ui| {
                let time_label = match language {
                    Language::Japanese => "思考時間",
                    Language::English => "Thinking Time",
                };
                plot_ui.line(
                    Line::new(time_points)
                        .color(egui::Color32::RED)
                        .name(time_label),
                );

                // Add average line
                if !time_history.is_empty() {
                    let first_move = time_history.first().unwrap().0 as f64;
                    let last_move = time_history.last().unwrap().0 as f64;
                    let avg_line: PlotPoints =
                        vec![[first_move, avg_time], [last_move, avg_time]].into();

                    let avg_label = match language {
                        Language::Japanese => format!("平均: {:.2}秒", avg_time),
                        Language::English => format!("Average: {:.2}s", avg_time),
                    };
                    plot_ui.line(
                        Line::new(avg_line)
                            .color(egui::Color32::GREEN)
                            .stroke(egui::Stroke::new(2.0, egui::Color32::GREEN))
                            .name(avg_label),
                    );
                }
            });

        ui.add_space(10.0);
        self.show_thinking_time_stats(ui, language, result, stats);
    }

    fn show_evaluation_plot(
        &self,
        ui: &mut egui::Ui,
        language: Language,
        stats: &GameStats,
        _result: &GameResult,
    ) {
        let eval_history = stats.get_evaluation_history();

        if eval_history.is_empty() {
            let no_data_text = match language {
                Language::Japanese => "AI評価値データがありません。AIプレイヤーが必要です。",
                Language::English => "No AI evaluation data available. AI players required.",
            };
            ui.label(no_data_text);
            return;
        }

        let _title = match language {
            Language::Japanese => "AI評価値の推移",
            Language::English => "AI Evaluation History",
        };

        let x_label = match language {
            Language::Japanese => "手数",
            Language::English => "Move Number",
        };

        let y_label = match language {
            Language::Japanese => "評価値",
            Language::English => "Evaluation Score",
        };

        // Separate data by player
        let black_evals: PlotPoints = eval_history
            .iter()
            .filter(|(_, player, _)| *player == Player::Black)
            .map(|(move_num, _, eval)| [*move_num as f64, *eval as f64])
            .collect();

        let white_evals: PlotPoints = eval_history
            .iter()
            .filter(|(_, player, _)| *player == Player::White)
            .map(|(move_num, _, eval)| [*move_num as f64, *eval as f64])
            .collect();

        Plot::new("evaluation_plot")
            .legend(egui_plot::Legend::default())
            .x_axis_label(x_label)
            .y_axis_label(y_label)
            .show(ui, |plot_ui| {
                if black_evals.points().len() > 0 {
                    let black_label = match language {
                        Language::Japanese => "黒AI評価値",
                        Language::English => "Black AI Evaluation",
                    };
                    plot_ui.line(
                        Line::new(black_evals)
                            .color(egui::Color32::RED)
                            .name(black_label),
                    );
                }

                if white_evals.points().len() > 0 {
                    let white_label = match language {
                        Language::Japanese => "白AI評価値",
                        Language::English => "White AI Evaluation",
                    };
                    plot_ui.line(
                        Line::new(white_evals)
                            .color(egui::Color32::BLUE)
                            .name(white_label),
                    );
                }

                // Add zero line
                if let (Some(first), Some(last)) = (eval_history.first(), eval_history.last()) {
                    let zero_line: PlotPoints =
                        vec![[first.0 as f64, 0.0], [last.0 as f64, 0.0]].into();
                    plot_ui.line(
                        Line::new(zero_line)
                            .color(egui::Color32::GRAY)
                            .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
                            .name("Zero"),
                    );
                }
            });

        ui.add_space(10.0);
        self.show_evaluation_stats(ui, language, stats);
    }

    fn show_overview_plots(
        &self,
        ui: &mut egui::Ui,
        language: Language,
        stats: &GameStats,
        result: &GameResult,
    ) {
        ui.horizontal(|ui| {
            // Left column - Disc count
            ui.vertical(|ui| {
                ui.set_min_width(300.0);
                let title = match language {
                    Language::Japanese => "石数推移（簡略）",
                    Language::English => "Disc Count (Brief)",
                };
                ui.label(title);
                self.show_mini_disc_plot(ui, language, stats);
            });

            ui.separator();

            // Right column - Thinking time
            ui.vertical(|ui| {
                ui.set_min_width(300.0);
                let title = match language {
                    Language::Japanese => "思考時間（簡略）",
                    Language::English => "Thinking Time (Brief)",
                };
                ui.label(title);
                self.show_mini_time_plot(ui, language, stats);
            });
        });

        ui.separator();
        ui.add_space(10.0);

        // Game summary
        self.show_detailed_game_summary(ui, language, result);
    }

    fn show_mini_disc_plot(&self, ui: &mut egui::Ui, _language: Language, stats: &GameStats) {
        let disc_history = stats.get_disc_count_history();

        if disc_history.is_empty() {
            return;
        }

        let black_points: PlotPoints = disc_history
            .iter()
            .map(|(move_num, black, _)| [*move_num as f64, *black as f64])
            .collect();

        let white_points: PlotPoints = disc_history
            .iter()
            .map(|(move_num, _, white)| [*move_num as f64, *white as f64])
            .collect();

        Plot::new("mini_disc_plot")
            .height(150.0)
            .show(ui, |plot_ui| {
                plot_ui.line(Line::new(black_points).color(egui::Color32::RED));
                plot_ui.line(Line::new(white_points).color(egui::Color32::BLUE));
            });
    }

    fn show_mini_time_plot(&self, ui: &mut egui::Ui, _language: Language, stats: &GameStats) {
        let time_history = stats.get_thinking_time_history();

        if time_history.is_empty() {
            return;
        }

        let time_points: PlotPoints = time_history
            .iter()
            .map(|(move_num, time)| [*move_num as f64, *time])
            .collect();

        Plot::new("mini_time_plot")
            .height(150.0)
            .show(ui, |plot_ui| {
                plot_ui.line(Line::new(time_points).color(egui::Color32::RED));
            });
    }

    fn show_game_result_summary(&self, ui: &mut egui::Ui, language: Language, result: &GameResult) {
        ui.group(|ui| {
            let title = match language {
                Language::Japanese => "ゲーム結果",
                Language::English => "Game Result",
            };
            ui.label(egui::RichText::new(title).strong());

            match (result.winner, language) {
                (Some(Player::Black), Language::Japanese) => {
                    ui.label(format!(
                        "勝者: 黒 ({} vs {})",
                        result.black_final_count, result.white_final_count
                    ));
                }
                (Some(Player::Black), Language::English) => {
                    ui.label(format!(
                        "Winner: Black ({} vs {})",
                        result.black_final_count, result.white_final_count
                    ));
                }
                (Some(Player::White), Language::Japanese) => {
                    ui.label(format!(
                        "勝者: 白 ({} vs {})",
                        result.black_final_count, result.white_final_count
                    ));
                }
                (Some(Player::White), Language::English) => {
                    ui.label(format!(
                        "Winner: White ({} vs {})",
                        result.black_final_count, result.white_final_count
                    ));
                }
                (None, Language::Japanese) => {
                    ui.label(format!(
                        "引き分け ({} vs {})",
                        result.black_final_count, result.white_final_count
                    ));
                }
                (None, Language::English) => {
                    ui.label(format!(
                        "Draw ({} vs {})",
                        result.black_final_count, result.white_final_count
                    ));
                }
            }
        });
    }

    fn show_thinking_time_stats(
        &self,
        ui: &mut egui::Ui,
        language: Language,
        result: &GameResult,
        _stats: &GameStats,
    ) {
        ui.group(|ui| {
            let title = match language {
                Language::Japanese => "思考時間統計",
                Language::English => "Thinking Time Statistics",
            };
            ui.label(egui::RichText::new(title).strong());

            match language {
                Language::Japanese => {
                    ui.label(format!("総手数: {}", result.total_moves));
                    ui.label(format!("総思考時間: {:.2?}", result.total_thinking_time));
                    if result.total_moves > 0 {
                        ui.label(format!(
                            "平均思考時間: {:.2?}",
                            result.total_thinking_time / result.total_moves as u32
                        ));
                    }
                }
                Language::English => {
                    ui.label(format!("Total moves: {}", result.total_moves));
                    ui.label(format!(
                        "Total thinking time: {:.2?}",
                        result.total_thinking_time
                    ));
                    if result.total_moves > 0 {
                        ui.label(format!(
                            "Average thinking time: {:.2?}",
                            result.total_thinking_time / result.total_moves as u32
                        ));
                    }
                }
            }
        });
    }

    fn show_evaluation_stats(&self, ui: &mut egui::Ui, language: Language, stats: &GameStats) {
        let eval_history = stats.get_evaluation_history();

        if eval_history.is_empty() {
            return;
        }

        ui.group(|ui| {
            let title = match language {
                Language::Japanese => "評価値統計",
                Language::English => "Evaluation Statistics",
            };
            ui.label(egui::RichText::new(title).strong());

            let max_eval = eval_history
                .iter()
                .map(|(_, _, eval)| *eval)
                .max()
                .unwrap_or(0);
            let min_eval = eval_history
                .iter()
                .map(|(_, _, eval)| *eval)
                .min()
                .unwrap_or(0);

            match language {
                Language::Japanese => {
                    ui.label(format!("最高評価値: {}", max_eval));
                    ui.label(format!("最低評価値: {}", min_eval));
                    ui.label(format!("評価値記録数: {}", eval_history.len()));
                }
                Language::English => {
                    ui.label(format!("Max evaluation: {}", max_eval));
                    ui.label(format!("Min evaluation: {}", min_eval));
                    ui.label(format!("Evaluation records: {}", eval_history.len()));
                }
            }
        });
    }

    fn show_detailed_game_summary(
        &self,
        ui: &mut egui::Ui,
        language: Language,
        result: &GameResult,
    ) {
        ui.group(|ui| {
            let title = match language {
                Language::Japanese => "詳細ゲーム情報",
                Language::English => "Detailed Game Information",
            };
            ui.label(egui::RichText::new(title).heading());

            ui.separator();

            match language {
                Language::Japanese => {
                    ui.label(format!("総手数: {}", result.total_moves));
                    ui.label(format!(
                        "ゲーム時間: {:.1}秒",
                        result.game_duration.as_secs_f64()
                    ));
                    ui.label(format!("総思考時間: {:.2?}", result.total_thinking_time));
                    ui.label(format!(
                        "最終スコア: 黒 {} - 白 {}",
                        result.black_final_count, result.white_final_count
                    ));

                    let winner_text = match result.winner {
                        Some(Player::Black) => "黒の勝利",
                        Some(Player::White) => "白の勝利",
                        None => "引き分け",
                    };
                    ui.label(format!("結果: {}", winner_text));
                }
                Language::English => {
                    ui.label(format!("Total moves: {}", result.total_moves));
                    ui.label(format!(
                        "Game duration: {:.1} seconds",
                        result.game_duration.as_secs_f64()
                    ));
                    ui.label(format!(
                        "Total thinking time: {:.2?}",
                        result.total_thinking_time
                    ));
                    ui.label(format!(
                        "Final score: Black {} - White {}",
                        result.black_final_count, result.white_final_count
                    ));

                    let winner_text = match result.winner {
                        Some(Player::Black) => "Black wins",
                        Some(Player::White) => "White wins",
                        None => "Draw",
                    };
                    ui.label(format!("Result: {}", winner_text));
                }
            }
        });
    }
}
