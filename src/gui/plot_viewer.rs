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
    fixed_bounds: bool,
}

impl PlotViewer {
    pub fn new() -> Self {
        Self {
            selected_plot: PlotType::DiscCount,
            has_data: false,
            fixed_bounds: true,
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

        // Add fixed container to control expansion
        ui.allocate_ui_with_layout(
            egui::Vec2::new(800.0, 600.0),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                egui::ScrollArea::vertical()
                    .max_height(580.0)
                    .show(ui, |ui| {
                        self.show_content(ui, language, stats, result);
                    });
            },
        );
    }

    fn show_content(
        &mut self,
        ui: &mut egui::Ui,
        language: Language,
        stats: &GameStats,
        result: &GameResult,
    ) {
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

        // Bounds control
        ui.horizontal(|ui| {
            let bounds_label = match language {
                Language::Japanese => "固定範囲:",
                Language::English => "Fixed Bounds:",
            };
            ui.label(bounds_label);

            let checkbox_tooltip = match language {
                Language::Japanese => "チェックするとグラフの範囲を固定し、継続的な拡張を防ぎます",
                Language::English => "Check to fix graph bounds and prevent continuous expansion",
            };
            ui.checkbox(&mut self.fixed_bounds, "")
                .on_hover_text(checkbox_tooltip);

            ui.separator();

            if ui
                .small_button("🔄")
                .on_hover_text(match language {
                    Language::Japanese => "グラフ表示をリセット",
                    Language::English => "Reset graph display",
                })
                .clicked()
            {
                // Force plot to recalculate bounds
                ui.ctx().request_repaint();
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

        let mut plot = Plot::new("main_disc_count_plot")
            .legend(egui_plot::Legend::default())
            .x_axis_label(x_label)
            .y_axis_label(y_label)
            .height(400.0)
            .width(700.0)
            .view_aspect(1.75);

        if self.fixed_bounds {
            // Set fixed bounds to prevent continuous expansion
            let max_move = disc_history.iter().map(|(m, _, _)| *m).max().unwrap_or(0) as f64;
            plot = plot
                .include_x(0.0)
                .include_x(max_move + 1.0)
                .include_y(0.0)
                .include_y(64.0);
        } else {
            plot = plot.auto_bounds_x().auto_bounds_y();
        }

        plot.show(ui, |plot_ui| {
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

        let mut plot = Plot::new("main_thinking_time_plot")
            .legend(egui_plot::Legend::default())
            .x_axis_label(x_label)
            .y_axis_label(y_label)
            .height(400.0)
            .width(700.0)
            .view_aspect(1.75);

        if self.fixed_bounds {
            // Set fixed bounds to prevent continuous expansion
            let max_move = time_history.iter().map(|(m, _)| *m).max().unwrap_or(0) as f64;
            let max_time = time_history.iter().map(|(_, t)| *t).fold(0.0, f64::max);
            plot = plot
                .include_x(0.0)
                .include_x(max_move + 1.0)
                .include_y(0.0)
                .include_y(max_time * 1.1);
        } else {
            plot = plot.auto_bounds_x().auto_bounds_y();
        }

        plot.show(ui, |plot_ui| {
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

        let mut plot = Plot::new("main_evaluation_plot")
            .legend(egui_plot::Legend::default())
            .x_axis_label(x_label)
            .y_axis_label(y_label)
            .height(400.0)
            .width(700.0)
            .view_aspect(1.75);

        if self.fixed_bounds {
            // Set fixed bounds to prevent continuous expansion
            let max_move = eval_history.iter().map(|(m, _, _)| *m).max().unwrap_or(0) as f64;
            let min_eval = eval_history.iter().map(|(_, _, e)| *e).min().unwrap_or(0) as f64;
            let max_eval = eval_history.iter().map(|(_, _, e)| *e).max().unwrap_or(0) as f64;
            let eval_range = (max_eval - min_eval).max(100.0); // Minimum range of 100
            plot = plot
                .include_x(0.0)
                .include_x(max_move + 1.0)
                .include_y(min_eval - eval_range * 0.1)
                .include_y(max_eval + eval_range * 0.1);
        } else {
            plot = plot.auto_bounds_x().auto_bounds_y();
        }

        plot.show(ui, |plot_ui| {
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
                ui.set_width(350.0);
                ui.set_height(200.0);
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
                ui.set_width(350.0);
                ui.set_height(200.0);
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

        let mut plot = Plot::new("overview_mini_disc_plot")
            .height(150.0)
            .width(300.0)
            .view_aspect(2.0);

        if self.fixed_bounds {
            let max_move = disc_history.iter().map(|(m, _, _)| *m).max().unwrap_or(0) as f64;
            plot = plot
                .include_x(0.0)
                .include_x(max_move + 1.0)
                .include_y(0.0)
                .include_y(64.0);
        } else {
            plot = plot.auto_bounds_x().auto_bounds_y();
        }

        plot.show(ui, |plot_ui| {
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

        let mut plot = Plot::new("overview_mini_time_plot")
            .height(150.0)
            .width(300.0)
            .view_aspect(2.0);

        if self.fixed_bounds {
            let max_move = time_history.iter().map(|(m, _)| *m).max().unwrap_or(0) as f64;
            let max_time = time_history.iter().map(|(_, t)| *t).fold(0.0, f64::max);
            plot = plot
                .include_x(0.0)
                .include_x(max_move + 1.0)
                .include_y(0.0)
                .include_y(max_time * 1.1);
        } else {
            plot = plot.auto_bounds_x().auto_bounds_y();
        }

        plot.show(ui, |plot_ui| {
            plot_ui.line(Line::new(time_points).color(egui::Color32::RED));
        });
    }

    fn show_game_result_summary(&self, ui: &mut egui::Ui, language: Language, result: &GameResult) {
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    let result_title = match language {
                        Language::Japanese => "ゲーム結果",
                        Language::English => "Game Result",
                    };
                    ui.strong(result_title);

                    let winner_text = match result.winner {
                        Some(Player::Black) => match language {
                            Language::Japanese => "勝者: 黒",
                            Language::English => "Winner: Black",
                        },
                        Some(Player::White) => match language {
                            Language::Japanese => "勝者: 白",
                            Language::English => "Winner: White",
                        },
                        None => match language {
                            Language::Japanese => "引き分け",
                            Language::English => "Draw",
                        },
                    };
                    ui.label(winner_text);

                    let score_text = match language {
                        Language::Japanese => {
                            format!(
                                "最終スコア: 黒{}個 - 白{}個",
                                result.black_final_count, result.white_final_count
                            )
                        }
                        Language::English => {
                            format!(
                                "Final Score: Black {} - White {}",
                                result.black_final_count, result.white_final_count
                            )
                        }
                    };
                    ui.label(score_text);
                });
            });
        });
    }

    fn show_thinking_time_stats(
        &self,
        ui: &mut egui::Ui,
        language: Language,
        result: &GameResult,
        stats: &GameStats,
    ) {
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    let stats_title = match language {
                        Language::Japanese => "思考時間統計",
                        Language::English => "Thinking Time Statistics",
                    };
                    ui.strong(stats_title);

                    let total_moves_text = match language {
                        Language::Japanese => format!("総手数: {}", result.total_moves),
                        Language::English => format!("Total Moves: {}", result.total_moves),
                    };
                    ui.label(total_moves_text);

                    let game_duration_text = match language {
                        Language::Japanese => {
                            format!("ゲーム時間: {:.1}秒", result.game_duration.as_secs_f64())
                        }
                        Language::English => {
                            format!("Game Duration: {:.1}s", result.game_duration.as_secs_f64())
                        }
                    };
                    ui.label(game_duration_text);

                    let total_thinking_text = match language {
                        Language::Japanese => format!(
                            "総思考時間: {:.1}秒",
                            result.total_thinking_time.as_secs_f64()
                        ),
                        Language::English => format!(
                            "Total Thinking Time: {:.1}s",
                            result.total_thinking_time.as_secs_f64()
                        ),
                    };
                    ui.label(total_thinking_text);

                    if result.total_moves > 0 {
                        let avg_thinking_time =
                            result.total_thinking_time.as_secs_f64() / result.total_moves as f64;
                        let avg_text = match language {
                            Language::Japanese => {
                                format!("平均思考時間: {:.2}秒", avg_thinking_time)
                            }
                            Language::English => {
                                format!("Average Thinking Time: {:.2}s", avg_thinking_time)
                            }
                        };
                        ui.label(avg_text);
                    }

                    // Min/Max thinking times
                    let time_history = stats.get_thinking_time_history();
                    if !time_history.is_empty() {
                        let times: Vec<f64> = time_history.iter().map(|(_, time)| *time).collect();
                        let min_time = times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                        let max_time = times.iter().fold(0.0f64, |a, &b| a.max(b));

                        let min_text = match language {
                            Language::Japanese => format!("最短思考: {:.2}秒", min_time),
                            Language::English => format!("Min Thinking: {:.2}s", min_time),
                        };
                        ui.label(min_text);

                        let max_text = match language {
                            Language::Japanese => format!("最長思考: {:.2}秒", max_time),
                            Language::English => format!("Max Thinking: {:.2}s", max_time),
                        };
                        ui.label(max_text);
                    }
                });
            });
        });
    }

    fn show_evaluation_stats(&self, ui: &mut egui::Ui, language: Language, stats: &GameStats) {
        let eval_history = stats.get_evaluation_history();
        if eval_history.is_empty() {
            return;
        }

        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    let stats_title = match language {
                        Language::Japanese => "AI評価値統計",
                        Language::English => "AI Evaluation Statistics",
                    };
                    ui.strong(stats_title);

                    // Separate by player
                    let black_evals: Vec<i32> = eval_history
                        .iter()
                        .filter(|(_, player, _)| *player == Player::Black)
                        .map(|(_, _, eval)| *eval)
                        .collect();

                    let white_evals: Vec<i32> = eval_history
                        .iter()
                        .filter(|(_, player, _)| *player == Player::White)
                        .map(|(_, _, eval)| *eval)
                        .collect();

                    if !black_evals.is_empty() {
                        let black_avg =
                            black_evals.iter().sum::<i32>() as f64 / black_evals.len() as f64;
                        let black_text = match language {
                            Language::Japanese => format!("黒AI平均評価: {:.1}", black_avg),
                            Language::English => format!("Black AI Avg Eval: {:.1}", black_avg),
                        };
                        ui.label(black_text);
                    }

                    if !white_evals.is_empty() {
                        let white_avg =
                            white_evals.iter().sum::<i32>() as f64 / white_evals.len() as f64;
                        let white_text = match language {
                            Language::Japanese => format!("白AI平均評価: {:.1}", white_avg),
                            Language::English => format!("White AI Avg Eval: {:.1}", white_avg),
                        };
                        ui.label(white_text);
                    }
                });
            });
        });
    }

    fn show_detailed_game_summary(
        &self,
        ui: &mut egui::Ui,
        language: Language,
        result: &GameResult,
    ) {
        ui.group(|ui| {
            ui.vertical(|ui| {
                let summary_title = match language {
                    Language::Japanese => "詳細ゲームサマリー",
                    Language::English => "Detailed Game Summary",
                };
                ui.strong(summary_title);

                ui.separator();

                // Winner
                let winner_text = match result.winner {
                    Some(Player::Black) => match language {
                        Language::Japanese => "🏆 勝者: 黒プレイヤー",
                        Language::English => "🏆 Winner: Black Player",
                    },
                    Some(Player::White) => match language {
                        Language::Japanese => "🏆 勝者: 白プレイヤー",
                        Language::English => "🏆 Winner: White Player",
                    },
                    None => match language {
                        Language::Japanese => "🤝 引き分け",
                        Language::English => "🤝 Draw",
                    },
                };
                ui.label(winner_text);

                // Score
                let score_text = match language {
                    Language::Japanese => {
                        format!(
                            "📊 最終スコア: 黒 {} - {} 白",
                            result.black_final_count, result.white_final_count
                        )
                    }
                    Language::English => {
                        format!(
                            "📊 Final Score: Black {} - {} White",
                            result.black_final_count, result.white_final_count
                        )
                    }
                };
                ui.label(score_text);

                // Score difference
                let diff =
                    (result.black_final_count as i32 - result.white_final_count as i32).abs();
                let diff_text = match language {
                    Language::Japanese => format!("📈 点差: {}点", diff),
                    Language::English => format!("📈 Score Difference: {} points", diff),
                };
                ui.label(diff_text);

                // Game stats
                let moves_text = match language {
                    Language::Japanese => format!("🎯 総手数: {}", result.total_moves),
                    Language::English => format!("🎯 Total Moves: {}", result.total_moves),
                };
                ui.label(moves_text);

                let duration_text = match language {
                    Language::Japanese => {
                        format!("⏱️ ゲーム時間: {:.1}秒", result.game_duration.as_secs_f64())
                    }
                    Language::English => {
                        format!(
                            "⏱️ Game Duration: {:.1}s",
                            result.game_duration.as_secs_f64()
                        )
                    }
                };
                ui.label(duration_text);

                let thinking_text = match language {
                    Language::Japanese => {
                        format!(
                            "🤔 総思考時間: {:.1}秒",
                            result.total_thinking_time.as_secs_f64()
                        )
                    }
                    Language::English => {
                        format!(
                            "🤔 Total Thinking Time: {:.1}s",
                            result.total_thinking_time.as_secs_f64()
                        )
                    }
                };
                ui.label(thinking_text);
            });
        });
    }
}
