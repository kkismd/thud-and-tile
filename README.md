# Thud & Tile

**3色システムを採用したテトリス風パズルゲーム**

> **✅ 新メカニクス実装完了** (2025年10月6日更新)  
> CHAIN-BONUSシステム・Solidライン相殺システムが完全実装されました

## 🎮 ゲーム概要

Thud & Tileは、従来のテトリスに新しい要素を加えたパズルゲームです：

- **3色システム**: シアン、マゼンタ、イエローの3色のみを使用
- **連結システム**: 同色ブロック同士が自動で連結し、数字表示される  
- **CHAIN-BONUSシステム**: ピース着地時のMAX-CHAIN増加で蓄積されるボーナス
- **Solidライン相殺**: CHAIN-BONUSを消費してSolidラインを消去するアニメーション
- **統合スコアシステム**: 色別スコア表示から統合TOTAL SCORE表示へ
- **戦略性**: CHAIN-BONUSとSolidライン管理による高度な戦略ゲーム

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
- 色別スコア管理（シアン、マゼンタ、イエロー）
- 戦略的な色配置でハイスコアを狙う

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

### 🎮 ゲーム仕様・履歴
- **ゲーム仕様**: `game_spec.md` - ゲームルールとメカニクス詳細
- **開発履歴**: `HISTORY.md` - プロジェクト開発経緯
- **ロードマップ**: `ROADMAP.md` - 今後の開発計画
- **完了レポート**: `FINAL_COMPLETION_REPORT.md` - Phase 1完了報告

### 🏗️ WASM統合設計（Phase 2準備）
- **統合再設計指針**: `CLI_WASM_INTEGRATION_REDESIGN.md` - 3層分離アーキテクチャ基本方針
- **Phase 1-4分析**: `WASM_REDESIGN_PHASE_ANALYSIS.md` - 4段階設計プロセス管理文書
- **統合実装計画**: `WASM_CORE_INTEGRATION_PLAN.md` - 4フェーズ実装ロードマップ
- **技術仕様書**: `WASM_CORE_INTEGRATION_TECHNICAL.md` - WASM境界安全設計詳細

### 📋 Phase別設計文書
- **Phase 1**: `PHASE1_CORE_MODULE_COMPATIBILITY.md` - Core Module適合性検証結果
- **Phase 2**: `PHASE2_LAYER_SEPARATION_DESIGN.md` - 3層分離アーキテクチャ設計
- **Phase 3**: `PHASE3_WASM_BOUNDARY_REDESIGN.md` - WASM境界安全設計
- **Phase 4**: `PHASE4_INTEGRATION_PLAN_REBUILT.md` - 統合計画再構築完了報告

### 🛡️ リスク管理・移行戦略
- **包括的リスク管理**: `MIGRATION_RISK_COMPREHENSIVE_ANALYSIS.md` - WASM統合全体のリスク管理マスター文書
- **CLI移行戦略**: `PHASE2_CLI_MIGRATION_STRATEGY.md` - CLI Layer移行の詳細戦略・テクニック
- **Phase 2計画改訂**: `PHASE2_PLAN_REVISION.md` - 段階的移行戦略とロールバック計画

### 🌐 Web版
- **Web版リポジトリ**: [thud-and-tile-web](https://github.com/kkismd/thud-and-tile-web)

## 📄 ライセンス

MIT License - 詳細はLICENSEファイルを参照

## 🎉 楽しいゲーム体験を！

Thud & Tileは従来のテトリスとは異なる戦略性を持つゲームです。色の配置と連結を考慮した戦略的なプレイをお楽しみください！