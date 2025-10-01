## テトリスゲーム (Rust CLI) - 開発仕様書

### 1. プロジェクト概要

Rust言語と`crossterm`ライブラリを使用して開発された、ターミナル上で動作するテトリスゲーム。標準的なテトリスの要素に加え、独自のブロック消去ルール、スコア計算、および重力システムを特徴とする。

### 2. コアゲームメカニクス

*   **盤面サイズ:** 幅10マス x 高さ20マス。
*   **テトリミノ:** 標準的な7種類のテトリミノ（I, O, T, L, J, S, Z）を使用。
*   **基本操作:**
    *   左右移動 (`←`, `→` キー)
    *   時計回り回転 (`↓` キー)
    *   半時計回り回転 (`↑` キー)
    *   ソフトドロップ (`Space` キー)
    *   ハードドロップ (`Shift` + `↓` キー)
*   **ゲームオーバー:** ブロックが盤面上部に積み上がるとゲームオーバー。
*   **ゲームサイクル** タイトル画面→ゲームプレイ→ゲームオーバー→タイトル画面。

### 3. 視覚表現とUI

*   **ブロック表現:** 各ブロックは `[]` で表現され、前景色で色付けされる。
*   **テトリミノの色:**
    *   使用色はシアン、マゼンタ、イエローの3色のみ。
    *   各テトリミノは4つのブロックで構成され、出現時にこれら3色からランダムに割り当てられる（1つのテトリミノ内で4つのブロックは異なる色を持ち、かつ隣接するブロックは異なる色を持つ）。
    *   ミノが回転しても、ブロックと色の対応関係は変わらない。
*   **壁の色:** 盤面の枠線は灰色で表示され、ブロックと区別しやすい。
*   **落下位置予測 (ゴースト):**
    *   現在落下中のテトリミノの最終着地点が `::` の文字で表示される。
    *   ゴーストの色は、その位置に着地するブロックの色と同じ。
*   **ライン消去アニメーション:**
    *   アニメーション中のブロックは、元の色を反転させた色で表示される。
    *   アニメーション速度は1ステップあたり120ミリ秒。
*   **UI表示:**
    *   得点が盤面右側に表示される。
    *   その下にNEXTミノが表示される

### 4. カスタムルール

*   **ミノの着地時の盤面の変化**
    1. ブロックが着地するたびに固定ブロックがスキャンされ、上下左右に同じ色が隣接している場合はブロックが数字に変化する。
    2. 数字は、そのブロックが同色のブロックと何個連結しているかを表す数
    3. 数字はバックグラウンドがそのブロックの色で文字色は黒

*   **得点システム**
    1. 得点はブロック消去で加算されるSCOREとMAX-CHAINがある。
    2. SCOREもMAX-CHAINもどちらも色別に数値を管理する
        1. SCORE(CYAN), SCORE(MAGENTA), SCORE(YELLOW)
        2. MAX-CHAIN(CYAN), MAX-CHAIN(MAGENTA), MAX-CHAIN(YELLOW)
    3. SCOREはブロックの消去によって加算される整数値で、ライン消去のたびに加算される
    4. MAX-CHAINは同色ブロックの連結数のゲームプレイ中での最大値で、ミノが着地するたびにに更新される
    5. 画面上での表示は下記のようになる。SCOREは合計値、MAX-CHAINは最大値がまとめとして使われる。
```
SCORE:    1120
  CYAN:    200
  MAGENTA: 420
  YELLOW:  500

MAX-CHAIN: 5
  CYAN:    2
  MAGENTA: 4
  YELLOW:  5
```

