---
name: primer29-firmware
description: >
  yushakobo Primer29 (ProMicro RP2040換装 + vial-qmk) ファームウェアのソース修正・ビルドエラー対応・
  キーマップやレイアウト変更・Vialトラブル対応のためのプロジェクト知識。このリポジトリで
  Primer29 / vial-qmk / QMK / keyboard.json / matrix.c / keymap / vial.json / .uf2 / QMK MSYS /
  ビルドエラー / キーが効かない / Vialで認識しない などの話題が出たら、スキル名が明示されなくても
  必ずこのスキルを読むこと。過去の設計判断と壊してはいけない不変条件がここに書いてある。
---

# Primer29 (ProMicro RP2040 / vial-qmk) ファームウェア修正ガイド

## プロジェクト概要

yushakobo Primer29(29キー、テンキー+ナビ)のプロセッサボードを ProMicro AVR から
ProMicro RP2040 に換装した個体向けの vial-qmk ベースファームウェア。
オリジナルは https://github.com/yushakobo/qmk_firmware の `primer29` ブランチ。

このリポジトリはオーバーレイ構成:
`keyboards/yushakobo/primer29/` を vial-qmk の `keyboards/yushakobo/` にコピーしてビルドする。
ユーザーのローカルには ARM ツールチェーンが無い(ビルドは QMK MSYS で本人が実行)。
手順の詳細は `BUILD.md` にあり、大きな変更をしたらそちらの記述も追随させること。

## 壊してはいけない不変条件

修正時は以下を必ず守る。どれもハマった末の決定なので理由ごと理解すること。

1. **keyboard.json に `matrix_pins` を書かない。**
   Primer29 は双方向マトリクス(duplex)で、物理ピンは5行×4列だが論理マトリクスは10行×4列。
   `matrix_pins` を書くと QMK が MATRIX_ROWS=5 を生成して不整合になる。
   代わりに `matrix_size: {rows: 10, cols: 4}` を keyboard.json に、
   `MATRIX_ROW_PINS` / `MATRIX_COL_PINS` を `config.h` に書く(CUSTOM_MATRIX = lite の正規の形)。

2. **`VIAL_KEYBOARD_UID` を変更しない**(`keymaps/vial/config.h`)。
   UID が変わると Vial 上で別キーボード扱いになり、保存済み設定との対応が切れる。

3. **29キーの三重整合を保つ。** レイアウト・キーマップを触ったら必ず揃える:
   - `keyboard.json` の `layouts.LAYOUT.layout`(29エントリ、matrix座標は重複なし・row<10・col<4)
   - `keymap.c` の各レイヤーの LAYOUT() 引数(各29個、キー順は keyboard.json の配列順と同一)
   - `keymaps/vial/vial.json` の KLE ラベル(`"行,列"` 形式29個、keyboard.json の matrix 集合と完全一致)
   修正後は `scripts/validate.py` で機械検証する(後述)。

4. **GPIO API は現行名を使う**(`gpio_set_pin_output`, `gpio_read_pin` など)。
   旧名(`setPinOutput` 等)は非推奨シムで、vial-qmk の更新で消える可能性がある。

5. **`development_board: promicro_rp2040` を維持。** `CONVERT_TO=promicro_rp2040` コンバータは
   vial-qmk で非推奨化済み(sparkfun_pm2040 / rp2040_ce に分割)なので使わない。
   ピンは GP 直書きなのでコンバータ自体が不要。

## ピン対応表(PCB配線は ProMicro footprint)

| 役割 | Pro Micro (AVR) | RP2040 GPIO |
|------|-----------------|-------------|
| rows | C6, D7, E6, B4, B5 | GP5, GP6, GP7, GP8, GP9 |
| cols | F4, F5, F6, F7 | GP29, GP28, GP27, GP26 |

SparkFun Pro Micro RP2040 と RP2040-CE 互換ボードでこれらのピンのマッピングは同一
(差分は Primer29 が使わない LED ピンのみ)。

## 論理マトリクスの読み方(キー不具合の切り分けに必須)

`matrix.c` は両方向スキャンする:
- **論理行 0〜4**: 行ピンを駆動して列を読む(row→col)
- **論理行 5〜9**: 列ピンを駆動して行を読む(col→row)。論理行 = 物理行 + 5

つまり「論理行 0〜4 のキーだけ効く/5〜9 だけ効かない」ならダイオード向きや片方向の
はんだ不良を疑う。Vial のマトリクステスターで確認させると早い。

## 修正後の検証(ビルド前に必ず)

```
python .claude/skills/primer29-firmware/scripts/validate.py
```

JSON 構文、29キー整合、matrix 座標の重複・範囲、keymap.c の引数個数をチェックする。
ローカルではコンパイルできないため、これが出荷前の唯一の機械検証。必ず通してから渡すこと。

## ビルド・書き込み(ユーザーが QMK MSYS 内で実行)

```bash
cd ~/vial-qmk
cp -r /c/Users/qutto/claude/keyboard-firmware/keyboards/yushakobo/primer29 keyboards/yushakobo/
make yushakobo/primer29:vial      # → yushakobo_primer29_vial.uf2 がルートに生成
```

書き込み: BOOTSEL 押しながら USB 接続 → `RPI-RP2` ドライブに .uf2 コピー。
2回目以降はリセットボタンのダブルタップでもブートローダに入れる
(config.h の `RP2040_BOOTLOADER_DOUBLE_TAP_RESET`)。

## ビルドエラーが持ち込まれたときの調べ方

vial-qmk は QMK 本体を定期的に取り込むため、時間が経つと API やスキーマが変わっている
可能性が高い。エラーメッセージを見たら:

1. まずエラーが**このキーボード定義起因**か**vial-qmk ベース変更起因**かを切り分ける。
   `keyboards/yushakobo/primer29/` 内のファイル名が出ていれば前者、
   `quantum/` や `platforms/` なら後者の可能性が高い。
2. ベース変更起因なら、vial-qmk リポジトリ(https://github.com/vial-kb/vial-qmk)の
   最新の類似キーボード(CUSTOM_MATRIX = lite を使う機種、例: 検索 `matrix_scan_custom`)が
   今どう書いているかを raw.githubusercontent.com 経由で確認し、それに合わせる。
3. keyboard.json のスキーマエラーなら vial-qmk の `data/schemas/keyboard.jsonschema` を確認。
4. 直したら validate.py を通し、ユーザーに再ビルドしてもらう。

## Vial まわりの既知事項

- レイアウト定義はファーム内蔵(サイドロード不要)。Vial アプリ / https://vial.rocks で自動認識。
- アンロックコンボ: **Ins(左上 [0,0])+ P-(右上 [0,3])長押し**。
- vial キーマップは4レイヤー(オリジナル3 + 予備1)。`DYNAMIC_KEYMAP_LAYER_COUNT` の
  既定値4と揃えてあるので、レイヤーを増やすなら両方直す。
- USB ID は VID `0x3265` / PID `0x0012`(yushakobo オリジナルを踏襲)。
