use rmk::types::action::KeyAction;
use rmk::{a, k, layer};

pub(crate) const COL: usize = 7;
pub(crate) const ROW: usize = 5;
pub(crate) const NUM_LAYER: usize = 4;

// 論理マトリクス(5行×7列)と物理キーの対応:
//
// | row\col | 0      | 1    | 2      | 3       | 4  | 5   | 6       |
// |---------|--------|------|--------|---------|----|-----|---------|
// | 0       | Ins    | Home | PgUp   | NumLock | -  | *   | /       |
// | 1       | Del    | End  | PgDn   | 7       | 8  | 9   | SEQ1    |
// | 2       | SEQ2   | SEQ3 | (無)   | 4       | 5  | 6   | +(2u)   |
// | 3       | Shift  | ↑    | Ctrl   | 1       | 2  | 3   | (無)    |
// | 4       | ←      | ↓    | →      | (無)    | 0  | .   | Enter   |
//
// (無) は scan_map 側で Ignore になっているマス(物理キー無し)。keymap 側も a!(No) で揃えること。
//
// SEQ1/SEQ2/SEQ3 ((1,6) / (2,0) / (2,1)) も物理キーの無いマスだが、シーケンスマクロ
// (rmk/src/sequences.rs)が「キー1をタップ→素早くキー2をタップ」を検出したときに
// 合成キーイベントを送る出力スロットとして使う。既定値はここでは a!(No) のままにしてあり、
// Vial でこのマスに好きなキーコードやダイナミックマクロを割り当てると、それが
// シーケンスの出力になる。どのキーの組み合わせがどのスロットに対応するかは
// sequences.rs の SEQUENCES テーブルを参照。残り3マス((2,2) / (3,6) / (4,3))は
// 本当に未使用(将来シーケンスを追加する場合の予約枠)。
#[rustfmt::skip]
pub const fn get_default_keymap() -> [[[KeyAction; COL]; ROW]; NUM_LAYER] {
    [
        layer!([
            [k!(Insert), k!(Home), k!(PageUp),   k!(NumLock), k!(KpMinus), k!(KpAsterisk), k!(KpSlash)],
            [k!(Delete), k!(End),  k!(PageDown), k!(Kp7),     k!(Kp8),     k!(Kp9),        a!(No)],
            [a!(No),     a!(No),   a!(No),       k!(Kp4),     k!(Kp5),     k!(Kp6),        k!(KpPlus)],
            [k!(LShift), k!(Up),   k!(LCtrl),    k!(Kp1),     k!(Kp2),     k!(Kp3),        a!(No)],
            [k!(Left),   k!(Down), k!(Right),    a!(No),      k!(Kp0),     k!(KpDot),      k!(KpEnter)]
        ]),
        layer!([
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No)],
            [a!(No),          a!(No),          a!(No),          a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(No),          a!(Transparent), a!(Transparent), a!(Transparent)]
        ]),
        layer!([
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No)],
            [a!(No),          a!(No),          a!(No),          a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(No),          a!(Transparent), a!(Transparent), a!(Transparent)]
        ]),
        layer!([
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No)],
            [a!(No),          a!(No),          a!(No),          a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(No),          a!(Transparent), a!(Transparent), a!(Transparent)]
        ]),
    ]
}
