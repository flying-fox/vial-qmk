---
name: primer29-firmware
description: >
  yushakobo Primer29 (ProMicro RP2040換装) ファームウェアのソース修正・ビルドエラー対応・
  キーマップやレイアウト変更・Vialトラブル対応のためのプロジェクト知識。現行は RMK(Rust)版で、
  旧 vial-qmk(QMK)版の知識も含む。このリポジトリで Primer29 / RMK / rmk / vial-qmk / QMK /
  BidirectionalMatrix / duplex matrix / sequences.rs / シーケンスマクロ / keymap / vial.json /
  .uf2 / GitHub Actions ビルド / ビルドエラー / キーが効かない / キー位置がずれる /
  Vialで認識しない / 設定が保存されない などの話題が出たら、スキル名が明示されなくても
  必ずこのスキルを読むこと。実機デバッグで確定した設計判断と壊してはいけない不変条件がここに書いてある。
---

# Primer29 (ProMicro RP2040) ファームウェア修正ガイド

## 全体像(2系統ある)

yushakobo Primer29(29キー、テンキー+ナビ、duplex matrix)のプロセッサボードを
ProMicro AVR から ProMicro RP2040 に換装した個体向けファームウェア。

| 系統 | 場所 | 状態 |
|------|------|------|
| **RMK版(現行メインライン)** | `rmk/` | **実機で全キー+Vial+シーケンスマクロ動作確認済み** |
| QMK版(旧) | `keyboards/yushakobo/primer29/` | vial-qmk オーバーレイ。flying-fox/vial-qmk に PR #1 として提出済み。実機未検証 |

- ローカルリポジトリ: `C:\Users\qutto\claude\keyboard-firmware`(ブランチ `primer29-rp2040`)
- push 先: `flying-fox/vial-qmk` の **`rmk` ブランチ**(vial-qmk 本体とは無関係な履歴の同居ブランチ)
- GitHub アカウント: qutto1(コラボレーター)/ flying-fox(オーナー、同一人物の別アカウント。WSL ユーザー名 flyingfox)
- PAT は **repo + workflow の両スコープ必須**(workflow が無いと `.github/workflows/` を含む push が 403 で拒否される)

## ビルド(GitHub Actions。ローカルに Rust ツールチェーン無し)

`rmk/**` か workflow を変更して `rmk` ブランチに push すると
[.github/workflows/build-rmk.yml](../../../.github/workflows/build-rmk.yml) が走り、
Artifacts に `primer29-rmk-uf2`(primer29.uf2)が生成される。手動実行は workflow_dispatch。

- ツール: dtolnay/rust-toolchain@stable + thumbv6m-none-eabi + llvm-tools、
  flip-link / cargo-binutils / cargo-hex-to-uf2(cargo install)
- **`RUST_MIN_STACK: "33554432"` を必ず維持**。無いと依存クレート rmk-types のコンパイルで
  rustc 自体が SIGSEGV(スタックオーバーフロー)する
- ユーザーのWSL(別マシン)でのローカルビルドも可: `cargo make uf2`(Makefile.toml)

書き込み: BOOTSEL 押しながら USB 接続(または BOOTSEL+リセット同時押し)→ `RPI-RP2` に uf2 をコピー。

## RMK版の構成

- `rmk = "=0.8.2"` **完全固定**(BidirectionalMatrix の動作実績があるバージョン。安易に上げない)
- 論理マトリクス **8行×7列**:
  - 行0-4 = 物理キー29個 + 物理キー無し6マス((1,6)(2,0)(2,1)(2,2)(3,6)(4,3) — scan_map で Ignore)
  - 行5-7 = 仮想行(物理スキャン無し、シーケンスマクロ設定用。scan_map 全 Ignore)
- ファイル: `src/main.rs`(初期化+scan_map)/ `src/keymap.rs`(既定キーマップ+KeymapStore)/
  `src/sequences.rs`(シーケンスマクロ)/ `src/vial.rs` / `vial.json` / `build.rs` / `Makefile.toml` /
  `memory.x` / `.cargo/config.toml`

