# 🚀 Phase 2A クイックスタートガイド

**新しいセッション用の即座実行スクリプト**

## 📋 Step-by-Step実行手順

### 0. 🏗️ プロジェクト構造正規化（最優先）

#### Rustプロジェクトのディレクトリ名を正規化
```bash
# 現在のblock-downを正式名称thud-and-tileに変更
cd /Users/kshimada/Documents/move_folder/src/Rust
mv block-down thud-and-tile

# 変更後の構造確認
ls -la
# 期待結果: thud-and-tile/ と thud-and-tile-web/ が並列に存在
```

#### パス更新が必要な設定ファイル確認
```bash
# Webプロジェクトのpkg参照パス確認
cd thud-and-tile-web
grep -r "block-down" . || echo "パス参照問題なし"

# WASMビルド出力パスが正しく設定されているか確認
cd ../thud-and-tile
grep -r "../thud-and-tile-web" . || echo "相対パス問題なし"
```

### 1. 環境セットアップ（推奨）
```bash
# 両プロジェクトを管理するため親ディレクトリでVS Code起動
cd /Users/kshimada/Documents/move_folder/src/Rust
code .
```

### 2. 🚨 緊急修正：startGameエラー解決

#### ターミナル実行コマンド:
```bash
# Webプロジェクトに移動（ディレクトリ名変更後）
cd /Users/kshimada/Documents/move_folder/src/Rust/thud-and-tile-web

# HTMLファイルのID不整合を修正
sed -i.bak 's/id="start-button"/id="start-game"/g' index.html
sed -i.bak 's/onclick="startGame()"//g' index.html

# 同時にrestartボタンも修正
sed -i.bak 's/onclick="restartGame()"//g' index.html

# 修正結果確認
grep -n "start-game\|restart" index.html
```

### 3. 動作テスト
```bash
# WASMリビルド（新しいパスで）
cd ../thud-and-tile
wasm-pack build --target web --out-dir ../thud-and-tile-web/pkg

# 開発サーバー起動
cd ../thud-and-tile-web
npm run dev
```

ブラウザで http://localhost:5173/ にアクセス
→ "ゲーム開始"ボタンクリック
→ コンソールで「自動落下タイマー開始: 800ms間隔」確認

### 4. 次のタスク開始

#### 次ピース表示実装（優先度1位）

**Phase A: Rust API追加**
```rust
// thud-and-tile/src/lib.rs のWasmGameStateに追加
#[wasm_bindgen]
pub fn get_next_piece_info(&self) -> Vec<u32> {
    if let Some(ref piece) = self.next_piece {
        vec![piece.x as u32, piece.y as u32, piece.rotation as u32, piece.color as u32, piece.shape as u32]
    } else {
        vec![]
    }
}
```

**Phase B: HTML UI追加**
```html
<!-- thud-and-tile-web/index.htmlのgame-infoセクションに追加 -->
<div class="info-panel">
    <h3>Next Piece</h3>
    <canvas id="next-piece-canvas" width="120" height="120"></canvas>
</div>
```

**Phase C: TypeScript描画実装**
```typescript
// thud-and-tile-web/src/main.tsに次ピース描画関数追加
function drawNextPiece() {
    const nextCanvas = document.getElementById('next-piece-canvas') as HTMLCanvasElement;
    if (!nextCanvas || !gameState) return;
    
    const nextCtx = nextCanvas.getContext('2d');
    if (!nextCtx) return;
    
    // 背景クリア
    nextCtx.fillStyle = '#000000';
    nextCtx.fillRect(0, 0, nextCanvas.width, nextCanvas.height);
    
    // 次ピース情報取得
    const nextPieceInfo = gameState.get_next_piece_info();
    if (nextPieceInfo.length >= 5) {
        const [x, y, rotation, color, shape] = nextPieceInfo;
        // テトロミノ描画処理
        drawTetrominoOnCanvas(nextCtx, 0, 0, shape, color, nextCanvas.width / 4);
    }
}

// gameLoop()内に追加
function gameLoop() {
    updateUI();
    drawGame();
    drawNextPiece(); // ← 追加
    animationId = requestAnimationFrame(gameLoop);
}
```

## 📊 進捗チェックリスト

- [ ] **🏗️ ディレクトリ名正規化**: block-down → thud-and-tile完了
- [ ] VS Code環境セットアップ完了
- [ ] startGameエラー修正完了
- [ ] 自動落下動作確認完了
- [ ] 次ピース表示API実装
- [ ] 次ピース表示UI実装
- [ ] ゴーストピース実装開始

## 🔧 トラブル時のデバッグコマンド

```bash
# WASM関連エラー
cd /Users/kshimada/Documents/move_folder/src/Rust/thud-and-tile
wasm-pack build --target web --out-dir ../thud-and-tile-web/pkg --dev

# TypeScriptエラー確認
cd /Users/kshimada/Documents/move_folder/src/Rust/thud-and-tile-web
npm run build

# ブラウザコンソールでテスト
gameState.get_fall_speed_ms()  // 期待値: 800
gameState.auto_fall()          // 期待値: true (ゲーム中)
```

## 📞 継続ポイント

**現在完了**: 自動落下システム（Phase 2A最優先項目）
**次の目標**: 次ピース表示 → ゴーストピース → Phase 2A完了

**時間予想**: 次ピース表示 2-3時間、ゴーストピース 2-3時間

---
**この資料使用後**: HANDOVER_PHASE2A.mdの詳細資料も参照してください。