*   **ライン消去後の演出と得点計算:** ラインが揃ったとき、以下のような演出が行われる
    1. 揃ったすべてのラインのブロックが点滅する
    2. 揃ったラインが最下段でない場合
        1. 揃ったラインよりも下にあるブロックを調べる
        2. 上下左右に同じ色がない（つまり数字に変化していない）ブロックが消える（無得点）
    3. 揃ったすべてのラインのブロックが灰色になる
    4. 灰色になったラインは、互いの間隔を保ったまま、残っているブロックを1段ずつ消しながら一番下のSolidラインの上に下がっていくアニメーションを行う
    5. 下がっていくラインはSolidラインに着地すると停止する。複数のラインが間を開けて下がっていくとき、下のラインが停止しても上のラインは下がり続けて間隔が詰まり、最後には下がりきったラインがすべて隣接する。
    6. 消される数字ブロックごとにスコアが加算される。スコアはブロックの数字×10pt
    7. 灰色のラインが一番したの段に達したらSolidのままで残り、フィールドの高さがそのラインの数だけ減る
    8. 次のラインが落下するときは底辺のSolidラインの上に積み重なり、フィールドがまたSolidラインの数だけ減る
    9. フィールドがどんどん浅くなり、フィールドがすべて埋まったらゲームオーバー


### 5. 技術的詳細と実装ノート

*   **言語:** Rust
*   **ターミナル操作ライブラリ:** `crossterm`
*   **入力ハンドリング:**
    *   ノンブロッキング入力。
    *   先行入力を受け付けて移動・回転のタイミングで反映する。
*   **描画:**
    *   `prev_state`と`state`を比較し、差分のみを描画する方式（ちらつき防止）。
    *   アニメーション終了後の画面クリーンアップを明示的に行う。
*   **テスト:**
    *   `#[cfg(test)]`モジュール内にユニットテストを実装。

### 6. 開発の方針

*    開発時の対話は日本語で行う
*    TDDのred-green-refactorのサイクルで開発する
*    cargo testでテストを実施する
*    cargo clippyとcargo fmtでコードの品質を担保する
*    TDDの1サイクルが終わるたびにcargo check, cargo test, cargo clipply, cargo fmtを実行してからgit add, commitする
*    non tracking fileをgit addする場合は理由を説明する
*    コミット前に`game_spec.md`のTDD計画を更新して、完了したタスクにチェックを入れる
*    改行を含むコミットメッセージはコマンドラインで失敗しやすいので一時ファイル経由で登録する
*    コミットメッセージは日本語で記載する
*    ソースコードを修正したときだけcargoコマンドで検査を行う
*    TDDのRefactorではRed/Greenで追加したテストを削除しない
*    コードを変更しようとする場合は必ず内容を説明する

---

### バグリスト

*    [ ]

---

### 改善点リスト

*   得点システムが現在ScoreとLinesの2つの数値を管理表示しているが、カスタムルールのセクションに書いた得点システムと表示を実装したい

---

### TDD計画

#### 得点システムのカスタマイズ

**目標:** カスタムルールに記載された得点システム（SCOREとMAX-CHAIN、色別管理）を実装し、UIに表示する。

**ステップ 1: `GameState` に新しいスコア管理フィールドを追加する (Red)**

1.  **Red:** `GameState` に `score_by_color: HashMap<Color, u32>` と `max_chain_by_color: HashMap<Color, u32>` を追加する。
    *   `GameState::new()` でこれらのマップを初期化する。
    *   `GameState` の `score` と `lines_cleared` フィールドは一時的にそのままにしておく。
2.  **Test:** `GameState::new()` が呼ばれたときに、`score_by_color` と `max_chain_by_color` が空の状態で初期化されることを確認するテストを追加する。

**ステップ 2: `board_logic::handle_scoring` で色別スコアを加算する (Red/Green)**

1.  **Red:** `board_logic::handle_scoring` が `state.blocks_to_score` に基づいて `state.score_by_color` を更新し、`state.score` を合計値として計算することを検証するテストを追加する。このテストは、まだ実装がないため失敗するはずである。
2.  **Green:** `board_logic::handle_scoring` を修正し、`state.blocks_to_score` の各ブロックの `component_size * 10` を、そのブロックの色に対応する `state.score_by_color` に加算するように変更する。また、`state.score` (合計スコア) も、`score_by_color` の合計値として計算するように変更する。これにより、Redで書いたテストがパスすることを確認する。

**ステップ 3: `MAX-CHAIN` の計算と更新を実装する (Red/Green)**

ステップ 3.1: `GameState::update_connected_block_counts` 内で `max_chain_by_color` を更新するロジックを追加する (Red/Green)

