# Task History Log

## 2025-10-08 - Phase 2D再設計準備とロールバック完了

### ✅ WIPコードの安全退避と安定状態への復元
- 新ブランチ `phase2d-wip` を作成し、ターミナルイベントループ試作と関連ファイルをコミット
- 安定ライン `feature/wasm-integration-from-complete` をタグ `phase2c-completed` にハードリセットし、Phase 2C完了時点へ復元
- 作業ツリーをクリーンな状態に保ち、再設計着手の準備を整備

### ✅ CLIレンダラー再設計プランの確定
- レンダリング入出力契約、想定エッジケース、エラーモードを整理
- FrameBuilder/ターミナルドライバ/差分計算の三層構造で再構築する方針を決定
- スナップショットテスト導入とプレビュー用検証バイナリ追加を品質ゲートとして設定

### ✅ ドキュメント更新
- `PROJECT_STATUS.md` にPhase 2C完了およびPhase 2D再計画の状況を反映
- `IMPLEMENTATION_ROADMAP.md` をレンダラー再設計中心のステップへ改訂
- Phase履歴とロールバック判断を本履歴ファイルに追記

### 🔜 次ステップ
- FrameBuilderの骨格実装および最初の決定論的描画テストの追加
- ターミナルドライバのAPI設計とプロトタイプ実装
- スナップショットテスト基盤導入後、Phase 2D正式完了に向けた統合作業

## 2025-10-04 - メカニクス改善計画策定完了

### ✅ メカニクス改善計画の完成
- **スコアシステム改修計画:** ColorScores → total_score統一設計
  - 色別表示廃止、合計値のみ表示
  - 直接合計値加算方式への変更
  - CustomScoreSystem構造体の変更仕様確定
- **CHAIN-BONUSシステム設計:** 新機能の詳細仕様策定
  - ColorMaxChainsにchain_bonusメンバ追加
  - MAX-CHAIN更新時の加算ロジック明確化
  - Solidライン相殺での消費システム設計
- **Solidライン相殺機能（EraseLine）:** アニメーション仕様完成
  - LineBlink → PushDown → EraseLineのシーケンス設計
  - 120ミリ秒間隔での実装方針確定
  - ゲームバランス考察とプレイループ分析
- **実装計画策定:** TDDサイクルでの段階的実装ステップ明確化
  - 5段階の実装順序確定
  - 既存テスト修正（約15個）の影響範囲特定
  - ROADMAPへの最優先事項反映

### ✅ ドキュメント整備
- **mechanics_improve_plan.md:** 実装に向けた完全な技術仕様書完成
- **ROADMAP.md:** Priority 0として最優先事項設定
- **game_spec.md:** Solid=灰色表記の統一化

## 2024-10-04 - コード共通化・品質改善フェーズ完了

### ✅ Issue #4, #5 バグ修正完了
- **Issue #4:** ZミノのSHAPE回転時色配置ずれ → **解決済み**
  - 物理回転順序に準拠した座標配列修正
  - CLI・Web版両方で統一された挙動を実現
- **Issue #5:** ライン消去後Connected数未更新 → **解決済み**
  - `update_animation()`でのconnected数リアルタイム更新
  - アニメーション完了時の確実な再計算

### ✅ コード共通化による重複削除
- **SHAPES配列統一:** tetromino.rs → lib.rs共通化 (50+行削除)
  - `get_shape_coordinates()`公開関数で1箇所管理
  - CLI・Web版で同じ座標データ使用、修正は1箇所で完結
- **TetrominoShape定義統一:** Web版重複定義削除、CLI版をインポート使用
- **7-bag システム統一:** WebTetrominoBag削除、TetrominoBag共通使用
- **隣接制約チェック共通化:** `validate_adjacency_constraints()`でロジック統一

### ✅ API整理・品質向上
- **紛らわしいAPI削除:** `SimpleTetromino::new_random()`（完全ランダム）を削除
- **Web版テスト強化:** 3新規テスト追加
  - `test_web_7_bag_system()`: Web版7-bagシステム検証
  - `test_web_adjacency_constraints()`: 隣接制約検証
  - `test_web_cli_tetromino_compatibility()`: CLI・Web版互換性確認
