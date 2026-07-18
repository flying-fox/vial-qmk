use core::cell::UnsafeCell;

use rmk::types::action::KeyAction;
use rmk::{a, k, layer};

pub(crate) const COL: usize = 7;
pub(crate) const ROW: usize = 8;
pub(crate) const NUM_LAYER: usize = 4;

// 論理マトリクス(8行×7列)。行0-4 が物理キー、行5-7 は物理キーの無い「仮想行」
// (シーケンスマクロの設定用セル。scan_map では全マス Ignore):
//
// | row\col | 0      | 1    | 2      | 3       | 4  | 5   | 6       |
// |---------|--------|------|--------|---------|----|-----|---------|
// | 0       | Ins    | Home | PgUp   | NumLock | -  | *   | /       |
// | 1       | Del    | End  | PgDn   | 7       | 8  | 9   | (無)    |
// | 2       | (無)   | (無) | (無)   | 4       | 5  | 6   | +(2u)   |
// | 3       | Shift  | ↑    | Ctrl   | 1       | 2  | 3   | (無)    |
// | 4       | ←      | ↓    | →      | (無)    | 0  | .   | Enter   |
// | 5       | TRIG1 行: 列 n = シーケンス n の「キー1」のキーコード     |
// | 6       | TRIG2 行: 列 n = シーケンス n の「キー2」のキーコード     |
// | 7       | OUT 行:  列 n = シーケンス n の出力(合成イベント発行位置)|
//
// (無) は scan_map 側で Ignore になっているマス(物理キー無し)。keymap 側も a!(No) で揃えること。
//
// 行5-7(シーケンスマクロ設定行)の動作は rmk/src/sequences.rs 冒頭のコメント参照。
// 「キー1をタップ→素早くキー2をタップ(重ね押しも可)」で、OUT 行のキーが入力される。
// 3行とも Vial に表示されるので、トリガーも出力も Vial でライブに変更できる。
// 既定はシーケンス0 = Home→PgUp、1 = Del→End、2 = ↑→↓ の3本(出力は未設定 = a!(No))。
// TRIG1/TRIG2 のどちらかが No のシーケンス(列)は無効。
// 注意: トリガー判定(行5-6 と物理キーの照合)は「レイヤー0 の値」だけで行われる
// (rmk 0.8.2 にレイヤー状態をクレート外から読む API が無いため)。OUT 行の解決は
// RMK 本体が行うため通常のレイヤー透過が効く(レイヤー1-3 は Transparent 既定)。
#[rustfmt::skip]
pub const fn get_default_keymap() -> [[[KeyAction; COL]; ROW]; NUM_LAYER] {
    [
        layer!([
            [k!(Insert), k!(Home),   k!(PageUp),   k!(NumLock), k!(KpMinus), k!(KpAsterisk), k!(KpSlash)],
            [k!(Delete), k!(End),    k!(PageDown), k!(Kp7),     k!(Kp8),     k!(Kp9),        a!(No)],
            [a!(No),     a!(No),     a!(No),       k!(Kp4),     k!(Kp5),     k!(Kp6),        k!(KpPlus)],
            [k!(LShift), k!(Up),     k!(LCtrl),    k!(Kp1),     k!(Kp2),     k!(Kp3),        a!(No)],
            [k!(Left),   k!(Down),   k!(Right),    a!(No),      k!(Kp0),     k!(KpDot),      k!(KpEnter)],
            [k!(Home),   k!(Delete), k!(Up),       a!(No),      a!(No),      a!(No),         a!(No)],
            [k!(PageUp), k!(End),    k!(Down),     a!(No),      a!(No),      a!(No),         a!(No)],
            [a!(No),     a!(No),     a!(No),       a!(No),      a!(No),      a!(No),         a!(No)]
        ]),
        layer!([
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No)],
            [a!(No),          a!(No),          a!(No),          a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(No),          a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)]
        ]),
        layer!([
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No)],
            [a!(No),          a!(No),          a!(No),          a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(No),          a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)]
        ]),
        layer!([
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No)],
            [a!(No),          a!(No),          a!(No),          a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(No),          a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)]
        ]),
    ]
}

/// RMK の KeyMap の実体(Vial の編集でライブに書き換わる配列)を保持する static ストア。
///
/// rmk 0.8.2 では KeyMap の中身をクレート外から読む公開 API が存在しない
/// (`layers` フィールド・`get_action_at`・`get_action_with_layer_cache`・
/// `Storage::read_keymap` はすべて pub(crate)。rmk-v0.8.2 の src/keymap.rs /
/// src/host/storage.rs で確認済み)。一方でシーケンスマクロ(sequences.rs)は
/// 「Vial で設定されたトリガーキー(行5-6)」をライブに参照する必要がある。
///
/// そこでキーマップの実体(initialize_keymap_and_storage に &mut で渡す配列)を
/// この static に置き、RMK(KeyMap)には main.rs で &mut を渡しつつ、sequences.rs は
/// `read_layer0()` でレイヤー0 のセルを volatile 読みする。
pub(crate) struct KeymapStore(UnsafeCell<[[[KeyAction; COL]; ROW]; NUM_LAYER]>);

// 安全性: 本ファームは RP2040 のシングルコア(embassy executor-thread)でのみ動作し、
// 割り込みハンドラからこの配列に触れることも無い。書き込み(RMK 内部の Vial 処理)と
// 読み出し(sequences.rs)はどちらも .await を跨がない同期区間で行われるため、協調
// スケジューリング上、同時アクセス(データ競合)は発生しない。
unsafe impl Sync for KeymapStore {}

impl KeymapStore {
    /// RMK(initialize_keymap_and_storage)に渡す &mut を取り出す。
    ///
    /// # Safety
    /// main.rs の初期化で一度だけ呼ぶこと(&mut の一意性を呼び出し側で保証する)。
    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn keymap_mut(&self) -> &'static mut [[[KeyAction; COL]; ROW]; NUM_LAYER] {
        unsafe { &mut *self.0.get() }
    }

    /// レイヤー0 の (row, col) の KeyAction を読む(シーケンスマクロのトリガー判定用)。
    ///
    /// KeyMap が保持する &mut と並行に読むため、参照を作らず生ポインタからの
    /// volatile 読みに限定する(最適化による読みの省略・並べ替えを防ぐ)。
    /// 同時アクセスが無いことは KeymapStore の Sync 実装のコメント参照。
    pub(crate) fn read_layer0(&self, row: u8, col: u8) -> KeyAction {
        unsafe {
            core::ptr::read_volatile(core::ptr::addr_of!(
                (*self.0.get())[0][row as usize][col as usize]
            ))
        }
    }
}

pub(crate) static KEYMAP_STORE: KeymapStore = KeymapStore(UnsafeCell::new(get_default_keymap()));
