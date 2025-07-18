use crate::stats::{GameResult, GameStats};
use chrono::Local;
use plotters::prelude::*;
use std::error::Error;

/// ゲーム統計のグラフを生成する
pub fn plot_game_statistics(
    stats: &GameStats,
    game_result: &GameResult,
) -> Result<(), Box<dyn Error>> {
    // タイムスタンプ付きのファイル名を生成
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let base_filename = format!("game_stats_{}", timestamp);

    // 各種グラフを生成
    plot_disc_count_history(stats, &format!("{}_disc_count.png", base_filename))?;
    plot_thinking_time_history(stats, &format!("{}_thinking_time.png", base_filename))?;
    plot_evaluation_history(stats, &format!("{}_evaluation.png", base_filename))?;
    plot_combined_overview(
        stats,
        game_result,
        &format!("{}_overview.png", base_filename),
    )?;

    println!("\nグラフファイルを生成しました:");
    println!("・石数推移: {}_disc_count.png", base_filename);
    println!("・思考時間: {}_thinking_time.png", base_filename);
    println!("・評価値推移: {}_evaluation.png", base_filename);
    println!("・総合グラフ: {}_overview.png", base_filename);

    Ok(())
}

/// 石数の推移グラフを作成
fn plot_disc_count_history(stats: &GameStats, filename: &str) -> Result<(), Box<dyn Error>> {
    let disc_history = stats.get_disc_count_history();
    if disc_history.is_empty() {
        return Ok(());
    }

    let root = BitMapBackend::new(filename, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_move = disc_history.iter().map(|(m, _, _)| *m).max().unwrap_or(1);
    let max_count = disc_history
        .iter()
        .map(|(_, b, w)| (*b).max(*w))
        .max()
        .unwrap_or(32);
    let min_count = disc_history
        .iter()
        .map(|(_, b, w)| (*b).min(*w))
        .min()
        .unwrap_or(0);

    let mut chart = ChartBuilder::on(&root)
        .caption("石数の推移", ("sans-serif", 40))
        .margin(10)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(0..max_move, (min_count.saturating_sub(2))..(max_count + 2))?;

    chart
        .configure_mesh()
        .x_desc("手数")
        .y_desc("石数")
        .draw()?;

    // 黒の石数
    chart
        .draw_series(LineSeries::new(
            disc_history.iter().map(|(m, b, _)| (*m, *b)),
            &BLACK,
        ))?
        .label("黒")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &BLACK));

    // 白の石数
    chart
        .draw_series(LineSeries::new(
            disc_history.iter().map(|(m, _, w)| (*m, *w)),
            &BLUE,
        ))?
        .label("白")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &BLUE));

    chart.configure_series_labels().draw()?;
    root.present()?;

    Ok(())
}

/// 思考時間の推移グラフを作成
fn plot_thinking_time_history(stats: &GameStats, filename: &str) -> Result<(), Box<dyn Error>> {
    let time_history = stats.get_thinking_time_history();
    if time_history.is_empty() {
        return Ok(());
    }

    let root = BitMapBackend::new(filename, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_move = time_history.iter().map(|(m, _)| *m).max().unwrap_or(1);
    let max_time = time_history
        .iter()
        .map(|(_, t)| *t)
        .fold(0.0f64, |a, b| a.max(b));
    let min_time = time_history
        .iter()
        .map(|(_, t)| *t)
        .fold(f64::INFINITY, |a, b| a.min(b))
        .max(0.0);

    let mut chart = ChartBuilder::on(&root)
        .caption("思考時間の推移", ("sans-serif", 40))
        .margin(10)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(0..max_move, (min_time - 0.1)..(max_time + 0.1))?;

    chart
        .configure_mesh()
        .x_desc("手数")
        .y_desc("思考時間 (秒)")
        .draw()?;

    // 思考時間の折れ線グラフ
    chart.draw_series(LineSeries::new(
        time_history.iter().map(|(m, t)| (*m, *t)),
        &RED,
    ))?;

    // 平均線を追加
    if !time_history.is_empty() {
        let avg_time: f64 =
            time_history.iter().map(|(_, t)| t).sum::<f64>() / time_history.len() as f64;
        chart
            .draw_series(LineSeries::new(
                vec![(0, avg_time), (max_move, avg_time)],
                GREEN.stroke_width(2),
            ))?
            .label(format!("平均: {:.2}秒", avg_time))
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &GREEN));

        chart.configure_series_labels().draw()?;
    }

    root.present()?;

    Ok(())
}

/// 評価値の推移グラフを作成
fn plot_evaluation_history(stats: &GameStats, filename: &str) -> Result<(), Box<dyn Error>> {
    let eval_history = stats.get_evaluation_history();
    if eval_history.is_empty() {
        return Ok(());
    }

    let root = BitMapBackend::new(filename, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_move = eval_history.iter().map(|(m, _, _)| *m).max().unwrap_or(1);
    let max_eval = eval_history.iter().map(|(_, _, e)| *e).max().unwrap_or(100);
    let min_eval = eval_history
        .iter()
        .map(|(_, _, e)| *e)
        .min()
        .unwrap_or(-100);

    let margin = (max_eval - min_eval).max(100) / 10;

    let mut chart = ChartBuilder::on(&root)
        .caption("AI評価値の推移", ("sans-serif", 40))
        .margin(10)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(0..max_move, (min_eval - margin)..(max_eval + margin))?;

    chart
        .configure_mesh()
        .x_desc("手数")
        .y_desc("評価値")
        .draw()?;

    // プレイヤー別に色分け
    use crate::player::Player;

    let black_moves: Vec<_> = eval_history
        .iter()
        .filter(|(_, player, _)| *player == Player::Black)
        .map(|(m, _, e)| (*m, *e))
        .collect();

    let white_moves: Vec<_> = eval_history
        .iter()
        .filter(|(_, player, _)| *player == Player::White)
        .map(|(m, _, e)| (*m, *e))
        .collect();

    if !black_moves.is_empty() {
        chart
            .draw_series(LineSeries::new(black_moves, &BLACK))?
            .label("黒AI評価値")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &BLACK));
    }

    if !white_moves.is_empty() {
        chart
            .draw_series(LineSeries::new(white_moves, &BLUE))?
            .label("白AI評価値")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &BLUE));
    }

    // ゼロラインを追加
    chart.draw_series(LineSeries::new(
        vec![(0, 0), (max_move, 0)],
        RGBColor(128, 128, 128).stroke_width(1),
    ))?;

    chart.configure_series_labels().draw()?;
    root.present()?;

    Ok(())
}

