# Fourier Analogue-in-Digital

## プロジェクト概要

[AAPM](https://mikhael-openworkspace.notion.site/Project-Archivist-e512fa7a21474ef6bdbd615a424293cf)@Audio-8151のRust実装です。詳細は[Notion](https://mikhael-openworkspace.notion.site/Fourier-Analogue-in-Digital-d170c1760cbf4bb4aaea9b1f09b7fead?pvs=4)で確認することができます。

## 入出力PCMフォーマット

Float64 Big Endian(チャンネル数とサンプルレートは自由に指定)

フォーマット変換コマンド

```bash
ffmpeg -i audio.flac -f f64be -ar <サンプルレート> -ac <チャンネル数> audio.pcm
...
ffmpeg -f f64be -ar -ar <サンプルレート> -ac <チャンネル数> -i frad_res.pcm -c:a flac res.flac
```

## インストール方法

1. Git cloneでライブラリをダウンロード
2. cargo build --release でビルド
3. target/releaseにある実行ファイルを好きな場所に移動します。
4. PATHに変数を登録

```bash
git clone https://github.com/h4n-ul/FrAD_Rust.git
cd FrAD_Rust
cargo build --release
mv target/release/frad /path/to/bin/frad
export PATH=/path/to/bin:$PATH
```

**警告： --releaseオプションなしでビルドすると実行速度が極端に遅くなるので、必ず--releaseオプションと一緒にビルドしてください。**

## 外部リソース

[Rust](https://github.com/rust-lang/rust)

### Cargoクレート

1. flate2
2. half
3. libsoxr
4. rustfft

## 貢献方法

### FrAD フォーマットへの貢献

FrADフォーマット自体への貢献は[こちら](https://github.com/h4n-ul/Fourier_Analogue-in-Digital)にしていただくか、私に直接メールを書いてください。標準に対する提案、改善点などは、リンク先にあるリポジトリにissueやPRを作成してください。

### Rust実装への貢献

Rust実装に限った貢献なら、このリポジトリに直接投稿してください。機能追加やバグ修正、性能改善など、なんでも歓迎です。

以下はコントリビューションの手順です。

1. リポジトリをフォークする
2. 新しいブランチを作成する
3. 修正を作成し、バグに苦しむ
4. mainブランチにプッシュする
5. このリポジトリでPull Requestを生成する

Pull Requestが生成されたら、検討後、フィードバックをしたり、mergeします。 実際、FrAD標準と互換性があれば、ほとんど問答無用でmergeします。

## 開発者情報

ハンウル, <jun061119@proton.me>
