# WASM API Core Module統合 TDDサイクル計画書

## プロジェクト概要
WASM APIレイヤーを完全にCore Moduleベースに移行し、EraseLineアニメーション機能を含む統一的なゲーム状態管理を実現する。

**⚠ 設計コンセプト適合性レビュー結果：**  
この統合プランは過去のWASMインシデント教訓を活かした`CLI_WASM_INTEGRATION_REDESIGN.md`の新設計コンセプトに**部分的に適合**しているが、以下の重要な原則との乖離があります：

### 🚨 原則乖離箇所と対応
1. **データコピー最優先**: ✅ 部分適合 - Core Moduleの`process_input`戻り値処理で実現予定
2. **責任の明確分離**: ❌ 未適合 - Layer分離設計が統合プランに反映されていない
3. **段階的統合**: ✅ 適合 - 6つのPhase設計で実現
4. **検証可能性**: ✅ 適合 - TDDサイクル設計で実現

## 現状分析
### 完了済み
- ✅ Core Module EraseLineアニメーション実装完了（15/15テスト通過）
- ✅ WasmGameState構造のCore Module基盤への部分移行

## 特定された改修事項
1. **入力処理の非統合**: `handle_input`関数が独自実装でCore Moduleの`process_input`未使用
2. **ToggleEraseLine未対応**: `input_code: 8`（ToggleEraseLine）がWASM APIで未実装
3. **アニメーション状態API不足**: EraseLineアニメーションの状態取得・制御API不在
4. **イベント処理未統合**: Core Moduleイベント（GameModeChanged等）のWASM境界での処理不足
5. **型変換問題**: WASM境界での安全な型変換とエラーハンドリング不統一
6. **chain_bonus増加機能の統合不足**: 既存の`add_chain_bonus` WASM APIはあるが、Core Moduleの自動chain_bonus増加ロジック（ピース配置時の隣接ブロック処理）がWASM側で未適用
7. **🚨 アーキテクチャレイヤー不統合**: 再設計書のLayer分離（共通コアロジック/CLI専用/WASM API）が現在の統合アプローチに反映されていない

## TDDサイクル実行計画

### Phase 1: 統合テストフレームワーク構築
**期間**: 1日  
**目標**: WASM APIとCore Module間の統合テスト基盤確立
**⚠ 設計適合性**: 再設計書Phase 1（共通コアロジック抽出）との整合性要確認

#### Step 1.1: WASM統合テスト環境構築
- WASM bindgenテスト環境セットアップ
- Core Module - WASM API統合テストケース骨格作成
- JavaScript側との通信テスト基盤構築
- **追加必要**: 再設計書Layer分離の適合性検証

#### Step 1.2: 基本統合テストケース作成
- WasmGameState初期化テスト
- Core Module状態同期テスト
- 基本入力処理統合テスト
- **追加必要**: データコピーパターンの借用チェッカー競合回避テスト

### Phase 2: Core Module入力処理統合
**期間**: 1-2日  
**目標**: WASMの`handle_input`をCore Moduleの`process_input`ベースに完全移行
**⚠ 設計適合性**: 再設計書のWASM APIレイヤー（Layer 3）設計原則準拠要確認

#### Step 2.1: 入力処理統合テスト作成
- 全GameInput種別のWASM API経由テスト
- ToggleEraseLine（input_code: 8）専用テスト
- Core Moduleイベント発生確認テスト
- **追加必要**: データコピー最優先パターンの検証

#### Step 2.2: handle_input関数のCore Module統合実装
- `process_input`関数をWASM境界で呼び出す新実装
- input_code → GameInput変換の完全対応
- Core Module戻り値（InputProcessResult）のWASM境界変換
- **重要**: 再設計書の「値渡し」「借用チェッカー安全」原則に準拠

#### Step 2.3: ToggleEraseLine機能実装
- input_code: 8のToggleEraseLineマッピング追加
- enable_erase_line状態のWASM API露出
- EraseLineアニメーション開始条件の統合テスト

