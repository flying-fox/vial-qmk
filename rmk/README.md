# Primer29 RMK ファームウェア (ProMicro RP2040)

遊舎工房 Primer29 のプロセッサボードを ProMicro RP2040 に換装した個体向けの、
[RMK](https://github.com/HaoboGu/rmk) 0.8.2 ベースのファームウェアです。
Vial 対応(レイアウト内蔵)、duplex matrix は RMK の `BidirectionalMatrix` を使用しています。

## 以前動かなかった原因(修正済み)

WSL 上で試作したバージョンで「キー位置がバラバラ」「最下行が無反応」だった原因は2つ:

1. **行ピンが1本ずれていた**: Row は GP4-8 ではなく **GP5,6,7,8,9**(QMK 版 config.h と同じ)。
   GP9 が pins 配列に無かったため最下行(R4)が全滅し、他の行は1行ずれて認識されていた。
2. **スキャン駆動方向が逆だった**: QMK は active-low(駆動側を Low)、RMK の
   `BidirectionalMatrix` は active-high(駆動側を High)。ダイオードの導通方向に対して
   out/in の役割が QMK と逆転するため、`scan_map` の全 `Pins(a, b)` を `Pins(b, a)` に反転した。
   これが「隣接列ペアの入れ替わり」として観測されていた。

テスト配列(A,B,C…)とテンキー配列の両方の実測結果・全29キー(無反応マス含む)が
この修正内容で完全に一致することを確認済み。

3. **ストレージ**: RMK はキーマップをフラッシュに保存するため、ファーム書き換えだけでは
   キーマップが更新されない。`main.rs` の `CLEAR_STORAGE`(現在 `true`)で対処。

## ビルド (GitHub Actions)

`rmk/` 配下を変更して push すると、`.github/workflows/build-rmk.yml` が自動でビルドし、
Actions の Artifacts に **primer29-rmk-uf2**(`primer29.uf2`)が生成されます。
手動実行は Actions タブ → "Build RMK firmware (Primer29)" → Run workflow。

ローカル(要 Rust + cargo-make)では:

```bash
cd rmk
cargo make uf2   # → primer29.uf2
```

## 書き込み

1. BOOTSEL を押しながら USB 接続(または BOOTSEL+リセット同時押し)
2. `RPI-RP2` ドライブに `primer29.uf2` をコピー

## 書き込み後の手順(重要)

1. まず `CLEAR_STORAGE = true` のビルドを書き込み、全キーの配置が正しいか確認する
2. 正しければ `src/main.rs` の `CLEAR_STORAGE` を `false` に変えて再ビルド・再書き込み
   (これをしないと Vial での変更が起動のたびに消える)

## Vial

- https://vial.rocks/ (Chrome/Edge) またはデスクトップアプリで自動認識
- セキュリティアンロック: **Ins(左上)+ /(右上)を同時長押し**
- レイヤーは4枚(レイヤー0のみ割り当て済み、1-3は透過)

## キーマップ(レイヤー0)

| row\col | 0     | 1    | 2    | 3       | 4 | 5 | 6        |
|---------|-------|------|------|---------|---|---|----------|
| 0       | Ins   | Home | PgUp | NumLock | - | * | /        |
| 1       | Del   | End  | PgDn | 7       | 8 | 9 | (無)     |
| 2       | (無)  | (無) | (無) | 4       | 5 | 6 | +(2u)    |
| 3       | Shift | ↑    | Ctrl | 1       | 2 | 3 | (無)     |
| 4       | ←     | ↓    | →    | (無)    | 0 | . | Enter(2u)|

※ 最上段の記号の並びはヒアリング通り `- * /` にしてあるが、オリジナル QMK 版の
デフォルトは `/ * -` の順。実物のキーキャップと違っていたら Vial で入れ替えるか
`keymap.rs` の row0 を修正すること。

## ピン対応表

| 役割 | Pro Micro (AVR) | RP2040 GPIO | pins配列 idx |
|------|-----------------|-------------|--------------|
| R0-R4 | C6, D7, E6, B4, B5 | GP5, GP6, GP7, GP8, GP9 | 0-4 |
| C0-C3 | F4, F5, F6, F7 | GP29, GP28, GP27, GP26 | 5-8 |
