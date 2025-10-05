# Thud&Tile メカニクス改善計画

## スコアシステムの改修計画

### 既存の仕組みの変更点
* SCOREを色別に表示するのをやめて、合計値ひとつにする
  * ColorScoresを単一の total_score: u32 に変更
* 色別のMAX-CHAINだけを表示して最大値は表示しない

* スコア計算方式の変更
  * 従来：色別に加算後、total()で合計計算
  * 改善後：直接合計値(total_score)に加算

* CustomScoreSystem構造体の変更
  * scores: ColorScores → total_score: u32
  * max_chains: ColorMaxChains (chain_bonusメンバ追加)

### 追加要素
* CHAIN-BONUSという新しい値を作成する
  * ColorMaxChains構造体に新しく chain_bonus というメンバを追加する
* CHAIN−BONUSはピースが着地したタイミングで、色別のMAX-CHAIN値が増えたら、その増えた数が加算される
  * 例：CYAN: 2→3, MAGENTA: 4→4, YELLOW: 5→6の場合、CHAIN-BONUSには1+0+1=2が加算される
* CHAIN-BONUSは後述するSolidラインの相殺で1ラインにつき1ポイント消費される
  * 着地→連結数字計算→MAX-CHAIN更新→CHAIN-BONUS加算の順序で実行

### 変更後のUIイメージ

```
SCORE:    1120

MAX-CHAIN:
  CYAN:    2
  MAGENTA: 4
  YELLOW:  5

CHAIN-BONUS: 11
```

## Solidラインシステムの改修計画

フィールドの高さを狭めるSolidラインは、ライン消去のたびに揃ったラインの数だけ積み上がるが、積み上がった後に、CHAIN-BONUSが1以上の場合はCHAIN-BONUSの数値を消費してSolidラインの相殺（EraseLine）が起こる。
CHAIN-BONUSが0の場合は相殺が起こらない。

相殺のアニメーションは
- CHAIN-BONUSが1減らされる
- Solidラインが1列消し込まれる
- CHAIN-BONUSが0になるか、Solidラインがすべて消えるまで、これが繰り返される

アニメーションシーケンスとして、LineBlink, PushDownのあとにEraseLineというシーケンスを新しく追加して対応する。

- LineBlink -- ラインが揃った後に点滅する演出
- PushDown -- Solidラインが降下していく演出
- EraseLine -- Solidラインが相殺される演出

PushDownは「残っているブロックを1段ずつ消しながら一番下のSolidラインの上に下がっていく」という演出を実装するシーケンス

PushDown完了後にEraseLineを開始するという順序になる

アニメーションの間隔はまず120ミリ秒で実装し、動いているのを見て調整したい

## ゲームバランスに関する考察

現状のメカニクスでは、1ライン揃うごとに、ボトムでないかぎりSolidラインが1段ずつ増えていくため、着実にフィールドが狭くなっていくく。

MAX-CHAINを更新するごとにSolidラインを相殺できれば、ゲームを継続させるというメリットが生まれるため、色を揃える行為が単にスコアを稼ぐこと以上の価値とリターンにつながる。

難易度設定のポイントとしては、MAX-CHAINの合計値=相殺できるライン数なので、ラインを揃えるたびに相殺がおきるわけではないということ。
MAX-CHAINを伸ばすことには限界があるため、必ず何処かで頭打ちとなり、相殺はそこでストップする。そのあとはフィールドが迫ってきてゲームオーバーとなる。

## 既存テストへの影響

ColorScoresを廃止すると、既存のテストコード（約15個）の修正が必要になります。

## TDD実装について

**詳細な実装計画は別ドキュメントに分離されています:**
[mechanics_tdd_implementation_plan.md](./mechanics_tdd_implementation_plan.md)

上記ドキュメントには以下の詳細が含まれています：
- Phase 1-9 の全TDD Cycleの実装手順
- RED-GREEN-REFACTOR パターンの具体的なテストコード
- 各フェーズの依存関係と実装順序
- 開発完了基準とエラー対応方針

### 実装フェーズの概要

#### Phase 1-4: スコアシステム基盤構築
- **Phase 1**: ColorMaxChainsにchain_bonusメンバ追加
- **Phase 2**: CustomScoreSystem構造変更（段階的移行）
- **Phase 3**: スコア計算ロジックの変更
- **Phase 4A/4B**: CHAIN-BONUS更新ロジック実装とスコア加算処理統合

#### Phase 5-6: アニメーションシステム統合
- **Phase 5**: EraseLineアニメーション実装
- **Phase 6**: 新スコア計算システム統合

#### Phase 7-9: 完成とUI更新
- **Phase 7**: 旧システム削除とテスト移行（一括実行）
- **Phase 8**: UI/表示系更新とCHAIN-BONUS連携 ✅ (Phase 8-1, 8-2完了)
- **Phase 9**: EraseLineアニメーションシステム完成（厳密な仕様実装）

### 実装状況
- **完了済み**: Phase 1-6, Phase 8-1/8-2
- **現在作業中**: Phase 9-4 統合テストとエッジケース
- **保留中**: Phase 7 (大量のレガシー依存のため), Phase 8-3 (WebAssembly複雑性)

## 開発完了基準

1. **機能完備性**：
   - ✅ 新スコア計算システム（Phase 3-4）
   - ⏳ EraseLineアニメーション（Phase 5）
   - ⏳ 統合システム（Phase 6-7）
   - ⏳ 高度機能（Phase 8-9）

2. **品質基準**：
   - すべてのTDDサイクル完了
   - テストカバレッジ95%以上維持
   - パフォーマンス要件充足
   - ドキュメント整備完了

3. **デプロイ準備**：
   - 旧システム完全削除（Phase 7）
   - 設定ファイル更新
   - リリースノート作成

### 各Cycleでの確認事項
1. **cargo check**: コンパイルエラーなし
2. **cargo test**: 全テスト通過（95/95維持）
3. **cargo clippy**: 警告なし
4. **cargo fmt**: フォーマット適用
5. **git add && git commit**: 各Cycle完了時にコミット
6. **in japanese**: コミットメッセージは日本語で書く

### エラー発生時の対応
- **RED段階**: コンパイルエラーは期待される（新機能追加時）
- **GREEN段階**: テスト通過最優先、最小実装でOK
- **REFACTOR段階**: 機能変更禁止、品質向上のみ
- **想定外の失敗**: 前Cycleに戻って原因調査
- **テスト数減少**: 即座に原因特定と修復

### 実装完了の確認基準
- [ ] 全95テストが通過
- [ ] 新機能のテストが追加済み
- [ ] ColorScoresが完全削除済み
- [ ] CHAIN-BONUSが正常動作
- [ ] EraseLineアニメーションが実装済み
- [ ] CLI版での動作確認完了