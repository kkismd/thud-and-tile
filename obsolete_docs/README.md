# Obsolete Documents Archive

このフォルダには、Phase 2A完了により現状に合わなくなった過去の文書が保管されています。

## 移動理由と日付

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

## 新しい文書構造（Phase 2A完了後）

現在有効な文書：
- `current_status.json` - 現在状況の軽量サマリー
- `ROADMAP.md` - 優先度付き次ステップと詳細計画
- `HISTORY.md` - 完了タスクの履歴（追記専用）
- `game_spec.md` - ゲーム仕様（継続有効）

## アーカイブポリシー

これらの文書は：
- ✅ 履歴として保持（git historyで追跡可能）
- ✅ 将来の参考資料として利用可能
- ✅ obsolete_docsフォルダで整理保管
- ❌ 現在の開発では参照・更新しない