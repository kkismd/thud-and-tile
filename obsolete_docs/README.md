# Obsolete Documents Archive

このフォルダには、Phase 2A完了により現状に合わなくなった過去の文書が保管されています。

## 移動理由と日付

### 2025-10-07 - WASM統合再設計による文書整理

#### Phase 4完了による旧統合文書バックアップ
- `WASM_CORE_INTEGRATION_PLAN_OLD.md` - 旧統合計画（2層設計） → 新3層設計版で置き換え
- `WASM_CORE_INTEGRATION_TECHNICAL_OLD.md` - 旧技術仕様 → データコピー最優先版で置き換え

#### 開発完了により不要となった個別ファイル
- `z_mino_test.rs` - 個別テストファイル → 統合テストスイートに移行済み
- `run_tests.sh` - 古いテストスクリプト → `cargo test`で置き換え
- `current_status.json` - 軽量ステータス管理 → 新文書構造で不要

#### 再設計により置き換えられた統合文書
- `GRADUAL_MIGRATION_PLAN.md` - 段階的移行実装計画 → `WASM_REDESIGN_PHASE_ANALYSIS.md`のPhase別アプローチで置き換え
- `CLI_WASM_UNIFIED_ARCHITECTURE.md` - 統合アーキテクチャ戦略 → `CLI_WASM_INTEGRATION_REDESIGN.md`の3層分離設計で置き換え  
- `WASM_API_LAYER_DESIGN.md` - WASM APIレイヤー設計 → データコピー最優先原則による新設計で置き換え予定
- `UNIFIED_ARCHITECTURE_RETROSPECTIVE.md` - アーキテクチャ振り返り → Phase別再設計プロセスで置き換え

**移動理由**: Phase 4完了による文書整理と、CLI_WASM_INTEGRATION_REDESIGN.mdの設計原則（データコピー最優先、Layer分離、借用チェッカー安全性）に準拠した新しいアプローチによる全面見直しのため

### 2024-10-03 - Phase 2A完了による大規模アーカイブ

#### 計画・分析文書（実装完了により不要）
- `srs_tdd_plan.md` - SRS実装TDD計画書 → SRS実装完了により不要
- `rotation_analysis.md` - 回転システム分析レポート → 回転実装完了により不要
- `physical_rotation_analysis.md` - 物理回転分析 → 物理回転実装完了により不要
- `z_mino_analysis.md` - Z-mino形状分析 → Z-mino実装完了により不要
- `t_mino_mapping.md` - T-mino色マッピング計算 → T-mino実装完了により不要
- `shape_rotation.txt` - 回転実装メモ → SRS実装完了により不要

#### フェーズ文書（Phase 2A完了により不要）
- `QUICKSTART_PHASE2A.md` - Phase 2Aクイックスタートガイド → Phase 2A完了により不要
- `HANDOVER_PHASE2A.md` - Phase 2A引き継ぎ資料 → Phase 2A完了により不要
- `animation_integration_report.md` - アニメーション統合レポート → 統合完了により不要

#### 予測・提案文書（実装完了により不要）
- `mobile_web_migration_proposal.md` - モバイルWeb移植提案書 → Phase 2Aで大部分実現により不要

#### レガシー管理文書（新構造への移行により不要）
- `session_state.json` - 旧式状況管理ファイル → 3ファイル構造に置き換えにより不要

## 新しい文書構造（2025年10月7日 Phase 4完了時点）

### 🏗️ WASM統合設計（Phase 2準備）
**現在有効な設計文書**：
- `CLI_WASM_INTEGRATION_REDESIGN.md` - 3層分離アーキテクチャ基本方針（設計基準文書）
- `WASM_REDESIGN_PHASE_ANALYSIS.md` - 4段階設計プロセス管理文書
- `WASM_CORE_INTEGRATION_PLAN.md` - 4フェーズ実装ロードマップ（Phase 4で全面改訂）
- `WASM_CORE_INTEGRATION_TECHNICAL.md` - WASM境界安全設計詳細（Phase 4で全面改訂）

### 📋 Phase別設計文書
- `PHASE1_CORE_MODULE_COMPATIBILITY.md` - Core Module適合性検証結果（95%適合）
- `PHASE2_LAYER_SEPARATION_DESIGN.md` - 3層分離アーキテクチャ設計
- `PHASE2_MIGRATION_RISK_ANALYSIS.md` - CLI機能移行時のリスク対策
- `PHASE2_PLAN_REVISION.md` - 段階的移行戦略とロールバック計画
- `PHASE3_WASM_BOUNDARY_REDESIGN.md` - WASM境界安全設計（データコピー最優先）
- `PHASE4_INTEGRATION_PLAN_REBUILT.md` - 統合計画再構築完了報告

### 📚 プロジェクト管理文書
- `README.md` - プロジェクト概要と関連資料一覧
- `ROADMAP.md` - 今後の開発計画
- `HISTORY.md` - 開発履歴（追記専用）
- `game_spec.md` - ゲーム仕様（継続有効）
- `FINAL_COMPLETION_REPORT.md` - Phase 1完了報告

## アーカイブポリシー

これらの文書は：
- ✅ 履歴として保持（git historyで追跡可能）
- ✅ 将来の参考資料として利用可能
- ✅ obsolete_docsフォルダで整理保管
- ❌ 現在の開発では参照・更新しない