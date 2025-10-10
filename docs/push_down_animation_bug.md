# Push Down Animation – Known Issue (2025-10-10)

## 概要
- **現象**: ライン消去後の Push Down アニメーションで、Solid ラインが下がりきった後も連結セル (`Cell::Connected`) が空中に残存するケースがまれに発生する。
- **発生頻度**: 長時間プレイ中に 1 度のみ確認。通常は問題なく Push Down が完了する。
- **再現状況**: 下から 3 段目・5 段目に `2` 表示の連結セルが残っているスクリーンショット (`buggy-ss.txt`) を記録済み。

## 想定される原因
1. **対象ケース**: 非底面ラインが複数本、間隔を空けて同時に消去されるシナリオ。
2. **処理の流れ**:
   - ライン消去後、`animation::update_animations()` が `result.completed_push_downs` に複数の `gray_line_y` を同時に返す。
   - 呼び出し側では、それらの行を順番に `process_push_down_step()` へ渡し、即座にベクタ `board` に対する `remove(target_y)` / `insert(0, empty row)` を実行する。
3. **問題点**:
   - `remove(target_y)` により行インデックスが変化するため、後続の Push Down が参照する `gray_line_y` / `target_y` が実際の盤面位置とずれてしまう可能性がある。
   - その結果、Push Down で消すべき行が取り除かれず、連結セルが空中に残ることがある。

## 暫定的な解決方針
- **シリアル処理化**: Push Down は常に 1 本ずつ処理し、下側の Solid ラインが着地してから次のラインをアニメーションに乗せる。
  - `result.completed_push_downs` を受け取ったタイミングで降順ソートし、キューに格納。
  - `GameState` に "進行中の Push Down" を表すフィールドを追加し、アニメーション更新ループでキューを 1 本ずつ消化する。
  - `process_push_down_step` が `Completed` を返したら次のラインを取り出す。処理中は新たな Push Down を重ねない。
- **テスト追加**: 非連続な複数ラインを同時に消去するケースを `tests::game_state_tests` などへ追加し、Push Down 完了後に余分な `Cell::Connected` が残らないことを検証する。

## 現在の対応状況
- 今回は CHAIN-BONUS 関連のマージを優先するため、Push Down シーケンスの改修は後日対応とする。
- 本ドキュメントと `HISTORY.md` に既知課題として記録済み。
- 今後修正する際は、シリアル化とテスト追加を合わせて実施すること。