1. **Red:** `GameState::update_connected_block_counts` 関数が `board_logic::count_connected_blocks` から返される各ブロックの連結数 (`component_size`) を利用して、対応する色の `max_chain_by_color` を更新することを検証するテストを追加する。このテストは、まだ実装がないため失敗するはずである。
2. **Green:** `GameState::update_connected_block_counts` 関数内で、`board_logic::count_connected_blocks` から返される各ブロックの連結数 (`component_size`) を利用して、対応する色の `max_chain_by_color` を更新するロジックを追加する。
    * 現在の `max_chain_by_color` の値と `component_size` を比較し、大きい方を採用する。これにより、Redで書いたテストがパスすることを確認する。

**ステップ 4: UI表示を更新する (Red/Green/Refactor)**

1.  **Red:** `src/render.rs` の `draw` 関数が、新しいカスタムルールに沿ったスコア表示（`SCORE` と `MAX-CHAIN` の色別表示）を正しくレンダリングすることを検証するテストを追加する。このテストは、まだ実装がないため失敗するはずである。
2.  **Green:** `src/render.rs` の `draw` 関数を修正し、現在の `Score` と `Lines` の表示を、新しいカスタムルールに沿った表示に置き換える。
    *   `SCORE: 合計値`
    *   `  CYAN: 値`
    *   `  MAGENTA: 値`
    *   `  YELLOW: 値`
    *   `MAX-CHAIN: 最大値` (全色の `max_chain_by_color` の最大値)
    *   `  CYAN: 値`
    *   `  MAGENTA: 値`
    *   `  YELLOW: 値`
    これにより、Redで書いたテストがパスすることを確認する。
3.  **Refactor:** `GameState` の `score` と `lines_cleared` フィールドは、この段階で削除または非表示にする。このRefactorによって既存のテストが壊れていないことを確認する。

ステップ 5.1: `GameState` に `solid_lines_count` フィールドを追加し、`current_board_height` 
の計算を調整する (Refactor)

1. Refactor: GameState に solid_lines_count: usize フィールドを追加し、GameState::new() 
   で初期化する。
2. Refactor: current_board_height の計算を BOARD_HEIGHT - solid_lines_count に変更する。
3. Test: このRefactorによって、`GameState::new()` の既存のテストや、`current_board_height` に依存する既存のテストが引き続きパスすることを確認する。

ステップ 5.2.1: `handle_push_down_animation` で灰色のラインを `Cell::Solid` に変換し、`solid_lines_count` を更新する (Refactor)

1.  **Refactor:** `handle_push_down_animation` 関数内で、灰色のラインが一番下の段に達したときに、そのラインを `Cell::Solid` で埋めるように変更する。
2.  **Refactor:** `solid_lines_count` をインクリメントする。
3.  **Test:** このRefactorによって、既存のテストがすべてパスすることを確認する。特に、灰色のラインが `Cell::Solid` に変換され、`solid_lines_count` が正しくインクリメントされるロジックが既存の動作を壊していないことを確認する。

ステップ 5.2.2: `Board` 構造体または関連する描画ロジックで、`current_board_height` に基づいてボードの表示範囲を調整する (Refactor)

1.  **Refactor:** `Board` 構造体または `render.rs` 内の描画ロジックを修正し、`current_board_height` を利用して、実際に描画されるボードの範囲を調整する。これにより、`solid_lines_count` によって隠された下部の行が描画されないようにする。
2.  **Test:** このRefactorによって、既存のテストがすべてパスすることを確認する。特に、`current_board_height` に基づくボードの表示範囲調整が既存の描画ロジックを壊していないことを確認する。

ステップ 5.3: `is_valid_position` および関連ロジックを `current_board_height` に対応させる 
(Refactor)

1. Refactor: is_valid_position 関数が current_board_height を適切に利用して、テトリミノがプレイ
   可能な領域内に収まっているかをチェックするように変更する。
2. Refactor: draw 関数など、ボードの高さに依存する他の描画ロジックも current_board_height 
   に対応させる。
3. Test: このRefactorによって、既存のテストがすべてパスすることを確認する。特に、`is_valid_position` や `draw` 関数が `current_board_height` に対応した変更によって既存の動作を壊していないことを確認する。

