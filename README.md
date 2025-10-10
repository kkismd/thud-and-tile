# Thud & Tile

**3色システムを採用したテトリス風パズルゲーム**

## 🎮 ゲーム概要

Thud & Tileは、従来のテトリスに新しい要素を加えたパズルゲームです：

- **3色システム**: シアン、マゼンタ、イエローの3色のみを使用
- **連結システム**: 同色ブロック同士が自動で連結し、数字表示される
- **特殊消去**: ライン消去時にフィールドが永続的に縮小する独自メカニクス
- **戦略性**: 合計スコアと色別MAX-CHAINを活用した高得点システム

## 🚀 実行方法

### CLI版（ターミナル）

```bash
# プロジェクトディレクトリに移動
cd thud-and-tile

# 実行
cargo run

# リリースビルドで実行
cargo run --release
```

### 操作方法（CLI版）
- **A/←**: 左移動
- **D/→**: 右移動  
- **S/↓**: ソフトドロップ
- **W/↑**: 回転
- **Space**: ハードドロップ
- **R**: リスタート
- **Q**: 終了

## 🌐 Web版

Web版は別リポジトリで管理されています：

**👉 [thud-and-tile-web](https://github.com/kkismd/thud-and-tile-web)**

- ブラウザで遊べるWeb版
- タッチ操作対応（モバイル対応）
- GitHub Pagesでデプロイ済み

## 🛠 開発・ビルド

### 依存関係

```bash
# Rustツールチェーンが必要
rustup install stable

# WASM開発用（Web版ビルド時）
cargo install wasm-pack
```

### ビルド

#### CLI版
```bash
# デバッグビルド
cargo build

# リリースビルド
cargo build --release
```

#### Web版用WASMビルド
```bash
# Web版プロジェクトのpkgディレクトリにWASMファイルを生成
wasm-pack build --target web --out-dir ../thud-and-tile-web/pkg --features wasm
```

### テスト実行

```bash
# 全テスト実行
cargo test

# 特定のテストのみ
cargo test tetromino
cargo test board_logic
```

## 📁 プロジェクト構造

```
thud-and-tile/
├── src/
│   ├── main.rs              # CLI版エントリーポイント
│   ├── lib.rs               # 共通ライブラリ + WASM API
│   ├── tetromino.rs         # テトロミノ定義・回転ロジック
│   ├── board_logic.rs       # ボード処理・連結システム
│   ├── animation.rs         # アニメーション共通処理
│   ├── cell.rs              # セル・ボード定義
│   ├── config.rs            # ゲーム設定
│   ├── game_color.rs        # 色定義
│   ├── scoring.rs           # スコアシステム
│   ├── render.rs            # CLI版描画
│   └── tests/               # テストファイル群
├── Cargo.toml               # 依存関係定義
└── README.md                # このファイル
```

## 🔧 技術仕様

### アーキテクチャ
- **共通コア**: Rustで実装されたゲームロジック
- **CLI版**: crossterm使用のターミナルUI
- **Web版**: wasm-bindgen経由でWebAssemblyとして動作

### フィーチャーフラグ
- `native-bin`: CLI版ビルド用（デフォルト）
- `wasm`: Web版WASM用
- `wasm-test`: WASMテスト用

### SRS準拠
- SRS（Super Rotation System）に準拠したテトロミノ回転
- 標準的なWall Kickオフセットテーブル使用

## 🎯 ゲームメカニクス

### 連結システム
1. 同色ブロック同士が上下左右で隣接すると自動連結
2. 連結したブロックは数字表示（連結数）
3. 色別に最大連結数（MAX-CHAIN）を記録

### スコアシステム
- 基本計算式: `消去ブロックの数字 × その色のMAX-CHAIN × 10点`
- 管理する得点は合計スコアのみ（色別スコアは表示・蓄積しない）
- 色別MAX-CHAIN（シアン、マゼンタ、イエロー）は連結戦略の指標として表示される
- Solidライン消去用の貯蓄数はUI上で`10-CHAIN`として数値表示され、上限なく累積する

### フィールド縮小
- ライン消去時に消去ラインは灰色の固定ブロックに変化
- 固定ブロックはフィールド底に沈み、プレイ領域を永続的に縮小
- ゲーム進行とともに難易度が上昇

## 🐛 バグ報告・コントリビューション

### Issue報告
GitHub Issuesで以下の情報と共に報告してください：
- 環境（OS、Rustバージョンなど）
- 再現手順
- 期待される動作と実際の動作

### 開発への参加
1. このリポジトリをフォーク
2. フィーチャーブランチを作成
3. 変更を実装・テスト
4. プルリクエストを送信

## 📚 関連資料

- **ゲーム仕様**: `game_spec.md`
- **開発履歴**: `HISTORY.md`
- **ロードマップ**: `ROADMAP.md`
- **Web版**: [thud-and-tile-web リポジトリ](https://github.com/kkismd/thud-and-tile-web)

## 📄 ライセンス

MIT License - 詳細はLICENSEファイルを参照

## 🎉 楽しいゲーム体験を！

Thud & Tileは従来のテトリスとは異なる戦略性を持つゲームです。色の配置と連結を考慮した戦略的なプレイをお楽しみください！