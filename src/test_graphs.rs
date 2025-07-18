use crate::player::Player;
use crate::stats::{GameResult, GameStats};
use std::time::Duration;

/// テスト用のサンプルデータでグラフを生成する
pub fn generate_test_graphs() -> Result<(), Box<dyn std::error::Error>> {
    println!("テスト用グラフを生成中...");

    // サンプルゲーム統計を作成
    let mut stats = GameStats::new();

    // サンプルの手を記録（短いゲームをシミュレート）
    let moves = vec![
        (Player::Black, Some((2, 3)), 500, 3, 1, Some(-50)),
        (Player::White, Some((3, 5)), 800, 2, 3, Some(30)),
        (Player::Black, Some((4, 2)), 600, 4, 2, Some(-20)),
        (Player::White, Some((5, 4)), 700, 3, 4, Some(40)),
        (Player::Black, Some((2, 4)), 450, 5, 3, Some(10)),
        (Player::White, Some((1, 3)), 900, 4, 5, Some(-10)),
        (Player::Black, Some((0, 3)), 550, 6, 4, Some(25)),
        (Player::White, Some((3, 6)), 650, 5, 6, Some(-5)),
        (Player::Black, Some((4, 5)), 400, 7, 5, Some(35)),
        (Player::White, Some((5, 6)), 750, 6, 7, Some(15)),
        (Player::Black, Some((6, 5)), 500, 8, 6, Some(20)),
        (Player::White, Some((7, 4)), 600, 7, 8, Some(-25)),
        (Player::Black, Some((6, 3)), 350, 9, 7, Some(45)),
        (Player::White, Some((5, 2)), 800, 8, 9, Some(-15)),
        (Player::Black, Some((4, 1)), 480, 10, 8, Some(30)),
        (Player::White, Some((3, 0)), 700, 9, 10, Some(5)),
        (Player::Black, Some((2, 1)), 420, 11, 9, Some(40)),
        (Player::White, Some((1, 2)), 650, 10, 11, Some(-20)),
        (Player::Black, Some((0, 1)), 380, 12, 10, Some(50)),
        (Player::White, Some((1, 0)), 720, 11, 12, Some(-30)),
    ];

    // 手を統計に記録
    for (player, position, thinking_ms, black_count, white_count, evaluation) in moves {
        stats.record_move(
            player,
            position,
            Duration::from_millis(thinking_ms),
            black_count,
            white_count,
            evaluation,
        );
    }

    // ゲーム結果を作成
    let game_result = GameResult {
        winner: Some(Player::Black),
        black_final_count: 12,
        white_final_count: 12,
        total_moves: 20,
        game_duration: Duration::from_secs(15),
        total_thinking_time: Duration::from_secs(12),
    };

    // グラフを生成
    match crate::stats::plot_game_statistics(&stats, &game_result) {
        Ok(()) => {
            println!("✓ テストグラフが正常に生成されました！");
            println!("生成されたファイル:");

            // 生成されたファイルを確認
            use std::fs;
            let current_dir = std::env::current_dir()?;
            let entries = fs::read_dir(&current_dir)?;

            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "png" {
                        if let Some(filename) = path.file_name() {
                            println!("  - {}", filename.to_string_lossy());
                        }
                    }
                }
            }

            Ok(())
        }
        Err(e) => {
            println!("❌ グラフ生成でエラーが発生しました: {}", e);
            Err(e)
        }
    }
}

/// メイン関数から呼び出されるテスト実行関数
pub fn run_graph_test() {
    println!("\n==========================");
    println!("    グラフ生成テスト");
    println!("==========================");

    match generate_test_graphs() {
        Ok(()) => println!("テスト完了: グラフファイルを確認してください"),
        Err(e) => println!("テスト失敗: {}", e),
    }
}
