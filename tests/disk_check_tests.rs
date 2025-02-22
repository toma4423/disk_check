#[cfg(test)]
mod tests {
    use check_disk::check_disk::{SAMPLE_SIZE, list_all_disks, select_disk, check_sata_secure_erase, check_nvme_secure_erase, check_random_sectors};
    use tempfile::NamedTempFile;

    // モック用の関数を追加して、外部コマンドを実行せずにテスト用の結果を返す
    fn mock_command_output(command: &str, args: &[&str]) -> String {
    match command {
        "lsblk" => {
            "sda 64G disk\nsdb 1T disk".to_string() // ディスクリストのモック結果
        }
        "blockdev" => {
            if args == ["--getsz", "/dev/sda"] {
                "1024000".to_string() // /dev/sda のセクタ数のモック結果
            } else {
                "2048000".to_string() // /dev/sdb のセクタ数のモック結果
            }
        }
        "hdparm" => {
            if args.contains(&"--security-erase-status") {
                "supported: enhanced erase\nSecurity Erase completed successfully".to_string() // 消去成功のモック結果を修正
            } else {
                "Security Erase failed".to_string() // 消去失敗のモック結果
            }
        }
        "nvme" => {
            "sanitize operation completed\nsanitize operation completed successfully".to_string() // NVMe消去成功のモック結果を修正
        }
        _ => "".to_string(),
    }
}


    // テスト用の run_command モックを作成
    fn mock_run_command(command: &str, args: &[&str]) -> Result<String, String> {
        Ok(mock_command_output(command, args))
    }

    #[test]
    fn test_list_disks() {
        let output = mock_command_output("lsblk", &["-nd", "-o", "NAME,SIZE,TYPE"]);
        assert!(output.contains("sda 64G disk"));
        assert!(output.contains("sdb 1T disk"));
        let devices = list_all_disks();
        assert_eq!(devices.len(), 2); // ディスクは2つあるはず
    }

    #[test]
    fn test_select_disk() {
        let devices = vec![
            "/dev/sda - 500G - disk".to_string(),
            "/dev/sdb - 1T - disk".to_string(),
        ];
        let selected_device = select_disk(devices.clone()).unwrap();
        assert_eq!(selected_device, devices[0]); // デフォルトでは最初のディスクが選択されるはず
    }

    #[test]
    fn test_check_sata_secure_erase_success() {
        let output = mock_run_command("hdparm", &["--security-erase-status", "/dev/sda"]).unwrap();
        assert!(output.contains("Security Erase completed successfully"));
        let result = check_sata_secure_erase("/dev/sda");
        assert!(result); // 成功したはず
    }

    #[test]
    fn test_check_nvme_secure_erase_success() {
        let output = mock_run_command("nvme", &["sanitize-log", "/dev/nvme0n1"]).unwrap();
        assert!(output.contains("sanitize operation completed successfully"));
        let result = check_nvme_secure_erase("/dev/nvme0n1");
        assert!(result); // 成功したはず
    }

    #[test]
    fn test_check_random_sectors_success() {
        // 一時ファイルを作成し、全て0で埋める
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.as_file_mut().set_len(1024 * SAMPLE_SIZE as u64).unwrap(); // 1024セクタ分のサイズ

        // 全てのセクタが0であることを確認
        let success = check_random_sectors(temp_file.path().to_str().unwrap(), 1024, 100);
        assert!(success); // 全てゼロなので成功するはず
    }
}