## 壊してはいけない不変条件(RMK版)

実機デバッグの末に確定したもの。理由ごと理解して守ること。

1. **ピン: rows = GP5,GP6,GP7,GP8,GP9 / cols = GP29,GP28,GP27,GP26**(pins配列 idx0-4=R0-R4, idx5-8=C0-C3)。
   GP4 ではない(ProMicro シルク「5-9」= RP2040 GP5-9。yushakobo オリジナルの rows C6,D7,E6,B4,B5 に対応)。
   1本ずれると「全キーが1行ずれ+最下行全滅」になる。
2. **scan_map の方向は active-high 前提**。RMK の BidirectionalMatrix は出力ピンを High に駆動して
   入力を読む。QMK(active-low)と電流方向が逆なので、QMK 語彙で「Rx駆動・Cy読み」のキー(RxCy)は
   RMK では `Pins(in=Rx, out=Cy)`。**「隣接列ペアの入れ替わり」が観測されたら方向が逆**と疑う。
3. **Key イベントは KEY_EVENT_CHANNEL に送る**(EVENT_CHANNEL ではない)。rmk 0.8.2 の
   `run_devices!` マクロの実挙動で、`Keyboard::run()` は KEY_EVENT_CHANNEL しか読まない。
   sequences.rs の `forward()` がこの振り分けを再現している。
4. **KeymapStore パターンを崩さない**(keymap.rs)。rmk 0.8.2 の KeyMap は中身を外から読む公開 API が
   皆無(全部 pub(crate))なので、キーマップ実体を static(UnsafeCell)に置き、`keymap_mut()` は
   main.rs で一度だけ呼び、sequences.rs は `read_layer0()`(生ポインタ+read_volatile)で読む。
   借用を持ったまま .await しない。
5. **三点整合を保つ**: keymap.rs(8×7×4層)/ main.rs の scan_map(8行、行5-7は全Ignore)/
   vial.json(matrix.rows=8, cols=7、ラベル50個 = 物理29+仮想21)。vial.json の検証ワンライナー:
   ```
   python -c "import json; v=json.load(open('rmk/vial.json',encoding='utf-8')); L=[i for r in v['layouts']['keymap'] for i in r if isinstance(i,str)]; C=[tuple(map(int,s.split(','))) for s in L]; assert len(L)==50==len(set(C)) and all(0<=r<8 and 0<=c<7 for r,c in C), 'NG'; print('OK')"
   ```
6. **USB ID は VID 0x4C4B / PID 0x4643** で vial.json の vendorId/productId と一致させる。
   serial_number は `vial:f64c2b3c:000001` 形式を維持(Vial 検出用)。
7. **CLEAR_STORAGE の2段運用**(main.rs の const)。通常は false(Vial 編集がフラッシュ保存される)。
   マトリクス次元や既定キーマップを変えたときは、true のビルドを一度書き込んでから
   false のビルドを書き込む(2段書き込み)。true のままだと Vial の編集が起動のたびに消える。
   RMK は reflash してもストレージのキーマップが優先されるため、「ソースを変えたのに反映されない」の
   原因は大抵これ。
8. Cargo.toml の依存を勝手に追加・更新しない(embassy-time 等は既存のもので賄う)。

## シーケンスマクロ(src/sequences.rs)

「キー1→キー2」の2キー連続入力で、設定したキー/マクロを出力する自作機能。

- **2つの発動パス**: タップパス(キー1をTAP_TERM=200ms以内にタップ→SEQ_GAP=300ms以内にキー2press)と
  重ね押しパス(キー1押下のままSEQ_OVERLAP=300ms以内にキー2press)。
