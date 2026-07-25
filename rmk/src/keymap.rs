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
// | 1       | Del    | End  | PgDn   | 7       | 8  | 9   | (無)    |
// | 2       | (無)   | (無) | (無)   | 4       | 5  | 6   | +(2u)   |
// | 3       | Shift  | ↑    | Ctrl   | 1       | 2  | 3   | (無)    |
// | 4       | ←      | ↓    | →      | (無)    | 0  | .   | Enter   |
//
// (無) は scan_map 側で Ignore になっているマス。keymap 側も a!(No) で揃えること。
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
