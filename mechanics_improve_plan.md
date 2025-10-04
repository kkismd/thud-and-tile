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