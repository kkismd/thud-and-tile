# Phase 2A開発引き継ぎ資料

**作成日時**: 2025年10月2日  
**フェーズ**: Phase 2A - Critical機能実装  
**進捗状況**: 自動落下システム完了、残り2項目継続中

## 🎯 現在の状況

### ✅ 完了済み項目
1. **Timer/Scheduler実装** - WASMTimeProviderによる時間管理基盤
2. **Auto-fall Logic移植** - 800ms間隔での自動落下システム  
3. **JavaScript Timer統合** - setIntervalによるゲームループ

### 🚨 **現在発生中の問題**
**問題**: HTMLの`start-button`で`Uncaught ReferenceError: startGame is not defined`エラー
**原因**: TypeScriptの関数がHTMLからアクセスできない（スコープ問題）
**場所**: `/thud-and-tile-web/index.html` line 152

### 📋 継続タスク
- [ ] **緊急**: startGameエラーの修正
- [ ] **次ピース表示実装**
- [ ] **ゴーストピース実装**

## 🏗️ プロジェクト構造

### メインプロジェクト（Rustゲームロジック）
```
/Users/kshimada/Documents/move_folder/src/Rust/block-down/
├── src/
│   ├── lib.rs          # WASM API実装（自動落下機能追加済み）
│   ├── main.rs         # CLI版（完全実装済み）
│   ├── config.rs       # 定数定義
│   ├── tetromino.rs    # テトロミノロジック
│   └── ...
└── Cargo.toml
```

### Webプロジェクト（フロントエンド）
```
/Users/kshimada/Documents/move_folder/src/Rust/thud-and-tile-web/
├── src/
│   ├── main.ts         # TypeScript実装（自動落下機能追加済み）
│   ├── main_original.ts # オリジナルバックアップ
│   └── main.ts.backup  # バックアップ
├── index.html          # HTML UI（問題箇所あり）
├── pkg/               # WASMビルド出力
└── package.json
```

## 🔧 技術実装詳細

### 自動落下システム（完了済み）

#### Rust側（lib.rs）
```rust
// 新規追加されたフィールド
pub struct WasmGameState {
    fall_speed: Duration,           // 落下速度（800ms）
    last_fall_time: Duration,       // 最後の落下時刻
    time_provider: WasmTimeProvider, // 時間管理
    // ... 既存フィールド
}

// 新規追加されたメソッド
#[wasm_bindgen]
impl WasmGameState {
    pub fn auto_fall(&mut self) -> bool { ... }      // 自動落下処理
    pub fn get_fall_speed_ms(&self) -> u32 { ... }   // 速度取得
    pub fn set_fall_speed_ms(&mut self, ms: u32) { ... } // 速度設定
}
```

#### TypeScript側（main.ts）
```typescript
// 新規追加された変数とメソッド
let autoFallInterval: number | null = null;

function startAutoFall() { ... }    // タイマー開始
function stopAutoFall() { ... }     // タイマー停止
function startGame() { ... }        // ゲーム開始（問題箇所）
```

## 🚨 緊急修正が必要な問題

### 問題: startGame関数へのアクセスエラー

**エラー内容**:
```
Uncaught ReferenceError: startGame is not defined
```

**発生場所**:
```html
<!-- /thud-and-tile-web/index.html line 152 -->
<button id="start-button" onclick="startGame()">ゲーム開始</button>
```

**原因分析**:
1. TypeScriptの`startGame()`関数がモジュールスコープ内に定義されている
2. HTMLの`onclick`属性はグローバルスコープを参照する
3. ViteのESモジュールシステムで関数が自動的にグローバル公開されない

**解決方法（2つの選択肢）**:

#### 方法1: HTMLのonclick削除（推奨）
```html
<!-- 修正前 -->
<button id="start-button" onclick="startGame()">ゲーム開始</button>

<!-- 修正後 -->
<button id="start-button">ゲーム開始</button>
```

TypeScript側で既に`setupEventListeners()`でイベントリスナー設定済み：
```typescript
const startButton = document.getElementById('start-button'); // ※IDが違う
if (startButton) {
    startButton.addEventListener('click', startGame);
}
```

**重要**: HTMLのid `start-button` とTypeScriptで探している`start-game`が不一致！

#### 方法2: グローバル公開（簡易）
```typescript
// main.tsの最後に追加
(window as any).startGame = startGame;
```

## 🔧 即座に実行すべき修正手順

