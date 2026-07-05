# Primer29 (ProMicro RP2040 / Vial)

![Primer29](https://raw.githubusercontent.com/yushakobo/build-documents/master/Primer29/imgs/PCB.JPG)

yushakobo [Primer29](https://github.com/yushakobo/build-documents/tree/master/Primer29) の
プロセッサボードを **ProMicro RP2040** に換装した個体向けの、
[vial-qmk](https://github.com/vial-kb/vial-qmk) ベースのファームウェアです。

* Keyboard Maintainer: [yushakobo](https://github.com/yushakobo) (original), RP2040/Vial port by qutto
* Hardware Supported: Primer29 PCB + ProMicro RP2040 (SparkFun Pro Micro RP2040 / RP2040-CE 互換ボード)
* Hardware Availability: https://shop.yushakobo.jp/

## 特徴

- 双方向マトリクス(duplex matrix、物理 5行×4列 → 論理 10行×4列)のカスタムスキャンを継承
- ピン割り当ては ProMicro footprint 互換の RP2040 GPIO に直接マッピング
  - rows: `GP5, GP6, GP7, GP8, GP9`(AVR の C6, D7, E6, B4, B5 相当)
  - cols: `GP29, GP28, GP27, GP26`(AVR の F4, F5, F6, F7 相当)
- Vial 対応(`vial` キーマップ)。アンロックコンボは **Ins(左上)+ P-(右上)長押し**
- リセットボタンのダブルタップでブートローダ(RPI-RP2)に入れます

## ビルド

vial-qmk のルートで:

```
make yushakobo/primer29:vial
```

デフォルト(Vial なしの素の QMK)キーマップ:

```
make yushakobo/primer29:default
```

生成された `yushakobo_primer29_vial.uf2` を、BOOTSEL を押しながら USB 接続して
現れる `RPI-RP2` ドライブにコピーすると書き込まれます。

詳細はリポジトリルートの `BUILD.md` を参照してください。
