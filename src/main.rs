use check_disk::check_disk::*;

fn main() {
    println!("ディスクゼロフィル＆SSDチェックツール起動");

    // 1. すべてのディスクを認識
    let devices = list_all_disks();

    // 2. ディスクが見つかった場合のみ選択プロセスを実行
    if let Some(selected_device) = select_disk(devices) {
        let device_path = selected_device.split_whitespace().next().unwrap();

        // 3. チェックレベルを選択
        let num_checks = select_check_level();

        if num_checks == 0 {
            // SSDチェックを実行し、成功/失敗を表示
            let success = check_ssd(device_path);
            if success {
                println!("ディスク {} のSSD消去確認は成功しました。", selected_device);
            } else {
                println!("ディスク {} のSSD消去確認は失敗しました。", selected_device);
            }
        } else {
            // ディスクのセクタ数を取得
            let sector_count = match run_command("blockdev", &["--getsz", device_path]) {
                Ok(output) => output.trim().parse::<usize>().unwrap_or(0),
                Err(e) => {
                    println!("セクタ数の取得に失敗しました: {}", e);
                    return;
                }
            };

            // ディスクのチェックを実行し、成功/失敗を表示
            let success = check_random_sectors(device_path, sector_count, num_checks);
            if success {
                println!("ディスク {} のチェックは成功しました。", selected_device);
            } else {
                println!("ディスク {} のチェックは失敗しました。", selected_device);
            }
        }
    } else {
        println!("ディスクが見つかりませんでした。");
    }

    // Enterキーの入力待ち
    println!("Enterキーを押してプログラムを終了してください。");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("入力エラー");

    println!("プログラム終了。");
}
