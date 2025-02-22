pub mod check_disk {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};
    use indicatif::{ProgressBar, ProgressStyle};
    use std::thread;
    use std::time::Duration;
    use rand::Rng;

    // 各チェックレベルに応じたランダムアクセス回数
    pub const FAST_CHECK_COUNT: usize = 24900;
    pub const STANDARD_CHECK_COUNT: usize = 74700;
    pub const DEEP_CHECK_COUNT: usize = 149400;
    pub const SAMPLE_SIZE: usize = 512;  // 1セクタのサイズ（512バイト）

    // コマンドを実行し、結果を文字列として返す関数
    pub fn run_command(command: &str, args: &[&str]) -> Result<String, String> {
        let output = std::process::Command::new(command).args(args).output();
        match output {
            Ok(output) => Ok(String::from_utf8_lossy(&output.stdout).to_string()),
            Err(e) => Err(format!("コマンドの実行に失敗しました: {:?}", e)),
        }
    }

    // すべてのディスクのリストを表示 (USB接続のディスクも含む)
    pub fn list_all_disks() -> Vec<String> {
        match run_command("lsblk", &["-o", "NAME,SIZE,TYPE,TRAN"]) {
            Ok(output) => {
                let devices: Vec<String> = output
                    .lines()
                    .filter(|line| line.contains("disk"))  // すべてのディスク
                    .map(|line| {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        let disk_type = if parts.contains(&"ssd") { "SSD" } else { "HDD" };
                        format!("/dev/{} - {} - {} - {}", parts[0], parts[1], parts[2], disk_type)
                    })
                    .collect();
                devices
            }
            Err(err) => {
                println!("ディスクのリスト取得に失敗しました: {}", err);
                vec![]  // エラー時には空のリストを返す
            }
        }
    }

    // ランダムなセクタをチェックし、ゼロフィルされているか確認する
    pub fn check_random_sectors(device: &str, sector_count: usize, num_checks: usize) -> bool {
        let mut file = match File::open(device) {
            Ok(f) => f,
            Err(e) => {
                println!("ディスクを開けませんでした: {}", e);
                return false;
            }
        };

        let mut rng = rand::thread_rng();
        let bar = ProgressBar::new(num_checks as u64);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{bar:40.cyan/blue} {pos}/{len} チェック中...")
                .expect("プログレスバーのテンプレート設定に失敗しました")
        );

        let mut buffer = vec![0u8; SAMPLE_SIZE];

        for _ in 0..num_checks {
            let random_sector = rng.gen_range(0..sector_count);
            let offset = random_sector as u64 * SAMPLE_SIZE as u64;

            if file.seek(SeekFrom::Start(offset)).is_err() {
                println!("ディスク位置のシークに失敗しました。");
                return false;
            }

            if file.read_exact(&mut buffer).is_err() {
                println!("セクタの読み取りに失敗しました。");
                return false;
            }

            // セクタがすべてゼロであるか確認
            if !buffer.iter().all(|&byte| byte == 0) {
                println!("ゼロでないデータが検出されました。");
                return false;
            }

            bar.inc(1);
            thread::sleep(Duration::from_millis(1));  // 進捗バーを見せるためにスリープ
        }

        bar.finish_with_message("チェック完了");
        true
    }

    // ディスク選択
    pub fn select_disk(devices: Vec<String>) -> Option<String> {
        if devices.is_empty() {
            println!("選択可能なディスクがありません。");
            return None;
        }

        println!("チェックするディスクを選んでください:");
        for (i, device) in devices.iter().enumerate() {
            println!("{}. {}", i + 1, device);
        }

        loop {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).expect("入力エラー");
            if let Ok(index) = input.trim().parse::<usize>() {
                if index > 0 && index <= devices.len() {
                    return Some(devices[index - 1].clone());
                } else {
                    println!("無効な選択です。範囲内の数字を入力してください。");
                }
            } else {
                println!("無効な入力です。数字を入力してください。");
            }
        }
    }

    // チェックレベル選択
    pub fn select_check_level() -> usize {
        println!("チェックレベルを選んでください:");
        println!("1. ファストチェック (約5分)");
        println!("2. スタンダードチェック (約15分)");
        println!("3. ディープチェック (約30分)");
        println!("4. SSDチェック (Secure Erase確認)");

        loop {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).expect("入力エラー");
            match input.trim().parse::<usize>() {
                Ok(choice) => match choice {
                    1 => return FAST_CHECK_COUNT,
                    2 => return STANDARD_CHECK_COUNT,
                    3 => return DEEP_CHECK_COUNT,
                    4 => return 0, // SSDチェックではゼロフィルチェックの回数は必要ない
                    _ => println!("無効な選択肢です。1から4の数字を入力してください。"),
                },
                Err(_) => println!("無効な入力です。数字を入力してください。"),
            }
        }
    }

    // SATA SSDの消去ステータス確認
    pub fn check_sata_secure_erase(device: &str) -> bool {
        match run_command("hdparm", &["-I", device]) {
            Ok(output) => {
                if output.contains("supported: enhanced erase") {
                    println!("SATA SSDの消去プロセスが確認されました。");
                    true
                } else {
                    println!("SATA SSDの消去プロセスが確認できませんでした。");
                    false
                }
            }
            Err(e) => {
                println!("SATA SSDの消去確認に失敗しました: {}", e);
                false
            }
        }
    }

    // NVMe SSDの消去ステータス確認
    pub fn check_nvme_secure_erase(device: &str) -> bool {
        match run_command("nvme", &["sanitize-log", device]) {
            Ok(output) => {
                if output.contains("completed") {
                    println!("NVMe SSDの消去プロセスが確認されました。");
                    true
                } else {
                    println!("NVMe SSDの消去プロセスが確認できませんでした。");
                    false
                }
            }
            Err(e) => {
                println!("NVMe SSDの消去確認に失敗しました: {}", e);
                false
            }
        }
    }

    // SSDの消去確認
    pub fn check_ssd(device: &str) -> bool {
        if device.contains("nvme") {
            check_nvme_secure_erase(device)
        } else {
            check_sata_secure_erase(device)
        }
    }
}