- **品質保証:** 95/95テスト成功（lib.rs: 30, main.rs: 65）

### ✅ 技術的負債解消
- **DRY原則達成:** 重複コードの大幅削減
- **Single Source of Truth:** 重要なデータ構造の一元管理
- **保守性向上:** 将来の修正・機能追加コストの削減
- **テスト強化:** Web版品質保証体制の確立

**主要コミット:**
- `dc8c243`: "refactor: 共通化によりSHAPES配列の重複コードを削除"
  - 2 files changed, 17 insertions(+), 67 deletions(-)
  - CLI・Web版統一アーキテクチャの基盤確立

---

## 2024-10-04 - 統一アーキテクチャ第1回実装レトロスペクティブ

### ❌ 統一アーキテクチャ第1回実装 - 大幅な機能回帰
- **ブランチ:** feature/unified-architecture
- **実装期間:** 2024-10-04
- **結果:** 実装中断・機能回帰により廃棄決定
- **教訓:** レトロスペクティブ分析実施、テスト駆動開発の重要性を確認

**実装した要素:**
- UnifiedGameEngine trait と GameStateAccess interface
- CLI/Web分離アーキテクチャ
- 統一レンダリングシステム (draw_unified)
- クロスプラットフォーム状態アクセス

**深刻な問題:**
- CLI版: 落下ピースが1ブロックのみ表示、画面フリッカー
- 両版: Push Down animations, Line Blink効果, Connected Cells, Ghost Pieces完全消失
- Web版: 2000+行の機能が214行に激減、差分レンダリング喪失
- スコア: 詳細なカスタムスコアリングシステム簡素化

### ✅ レトロスペクティブ分析完了
- **文書:** UNIFIED_ARCHITECTURE_RETROSPECTIVE.md
- **フレームワーク:** Keep/Problem/Try analysis
- **結論:** テスト駆動開発による段階的アプローチが必要

**Keep (保持すべき要素):**
- UnifiedGameEngine trait設計思想
- GameStateAccess抽象化概念  
- CLI/Web分離アーキテクチャ
- 統一レンダリングの目標

**Problem (問題点):**
- Big Bang アプローチによる過度な変更
- 既存機能の軽視と保護不足
- テスト不足による品質保証欠如
- 過度な抽象化による複雑性増加

**Try (次回改善策):**
- Test-Driven Development (TDD) 適用
- Red-Green-Refactor サイクル厳守
- 段階的移行による機能保持
- 既存コードの段階的リファクタリング

### ✅ 実装履歴の記録保存
- **最終コミット:** 17dfb4d "統一アーキテクチャ実装試行の作業途中状態を記録"
- **レトロスペクティブ:** fb833a2 "レトロスペクティブにテスト駆動開発アプローチを追加"
- **mainブランチ統合:** b358386 "レトロスペクティブ文書をmainブランチに追加"

---

## 2024-10-03 - Phase 2A SRS Unification Complete

### ✅ SRS True Rotation Implementation for O-mino
- **Commit:** f99b3a4
- **Files:** src/tetromino.rs, src/lib.rs, session_state.json
- **Achievement:** SRS標準のTrue Rotation概念を正しく実装し、O-minoのwobble効果を達成
- **Technical Impact:** CLI版とWeb版で同じ「ブロック回転+色固定」方式に完全統一

**Key Changes:**
- CLI版tetromino.rs: O-mino SHAPES定義をSRS True Rotation座標に変更
- Web版lib.rs: SimpleTetromino get_blocks_at_rotation()でO-mino wobble効果実装
- SRS準拠の「ブロック回転+色固定」アプローチに統一
- 特殊な色回転処理を削除し、物理回転順序による自然な色追従実現

### ✅ Code Cleanup: get_rotated_color_mapping最適化
- **Commit:** d383ad6  
- **Files:** src/lib.rs
- **Achievement:** 不要なメソッドと引数を削除してコードの簡潔性と保守性を向上

**Key Changes:**
- get_rotated_color_mapping関数から不要な_clockwise引数を削除
- SRS True Rotation実装により不要となったrotate_colors関数を完全削除
- rotate_current_piece内のrotate_colors呼び出しも削除
- 実装をより簡潔にしてSRS標準準拠の単純な色マッピングに統一

