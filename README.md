# Disk Check Tool

ディスクのゼロフィル確認とSSDの消去確認を行うLinux専用ツール。

## 機能

- ディスクの一覧表示（USB接続のディスクを含む）
- 3段階のチェックレベル
  - ファストチェック（約5分）
  - スタンダードチェック（約15分）
  - ディープチェック（約30分）
- SSD特有の消去確認
  - SATA SSD対応
  - NVMe SSD対応
- プログレスバーによる進捗表示

## インストール

```bash
# 必要なコマンドのインストール
sudo apt-get update
sudo apt-get install hdparm nvme-cli

# Rustのインストール（未インストールの場合）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# ツールのインストール
cargo install check_disk
```

## 使用方法

```bash
# 管理者権限で実行
sudo check_disk
```

## 注意事項

- このツールはLinux環境専用です
- ディスクの確認には管理者権限（sudo）が必要な場合があります
- 動作確認はLubuntu上で行っています

## システム要件

- Ubuntu/Lubuntu推奨
- 必要なディスク容量: 10GB以上
- 必要なメモリ容量: 2GB以上

## ライセンス

MIT

## 作者

toma4423

