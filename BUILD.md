# Primer29 (ProMicro RP2040 / Vial) ビルド・書き込み手順

Primer29 のプロセッサボードを ProMicro RP2040 に換装した個体向けの、
vial-qmk ベースファームウェアのビルドと書き込みの手順です。

このリポジトリの `keyboards/yushakobo/primer29/` を vial-qmk にコピーして使います。

## 対応ハードウェア

- Primer29 PCB(ProMicro 用スルーホールに RP2040 ボードを実装)
- ProMicro RP2040 系ボード:
  - SparkFun Pro Micro RP2040
  - いわゆる RP2040-CE(Community Edition)互換ボード(Aliexpress 等の USB-C クローン含む)

Primer29 が使用するピンはどちらの系統でもマッピングが同一のため、区別なく動作します。

| 役割 | Pro Micro (AVR) | RP2040 GPIO |
|------|-----------------|-------------|
| row0 | C6 | GP5  |
| row1 | D7 | GP6  |
| row2 | E6 | GP7  |
| row3 | B4 | GP8  |
| row4 | B5 | GP9  |
| col0 | F4 | GP29 |
| col1 | F5 | GP28 |
| col2 | F6 | GP27 |
| col3 | F7 | GP26 |

## 1. ビルド環境の準備(Windows)

[QMK MSYS](https://msys.qmk.fm/)(公式ビルド環境、ARM ツールチェーン込み)をインストールします:

```powershell
winget install QMK.QMKMSYS
```

インストール後、スタートメニューから **QMK MSYS** を起動します。以降のコマンドはすべて QMK MSYS のシェル内で実行します。

> WSL / Linux / macOS の場合: `git`, `python3`, `arm-none-eabi-gcc`, `make` があればビルドできます。
> Ubuntu 系なら `sudo apt install -y git python3-pip gcc-arm-none-eabi make` のあと
> `python3 -m pip install qmk` で同等の環境になります。

## 2. vial-qmk の取得

```bash
cd ~
git clone https://github.com/vial-kb/vial-qmk.git
cd vial-qmk
make git-submodule
```

`make git-submodule` は ChibiOS や pico-sdk などのサブモジュールを取得します(数分かかります)。

## 3. キーボード定義のコピー

このリポジトリの `keyboards/yushakobo/primer29` フォルダを、
vial-qmk の `keyboards/yushakobo/` 配下にコピーします。

QMK MSYS 内から行う場合(このリポジトリが `C:\Users\qutto\claude\keyboard-firmware` にある前提):

```bash
mkdir -p ~/vial-qmk/keyboards/yushakobo
cp -r /c/Users/qutto/claude/keyboard-firmware/keyboards/yushakobo/primer29 ~/vial-qmk/keyboards/yushakobo/
```

## 4. ビルド

```bash
cd ~/vial-qmk
make yushakobo/primer29:vial
```

成功すると、vial-qmk のルートに **`yushakobo_primer29_vial.uf2`** が生成されます。

Vial 機能なしの素の QMK 版が欲しい場合は `make yushakobo/primer29:default`。

## 5. 書き込み

1. キーボードの USB ケーブルを抜く
2. ProMicro RP2040 の **BOOTSEL ボタンを押しながら** USB 接続する
3. `RPI-RP2` という名前の USB ドライブとして認識される
4. `yushakobo_primer29_vial.uf2` をそのドライブにコピーする
5. 自動的に再起動し、キーボードとして認識されれば完了

**2回目以降**はファームウェアの機能により、リセットボタンの**ダブルタップ**でも
ブートローダ(RPI-RP2)に入れます(BOOTSEL 長押し接続も引き続き使えます)。

## 6. Vial での設定

- デスクトップアプリ: https://get.vial.today/ からダウンロード
- ブラウザ版: https://vial.rocks/ (Chrome / Edge)

レイアウト定義(vial.json)はファームウェアに内蔵されているため、
接続するだけで自動認識されます。JSON のサイドロードは不要です。

初回はセキュリティロックの解除を求められることがあります。
その場合は画面の指示に従い、**Ins(左上)と P-(右上)のキーを同時に長押し**してください。

## デフォルトキーマップ

オリジナルの Primer29 と同じ3レイヤー構成(テンキー + ナビゲーション)+ 予備の空レイヤー1枚です。
レイヤー1は `MO(1)`(4段目左端)、レイヤー2は `MO(2)`(4段目左から3番目)で切り替えます。
すべて Vial 上で自由に変更できます。

## トラブルシューティング

- **`RPI-RP2` ドライブが出ない**: BOOTSEL を押すタイミングが遅い可能性があります。ボタンを押した状態のまま USB を挿してください。ケーブルがデータ通信対応かも確認を。
- **キーボードとして認識されない**: ビルドしたキーマップが `vial` か確認。デバイスマネージャで VID `0x3265` / PID `0x0012` のデバイスが見えるか確認してください。
- **一部のキーだけ効かない**: 双方向マトリクスのため、ダイオードの向き・はんだ不良の影響が「行の半分」単位で出ます。Vial のマトリクステスターで、効かないキーの論理行(0〜4 = 行→列スキャン、5〜9 = 列→行スキャン)に偏りがないか確認すると切り分けやすいです。
- **Vial がキーボードを認識しない**: ブラウザ版の場合は HID 権限のダイアログを許可したか確認。アプリ版で認識しない場合は USB ポートを変えてみてください。
- **ビルドエラーが出る**: vial-qmk は更新が続いているため、将来のバージョンで API が変わる可能性があります。エラーメッセージを添えて相談してください。