#### Step 2.4: chain_bonus自動増加機能統合
- Core Moduleの`lock_current_piece`機能をWASM APIに統合
- ピース配置時の隣接ブロック処理とchain_bonus自動増加の実装
- 既存の`add_chain_bonus` API（手動用）との統合テスト
- **重要**: 共通コアロジック（再設計書Layer 1）からの純粋関数活用

### Phase 3: アニメーション状態API統合
**期間**: 1日  
**目標**: EraseLineアニメーション状態の完全なWASM API露出

#### Step 3.1: アニメーション状態テスト作成
- `has_active_erase_line_animation()` APIテスト
- `get_erase_line_animation_progress()` APIテスト
- アニメーション完了イベント処理テスト

#### Step 3.2: アニメーション状態API実装
- Core Moduleアニメーション状態のWASM境界露出
- JavaScript側で使いやすい形式での状態返却
- アニメーション進行度情報の詳細提供

### Phase 4: イベント処理統合
**期間**: 1日  
**目標**: Core ModuleイベントのWASM境界での統一処理

#### Step 4.1: イベント処理テスト作成
- GameModeChangedイベント処理テスト
- EraseLineAnimationStarted/Completedイベントテスト
- 複数イベント同時発生処理テスト

#### Step 4.2: イベント処理統合実装
- Core ModuleイベントのWASM境界での受信・変換
- JavaScript側への適切なイベント通知メカニズム
- イベントキューの効率的な管理

### Phase 5: 型安全性・エラーハンドリング強化
**期間**: 1日  
**目標**: WASM境界での型安全性とエラーハンドリングの統一

#### Step 5.1: 型安全性テスト作成
- 不正input_code処理テスト
- WASM境界での型変換エラーテスト
- メモリ安全性確認テスト

#### Step 5.2: 型安全性強化実装
- input_code範囲チェック強化
- WASM境界でのResult型活用
- エラー状態の適切なJavaScript側通知

### Phase 6: 総合統合テスト・性能最適化
**期間**: 1日  
**目標**: 完全統合テストとパフォーマンス検証

#### Step 6.1: 総合統合テスト作成
- EraseLineアニメーション完全フローのWASM APIテスト
- CLI版とWASM版の動作等価性確認テスト
- 大量入力処理の性能テスト

#### Step 6.2: 最終統合・最適化
- 全機能統合テスト実行・修正
- WASM境界パフォーマンス最適化
- 完全なCore Module - WASM API統合達成

## 成功指標
1. **機能完全性**: CLI版と同等のEraseLineアニメーション機能をWASM APIで実現
2. **テストカバレッジ**: 全WASM API機能の統合テスト100%通過
3. **型安全性**: WASM境界での型変換エラー0件
4. **性能**: Core Module統合によるオーバーヘッド最小化
5. **保守性**: 単一のCore Moduleでの統一状態管理

## リスク管理
- **WASM制約**: wasm-bindgenの制約による実装制限の事前調査
- **型変換コスト**: WASM境界での型変換コスト最小化戦略
- **デバッグ困難性**: WASM環境でのデバッグツール・ログ戦略確立
- **テスト環境**: ブラウザ環境でのテスト実行の安定性確保
- **chain_bonus同期**: Core Moduleとの自動増加処理の確実な同期確保
- **🚨 借用チェッカー競合**: 再設計書で特定された過去インシデントの再発防止（データコピー最優先の徹底）
- **アーキテクチャ不整合**: 現統合プランと再設計書Layer分離設計の整合性確保

## 最終目標
Core ModuleベースのWASM APIにより、EraseLineアニメーション機能を含む完全な統一ゲーム状態管理を実現し、CLI版とWASM版の機能等価性を達成する。

**⚠ 重要な設計適合性課題:**  
現在の統合プランは再設計書の核心原則に部分的に適合していますが、Layer分離アーキテクチャ（共通コアロジック/CLI専用/WASM API）への準拠が不足しています。実装前に以下の対応が必要です：

1. **Layer 1（共通コアロジック）の確立**: Core Moduleが既にこの役割を担っているかの検証
2. **Layer 3（WASM APIレイヤー）の設計見直し**: データコピー最優先原則の徹底
3. **借用チェッカー競合回避**: 過去のWASMインシデント再発防止の具体的実装パターン確立