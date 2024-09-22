# Fourier Analogue-in-Digital

## プロジェクト概要

[AAPM](https://mikhael-openworkspace.notion.site/Project-Archivist-e512fa7a21474ef6bdbd615a424293cf)@Audio-8151のRust実装です。詳細は[Notion](https://mikhael-openworkspace.notion.site/Fourier-Analogue-in-Digital-d170c1760cbf4bb4aaea9b1f09b7fead?pvs=4)で確認することができます。

## 入出力PCMフォーマット

浮動小数点数

- f16be, f32be, f64be(デフォルト)
- f16le, f32le, f64le

符号付き整数

- s8
- s16be, s24be, s32be, s64be
- s16le, s24le, s32le, s64le

符号なし整数

- u8
- u16be, u24be, u32be, u64be
- u16le, u24le, u32le, u64le

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

**警告： `--release`なしでビルドすると実行速度が極端に遅くなるので、必ず `--release`と一緒にビルドしてください。**

## メタデータJSON例

```json
[
    {"key": "KEY",                          "type": "string", "value": "VALUE"},
    {"key": "AUTHOR",                       "type": "string", "value": "ハンウル"},
    {"key": "キーとStringタイプのエンコーディング", "type": "string", "value": "UTF-8"},
    {"key": "Base64 サポート",                "type": "base64", "value": "QmFzZTY044Gu5L6L"},
    {"key": "ファイルサポート",                 "type": "base64", "value": "5pyA5aSnMjU2IFRpQuOBvuOBp+OCteODneODvOODiA=="},
    {"key": "未対応文字なし",                  "type": "string", "value": "Unicodeにあるどの文字でも互換性があります！"},
    {"key": "重複キーサポート",                 "type": "string", "value": "キーが重複するようにすると？"},
    {"key": "重複キーサポート",                 "type": "string", "value": "パンパカパーン！"},
    {"key": "",                             "type": "string", "value": "キーなしのメタデータもサポート"}
]
```

## 外部リソース

[Rust](https://github.com/rust-lang/rust)

### Cargoクレート

#### ライブラリ用

1. flate2
2. half
3. rustfft

#### アプリ用

1. base64
2. infer
3. rodio
4. same_file
5. serde_json
6. tempfile

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
