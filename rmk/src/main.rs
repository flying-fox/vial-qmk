#![no_main]
#![no_std]
#[macro_use]
mod keymap;
mod vial;
use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::flash::{Async, Flash};
use embassy_rp::gpio::Flex;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use keymap::{COL, ROW};
use rmk::channel::EVENT_CHANNEL;
use rmk::config::{BehaviorConfig, DeviceConfig, PositionalConfig, RmkConfig, StorageConfig, VialConfig};
use rmk::debounce::default_debouncer::DefaultDebouncer;
use rmk::futures::future::join3;
use rmk::input_device::Runnable;
use rmk::keyboard::Keyboard;
use rmk::matrix::bidirectional_matrix::ScanLocation::{Ignore, Pins};
use rmk::matrix::bidirectional_matrix::{BidirectionalMatrix, ScanLocation};
use rmk::{initialize_keymap_and_storage, run_devices, run_rmk};
use vial::{VIAL_KEYBOARD_DEF, VIAL_KEYBOARD_ID};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});
const FLASH_SIZE: usize = 2 * 1024 * 1024;

// duplex matrix の物理ピン総数 (R0-R4 の5本 + C0-C3 の4本)
const PIN_NUM: usize = 9;

// 初回書き込み時や、ストレージのレイアウトを変えた直後(マトリクス寸法の変更、
// StorageConfig.num_sectors の変更など)は true にして、フラッシュに残っている
// 旧データを消去する。動作を確認できたら false に戻して再ビルド・再書き込みすること。
// (true のままだと Vial で編集したキーマップやコンボが起動のたびに消える)
const CLEAR_STORAGE: bool = true;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("RMK start!");
    // Initialize peripherals
    let p = embassy_rp::init(Default::default());
    // Create the usb driver, from the HAL
    let driver = Driver::new(p.USB, Irqs);

    // Pin config: duplex matrix 用に全ピンを Flex として初期化。
    //
    // 行ピンは GP5..GP9(QMK 版 config.h と同じ。GP4 は誤り)。
    // ProMicro footprint 対応: rows C6,D7,E6,B4,B5 -> GP5,GP6,GP7,GP8,GP9
    //                          cols F4,F5,F6,F7    -> GP29,GP28,GP27,GP26
    let pins = [
        Flex::new(p.PIN_5),  // idx0: R0
        Flex::new(p.PIN_6),  // idx1: R1
        Flex::new(p.PIN_7),  // idx2: R2
        Flex::new(p.PIN_8),  // idx3: R3
        Flex::new(p.PIN_9),  // idx4: R4
        Flex::new(p.PIN_29), // idx5: C0
        Flex::new(p.PIN_28), // idx6: C1
        Flex::new(p.PIN_27), // idx7: C2
        Flex::new(p.PIN_26), // idx8: C3
    ];

    // duplex matrix のスキャンマップ(Primer29 用)。
    //
    // RMK の BidirectionalMatrix は出力ピンを High に駆動して入力ピンを読む(active-high)。
    // QMK(active-low: 駆動側を Low に落とす)とは電流の向きが逆になるため、
    // 導通試験のラベル RxCy(QMK 的には Rx 駆動・Cy 読み取り)のキーは、
    // RMK では Pins(in = Rx, out = Cy) で検出される。CyRx はその逆で Pins(in = Cy, out = Rx)。
    let scan_map: [[ScanLocation; COL]; ROW] = [
        // Row0: R0C0, C0R0, R0C1, C1R0, R0C2, C2R0, R0C3
        [Pins(0, 5), Pins(5, 0), Pins(0, 6), Pins(6, 0), Pins(0, 7), Pins(7, 0), Pins(0, 8)],
        // Row1: R1C0, C0R1, R1C1, C1R1, R1C2, C2R1, (無)
        [Pins(1, 5), Pins(5, 1), Pins(1, 6), Pins(6, 1), Pins(1, 7), Pins(7, 1), Ignore],
        // Row2: (無), (無), (無), C1R2, R2C2, C2R2, R2C3(2u "+")
        [Ignore, Ignore, Ignore, Pins(6, 2), Pins(2, 7), Pins(7, 2), Pins(2, 8)],
        // Row3: R3C0, C0R3, R3C1, C1R3, R3C2, C2R3, (無)
        [Pins(3, 5), Pins(5, 3), Pins(3, 6), Pins(6, 3), Pins(3, 7), Pins(7, 3), Ignore],
        // Row4: R4C0, C0R4, R4C1, (無), R4C2(2u "0"), C2R4, R4C3(2u Enter)
        [Pins(4, 5), Pins(5, 4), Pins(4, 6), Ignore, Pins(4, 7), Pins(7, 4), Pins(4, 8)],
    ];

    // Use internal flash to emulate eeprom
    let flash = Flash::<_, Async, FLASH_SIZE>::new(p.FLASH, p.DMA_CH0);
    let keyboard_device_config = DeviceConfig {
        vid: 0x4c4b,
        pid: 0x4643,
        manufacturer: "yushakobo",
        product_name: "Primer29",
        serial_number: "vial:f64c2b3c:000001",
    };
    // Vial セキュリティアンロック: Ins(左上 [0,0]) + /(右上 [0,6]) 同時長押し
    let vial_config = VialConfig::new(VIAL_KEYBOARD_ID, VIAL_KEYBOARD_DEF, &[(0, 0), (0, 6)]);
    let rmk_config = RmkConfig {
        device_config: keyboard_device_config,
        vial_config,
        ..Default::default()
    };
    // Initialize the storage and keymap
    let mut default_keymap = keymap::get_default_keymap();
    let storage_config = StorageConfig {
        clear_storage: CLEAR_STORAGE,
        // フラッシュ末尾から確保するストレージ領域のセクタ数(RP2040 は 1 セクタ = 4KB)。
        // 既定の 2(8KB)では、キーマップに加えて Vial のコンボ 128 個ぶんのレコード
        // (keyboard.toml の combo_max_num。1件あたり数十バイト)が入りきらない。
        // sequential-storage は更新のたびに追記して古いレコードをゴミにする方式なので、
        // 実データ量の数倍の余裕を見て 8 セクタ(32KB)確保する。
        num_sectors: 8,
        ..Default::default()
    };
    let mut behavior_config = BehaviorConfig::default();
    let mut per_key_config = PositionalConfig::default();
    let (keymap, mut storage) = initialize_keymap_and_storage(
        &mut default_keymap,
        flash,
        &storage_config,
        &mut behavior_config,
        &mut per_key_config,
    )
    .await;

    // Initialize the matrix + keyboard (duplex matrix)
    let debouncer = DefaultDebouncer::new();
    let mut matrix = BidirectionalMatrix::<_, _, PIN_NUM, ROW, COL>::new(pins, debouncer, scan_map);
    let mut keyboard = Keyboard::new(&keymap);

    // Start
    join3(
        run_devices! (
            (matrix) => EVENT_CHANNEL,
        ),
        keyboard.run(),
        run_rmk(&keymap, driver, &mut storage, rmk_config),
    )
    .await;
}