### Step 1: HTMLのID修正
```html
<!-- 修正前 -->
<button id="start-button" onclick="startGame()">ゲーム開始</button>

<!-- 修正後 -->
<button id="start-game">ゲーム開始</button>
```

### Step 2: 開発環境の起動
```bash
# Webプロジェクトディレクトリに移動
cd /Users/kshimada/Documents/move_folder/src/Rust/thud-and-tile-web

# WASMをリビルド（自動落下機能を反映）
cd ../block-down
wasm-pack build --target web --out-dir ../thud-and-tile-web/pkg

# 開発サーバー起動
cd ../thud-and-tile-web
npm run dev
```

### Step 3: ブラウザテスト
1. http://localhost:5173/ にアクセス
2. "ゲーム開始"ボタンをクリック
3. コンソールで`自動落下タイマー開始: 800ms間隔`を確認
4. ピースが800ms間隔で自動落下することを確認

## 📈 Phase 2A残りタスク

### 次ピース表示実装（優先度2位）

**現在の状況**:
- Rust側: `next_piece: Option<SimpleTetromino>` フィールド存在
- TypeScript側: 表示UI未実装

**実装場所**:
1. **UI追加**: index.htmlに次ピース表示エリア追加
2. **取得API**: lib.rsに`get_next_piece_info()`メソッド追加
3. **描画処理**: main.tsに次ピース描画処理追加

### ゴーストピース実装（優先度3位）

**CLIリファレンス**:
```rust
// main.rs line 126-133
fn ghost_piece(&self) -> Option<Tetromino> {
    self.current_piece.as_ref().map(|piece| {
        let mut ghost = piece.clone();
        while self.is_valid_position(&ghost.moved(0, 1)) {
            ghost = ghost.moved(0, 1);
        }
        ghost
    })
}
```

**移植タスク**:
1. **Rust側**: `ghost_piece()`メソッドをWASM用に移植
2. **TypeScript側**: 半透明描画処理実装

## 🔄 開発環境セットアップ

### VS Code Workspace推奨設定
```bash
# 両プロジェクトを管理するため親ディレクトリで起動
cd /Users/kshimada/Documents/move_folder/src/Rust
code .
```

### 必要な拡張機能
- Rust Analyzer
- TypeScript and JavaScript Language Features
- ES6 snippets
- Error Lens

### ターミナル設定
```bash
# Terminal 1: Rustプロジェクト用
cd /Users/kshimada/Documents/move_folder/src/Rust/block-down

# Terminal 2: Webプロジェクト用  
cd /Users/kshimada/Documents/move_folder/src/Rust/thud-and-tile-web

# Terminal 3: 開発サーバー用
cd /Users/kshimada/Documents/move_folder/src/Rust/thud-and-tile-web
npm run dev
```

## 📝 重要なファイル状態

### 変更されたファイル（git status確認推奨）
- `src/lib.rs` - 自動落下機能追加
- `../thud-and-tile-web/src/main.ts` - 自動落下統合
- `mobile_web_migration_proposal.md` - Phase1完了レポート更新

### バックアップファイル
- `../thud-and-tile-web/src/main_original.ts` - 元のTypeScript実装
- `../thud-and-tile-web/src/main.ts.backup` - バックアップ

## 🎯 成功指標

### Phase 2A完全完了の条件
1. ✅ **自動落下**: 800ms間隔で動作確認済み
2. ⏳ **次ピース表示**: UIに次のテトロミノが表示される
3. ⏳ **ゴーストピース**: 現在ピースの着地予測が半透明で表示される

### デバッグ用コマンド
```bash
# WASMリビルド
wasm-pack build --target web --out-dir ../thud-and-tile-web/pkg

# Viteキャッシュクリア
npm run dev -- --force

# ブラウザコンソールで確認すべきログ
# - "自動落下タイマー開始: 800ms間隔"
# - "自動落下実行" (800ms間隔)
```

## 💡 トラブルシューティング

### よくある問題
1. **WASM更新が反映されない** → WASMリビルド + ブラウザリロード
2. **TypeScript変更が反映されない** → Viteホットリロード確認
3. **関数が見つからない** → グローバルスコープとモジュールスコープの違い確認

### 開発フロー
1. Rust変更 → `wasm-pack build`
2. TypeScript変更 → 自動リロード
3. HTML変更 → 自動リロード
4. テスト → ブラウザで動作確認

---

**次のセッション開始時**: この資料を確認後、まず緊急修正（startGameエラー）を実行してください。その後、次ピース表示実装に進むことを推奨します。