- 状態機械5状態: Idle / Held1 / WaitSecond / Emitting / EmittingOverlap。タイムアウトは
  embassy_time::with_deadline でタイマー即時確定(ホールド遅延の最小化)。
  キー2を押し続けると出力キーがホールド(リピート)。release の同定は実位置(row,col)。
- **設定はすべて Vial 上**: 仮想行が Vial に表示され、縦1列=1シーケンス(最大7本)。
  - 行5 = TRIG1(キー1)/ 行6 = TRIG2(キー2)/ 行7 = OUT(出力キー or Vialマクロ)
  - TRIG1/TRIG2 のどちらかが KC_NO(または Transparent)の列は無効
  - **トリガー判定はレイヤー0の値のみ**(レイヤー状態を読むAPIが無いため)。行5-7 の
    レイヤー1-3 は Transparent 既定で触らない
- 既定: 列0=Home→PgUp、列1=Del→End、列2=↑→↓(出力は未設定)

## キー不具合の切り分けパターン(実績ベース)

| 症状 | 原因 |
|------|------|
| 全キーが1行ずれる+最下行だけ全滅 | 行ピンが1本ずれ(GP4開始になっている) |
| 隣接列ペア(0↔1, 2↔3, 4↔5)が入れ替わる | scan_map の in/out 方向が逆 |
| ソース変更(キーマップ)が反映されない | フラッシュの保存キーマップが優先されている → CLEAR_STORAGE true を一度 |
| Vial に設定行が出ない | 旧ファームのまま(vial.json はファーム内蔵。新 uf2 を書き込む) |
| CI で rustc SIGSEGV | RUST_MIN_STACK 未設定 |
| workflow ファイルの push が 403 | PAT に workflow スコープが無い |

## 過去の経緯(なぜこうなったか)

- WSL 試作版が動かなかった3原因: ①行ピン1本ずれ(GP4-8 と誤認。正は GP5-9)
  ②scan_map の方向が逆(active-high/active-low の差)③フラッシュ保存キーマップ。
  ①②は実測29キー全数照合で確定、③は RMK 公式FAQ 記載の仕様。
- 途中「奇数/偶数列が入れ替わる」報告はユーザー自身の WSL 部分修正ビルド由来で、
  リポジトリのソースは正しかった(観測報告がどのビルド由来かを必ず確認すること)。
- 実装は Sonnet サブエージェントに委任し(クレジット節約)、rmk-v0.8.2 の実ソースを
  raw.githubusercontent.com で読ませて API を確認させる方式で2ラウンドとも一発コンパイル成功。
  推測で API を書かせないことが成功要因。

## QMK版(旧・参考)

`keyboards/yushakobo/primer29/` は vial-qmk オーバーレイ(コピーしてビルドする方式)。
詳細手順は [BUILD.md](../../../BUILD.md)。不変条件:

1. keyboard.json に `matrix_pins` を書かない(duplex のため `matrix_size: {rows:10, cols:4}` +
   config.h の `MATRIX_ROW_PINS { GP5, GP6, GP7, GP8, GP9 }` / `MATRIX_COL_PINS { GP29, GP28, GP27, GP26 }`)。
2. `VIAL_KEYBOARD_UID = {0x8D,0xBA,0x27,0xDD,0x04,0x4F,0x07,0x01}` を変更しない。
3. 29キーの三重整合(keyboard.json / keymap.c / vial.json)。機械検証:
   `python .claude/skills/primer29-firmware/scripts/validate.py`(**QMK版専用**。RMK版には使えない)。
4. 論理行0-4 = row→col スキャン、5-9 = col→row(論理行 = 物理行+5)。
5. `development_board: promicro_rp2040` を維持、GPIO API は現行名(`gpio_set_pin_output` 等)。
6. USB ID は VID 0x3265 / PID 0x0012(yushakobo 踏襲。RMK版とは別 ID なので注意)。
7. Vial アンロック: QMK版 = Ins[0,0]+P-[0,3] / **RMK版 = Ins(0,0)+/(0,6)**。