/// 総合概要グラフを作成（複数のサブプロットを含む）
fn plot_combined_overview(
    stats: &GameStats,
    game_result: &GameResult,
    filename: &str,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(filename, (1200, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    let areas = root.split_evenly((2, 1));
    let upper = &areas[0];
    let lower = &areas[1];
    let upper_areas = upper.split_evenly((1, 2));
    let upper_left = &upper_areas[0];
    let upper_right = &upper_areas[1];

    // 上左: 石数推移
    plot_disc_overview(&upper_left, stats)?;

    // 上右: 思考時間
    plot_thinking_time_overview(&upper_right, stats)?;

    // 下: ゲーム結果サマリー
    plot_game_summary(&lower, game_result)?;

    root.present()?;
    Ok(())
}

fn plot_disc_overview(
    area: &DrawingArea<BitMapBackend, plotters::coord::Shift>,
    stats: &GameStats,
) -> Result<(), Box<dyn Error>> {
    let disc_history = stats.get_disc_count_history();
    if disc_history.is_empty() {
        return Ok(());
    }

    area.fill(&WHITE)?;

    let max_move = disc_history.iter().map(|(m, _, _)| *m).max().unwrap_or(1);
    let max_count = disc_history
        .iter()
        .map(|(_, b, w)| (*b).max(*w))
        .max()
        .unwrap_or(32);
    let min_count = disc_history
        .iter()
        .map(|(_, b, w)| (*b).min(*w))
        .min()
        .unwrap_or(0);

    let mut chart = ChartBuilder::on(area)
        .caption("石数推移", ("sans-serif", 20))
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0..max_move, (min_count.saturating_sub(2))..(max_count + 2))?;

    chart
        .configure_mesh()
        .x_desc("手数")
        .y_desc("石数")
        .draw()?;

    // 黒の石数
    chart.draw_series(LineSeries::new(
        disc_history.iter().map(|(m, b, _)| (*m, *b)),
        &BLACK,
    ))?;

    // 白の石数
    chart.draw_series(LineSeries::new(
        disc_history.iter().map(|(m, _, w)| (*m, *w)),
        &BLUE,
    ))?;

    Ok(())
}

fn plot_thinking_time_overview(
    area: &DrawingArea<BitMapBackend, plotters::coord::Shift>,
    stats: &GameStats,
) -> Result<(), Box<dyn Error>> {
    let time_history = stats.get_thinking_time_history();
    if time_history.is_empty() {
        return Ok(());
    }

    area.fill(&WHITE)?;

    let max_move = time_history.iter().map(|(m, _)| *m).max().unwrap_or(1);
    let max_time = time_history
        .iter()
        .map(|(_, t)| *t)
        .fold(0.0f64, |a, b| a.max(b));
    let min_time = time_history
        .iter()
        .map(|(_, t)| *t)
        .fold(f64::INFINITY, |a, b| a.min(b))
        .max(0.0);

    let mut chart = ChartBuilder::on(area)
        .caption("思考時間", ("sans-serif", 20))
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0..max_move, (min_time - 0.1)..(max_time + 0.1))?;

    chart.configure_mesh().x_desc("手数").y_desc("秒").draw()?;

    // 思考時間の折れ線グラフ
    chart.draw_series(LineSeries::new(
        time_history.iter().map(|(m, t)| (*m, *t)),
        &RED,
    ))?;

    Ok(())
}

fn plot_game_summary(
    area: &DrawingArea<BitMapBackend, plotters::coord::Shift>,
    game_result: &GameResult,
) -> Result<(), Box<dyn Error>> {
    area.fill(&WHITE)?;

    // ゲーム結果のテキスト表示
    let winner_text = match game_result.winner {
        Some(crate::player::Player::Black) => "勝者: 黒",
        Some(crate::player::Player::White) => "勝者: 白",
        None => "引き分け",
    };

    let text_style = ("sans-serif", 30);

    area.draw(&Text::new(
        format!("{}", winner_text),
        (50, 50),
        &text_style.into_font().color(&BLACK),
    ))?;

    area.draw(&Text::new(
        format!(
            "最終スコア - 黒: {} 白: {}",
            game_result.black_final_count, game_result.white_final_count
        ),
        (50, 100),
        &text_style.into_font().color(&BLACK),
    ))?;

    area.draw(&Text::new(
        format!("総手数: {}", game_result.total_moves),
        (50, 150),
        &text_style.into_font().color(&BLACK),
    ))?;

    area.draw(&Text::new(
        format!(
            "ゲーム時間: {:.1}秒",
            game_result.game_duration.as_secs_f64()
        ),
        (50, 200),
        &text_style.into_font().color(&BLACK),
    ))?;

    Ok(())
}