### ✅ Node.js互換WASMテスト環境実装
- **Files:** Cargo.toml, src/lib.rs, src/random.rs, src/scheduler.rs, wasm-pack.toml
- **Achievement:** ブラウザ限定API依存によるNode.js実行エラーを解決し、クロスプラットフォーム自動テストを実現

**Key Additions:**
- nodejs-test feature flag追加
- js_math_random() Node.js互換ポリフィル
- js_date_now() SystemTime実装
- console_log! println!フォールバック
- web_sys::window() 条件付き無効化
- wasm-pack.toml テスト設定

### ✅ Animation System Complete Migration
- **Commit:** 6f3dd49
- **Achievement:** CLI版の手動アニメーション処理を共通モジュールanimation::update_animations()に統合。WASM版との統一達成

**Implementation Details:**
- Replaced functions: handle_line_blink_animation, handle_push_down_animation
- Unified with: animation::update_animations()
- Resolved conflicts: render.rs内の重複Animation enum削除
- Architectural achievement: CLI版とWASM版でアニメーション処理完全統一、コード重複削減

### ✅ Complete Line Clear System Migration
- **Achievement:** CLI's connected block system, isolated block removal, and advanced scoring migrated to Web version

### ✅ Custom Score System Complete Integration  
- **Achievement:** 色別スコア計算とMAX-CHAIN統合完了。CLI版とWASM版でlock_piece()でのスコア計算タイミング統一実現
- **Details:** CLI版main.rsとWASM版lib.rsでcalculate_line_clear_score()共通関数使用、アニメーション時ではなくlock_piece()時にスコア計算実行

### ✅ Dynamic Board Height System
- **Achievement:** Full CLI-equivalent Dynamic Board Height System implementation complete with JavaScript APIs

---

## Technical Notes Archive

### SRS True Rotation Implementation
**Status:** Completed  
**Key Concept:** SRS True Rotation - 回転中心がミノ中心と一致し、回転時に座標が変化

**O-mino Implementation:**
- Approach: ブロック回転+色固定
- Coordinates change: State 0→1→2→3で座標が物理的に回転
- Color handling: 色は各ブロックに固定、回転に伴って自然に追従
- Wobble effect: SRS標準の「O tetromino does not kick」動作を正確に実装

**CLI/Web Unification:**
- tetromino.rs: SHAPES定義を回転する座標系に変更
- lib.rs: get_blocks_at_rotation()でstate-specific座標実装
- Color system: 特殊な色回転処理を削除し、直接マッピングに統一

**Benefits:**
- SRS標準完全準拠
- 実装の一貫性向上
- 特殊ケース処理削減
- 保守性向上

### Node.js WASM Testing Implementation
**Status:** Completed  
**Problem Solved:** ブラウザ限定API (console.log, Date.now, Math.random, web_sys::window) によるNode.js実行エラー
**Solution Approach:** 条件付きコンパイルとポリフィル実装

**Key Features:**
- Feature flag separation: wasm-test (browser) vs nodejs-test (Node.js)
- js_math_random() deterministic PRNG for Node.js
- js_date_now() SystemTime-based implementation
- console_log! println! fallback
- web_sys::window() conditional disabling in tests

**Build Verification:** ✅ cargo build --target wasm32-unknown-unknown --lib --features wasm-test

**Test Commands:**
- wasm-pack test --node --features nodejs-test -- --lib
- wasm-pack test --features wasm-test (browser)

**Impact:** クロスプラットフォーム自動テスト基盤の確立、CI/CD準備完了

### Dynamic Board Height Implementation  
**Status:** Completed

**Key Changes:**
- Added current_board_height field to WasmGameState
- Updated is_valid_position for dynamic boundary checking
- Modified clear_lines and get_connected_cells_info to use current_board_height
- Implemented finalize_gray_line with CLI-equivalent height reduction
- Added JavaScript APIs: get_current_board_height() and set_current_board_height()

**CLI Reusability:** Successfully reused CLI's saturating_sub() logic and boundary checking patterns