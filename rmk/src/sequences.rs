// シーケンスマクロ機能。
//
// 「キー1をタップ → 素早くキー2をタップ」または「キー1を押したままキー2を押す(重ね押し
// /ロール打ち)」で、物理キーの無い仮想行 OUT(行7)の位置の合成キーイベントを発行する
// 状態機械。出力される実際のキーコードは OUT セルに割り当てられた KeyAction を、通常の
// キー処理経路(Keyboard::run -> KeyMap::get_action_with_layer_cache)がそのまま解決する。
//
// トリガー(キー1/キー2)もソース固定のテーブルではなく、仮想行 TRIG1(行5)/ TRIG2(行6)
// のキーマップ値との「キーコード比較」で判定する。行5-7 は Vial に表示されるので、
// トリガーも出力も Vial でライブに変更できる:
//   - 列 n = シーケンス n(最大 COL=7 個)
//   - (5,n) = キー1 のキーコード、(6,n) = キー2 のキーコード、(7,n) = 出力
//   - TRIG1/TRIG2 のどちらかが No(または Transparent)の列は無効
//
// キーマップの参照は crate::keymap::KEYMAP_STORE(キーマップ実体の static)の
// レイヤー0 を直接読む。rmk 0.8.2 には KeyMap の中身をクレート外から読む公開 API も
// レイヤー状態を読む API も無いため(詳細は keymap.rs の KeymapStore のコメント)、
// トリガー判定は「レイヤー0 の値」だけで行われることに注意。
//
// main.rs は本来 `run_devices!((matrix) => EVENT_CHANNEL)` マクロで matrix を
// つないでいたが、ここではそのマクロを経由せず matrix.read_event() を直接呼び出し、
// 本エンジン(状態機械)を通してから配送する。
//
// [重要] run_devices! マクロの実際の挙動(rmk 0.8.2, src/input_device/mod.rs で確認済み):
// マクロは Event::Key(..) を呼び出し側が指定したチャンネルにではなく、常に
// KEY_EVENT_CHANNEL へ送る。それ以外の Event バリアントだけが指定チャンネル
// (旧 main.rs では EVENT_CHANNEL)へ送られる。Keyboard::run()(src/keyboard.rs)は
// KEY_EVENT_CHANNEL だけを受信するので、本エンジンが作る合成イベント・素通しイベントも
// 同じ振り分けルールに従わせる必要がある(forward() 関数がこれを再現している)。

use defmt::info;
use embassy_time::{Duration, Instant, with_deadline};
use rmk::channel::{EVENT_CHANNEL, KEY_EVENT_CHANNEL};
use rmk::event::{Event, KeyboardEvent};
use rmk::input_device::InputDevice;
use rmk::types::action::KeyAction;

use crate::keymap::{COL, KEYMAP_STORE, ROW};

// タップ判定時間(ms)。キー1を押してからこの時間以内に離せば「タップ」とみなす。
const TAP_TERM_MS: u64 = 200;
// シーケンス猶予時間(ms)。キー1を離してからキー2を押すまでに許される間隔。
const SEQ_GAP_MS: u64 = 300;
// 重ね押し猶予時間(ms)。キー1を押したまま、この時間以内にキー2を押せば発動する。
// 超過したらキー1は通常のホールド操作として確定する。
const SEQ_OVERLAP_MS: u64 = 300;

// 仮想行の行番号(keymap.rs のコメント表と対応)
const TRIG1_ROW: u8 = 5;
const TRIG2_ROW: u8 = 6;
const OUT_ROW: u8 = 7;

// シーケンスエンジンの状態。
// 保留イベントは KeyboardEvent フィールド(最大2個)として固定サイズで保持する(ヒープ不使用)。
enum State {
    // 何も保留していない。
    Idle,
    // キー1を押下中。重ね押し(キー2 press)とタップ(キー1 release)の両にらみで待つ。
    Held1 {
        seq_col: u8,
        key1_pos: (u8, u8),           // キー1の実位置(release 同定用)
        pending_press: KeyboardEvent, // 保留中の press(不成立時にそのまま転送する)
        pressed_at: Instant,          // タップ判定(TAP_TERM_MS)用の押下時刻
        deadline: Instant,            // = pressed_at + SEQ_OVERLAP_MS
    },
    // キー1をタップ済み。deadline までにキー2の press が来るかを待っている。
    WaitSecond {
        seq_col: u8,
        tap_press: KeyboardEvent,
        tap_release: KeyboardEvent,
        deadline: Instant,
    },
    // タップパスで発動中。キー2の release を待つ(タイムアウト無し = 押し続ければホールド)。
    Emitting {
        seq_col: u8,
        key2_pos: (u8, u8), // キー2の実位置(release 同定用)
    },
    // 重ね押しパスで発動中。キー1がまだ物理的に押されているため、キー1・キー2両方の
    // release を(順不同で)待つ。キー1の release は破棄し(press を送っていないため)、
    // キー2の release で合成 release を送る。両方処理し終えたら Idle へ。
    EmittingOverlap {
        seq_col: u8,
        key1_pos: (u8, u8),
        key2_pos: (u8, u8),
        key1_released: bool,
        key2_released: bool,
    },
}

// KeyboardEvent から (row, col, pressed) を復元する。
// フィールド(pos / pressed)は rmk クレート内 pub(crate) で直接読めないため、
// 全マスの候補を KeyboardEvent::key() で構築し PartialEq で比較して特定する
// (8行×7列×2 = 最大112回の軽量比較。物理イベントは行0-4 なので実際はさらに少ない)。
// ロータリーエンコーダ等、キー位置イベントでないものは None。
fn decode_key_event(kb: KeyboardEvent) -> Option<(u8, u8, bool)> {
    for row in 0..ROW as u8 {
        for col in 0..COL as u8 {
            if kb == KeyboardEvent::key(row, col, true) {
                return Some((row, col, true));
            }
            if kb == KeyboardEvent::key(row, col, false) {
                return Some((row, col, false));
            }
        }
    }
    None
}

// トリガー設定値として有効か(No / Transparent は「未設定」扱い)。
fn is_trigger_set(a: KeyAction) -> bool {
    !matches!(a, KeyAction::No | KeyAction::Transparent)
}

// 列 n のシーケンスが有効か(TRIG1・TRIG2 の両方が設定済みか)。
fn seq_enabled(n: u8) -> bool {
    is_trigger_set(KEYMAP_STORE.read_layer0(TRIG1_ROW, n))
        && is_trigger_set(KEYMAP_STORE.read_layer0(TRIG2_ROW, n))
}

// 物理キー (row, col) のキーマップ値(レイヤー0)が、いずれかの有効な列の TRIG1
// (キー1)と一致するか調べ、一致した最初の列を返す。
fn find_key1_col(row: u8, col: u8) -> Option<u8> {
    let act = KEYMAP_STORE.read_layer0(row, col);
    if !is_trigger_set(act) {
        return None;
    }
    (0..COL as u8).find(|&n| seq_enabled(n) && act == KEYMAP_STORE.read_layer0(TRIG1_ROW, n))
}

// 物理キー (row, col) のキーマップ値(レイヤー0)が、列 seq_col の TRIG2(キー2)と
// 一致するか。
fn is_key2_of(row: u8, col: u8, seq_col: u8) -> bool {
    let act = KEYMAP_STORE.read_layer0(row, col);
    is_trigger_set(act) && seq_enabled(seq_col) && act == KEYMAP_STORE.read_layer0(TRIG2_ROW, seq_col)
}

// キー1押下(press 保留)状態を作る。
fn held1(seq_col: u8, key1_pos: (u8, u8), pending_press: KeyboardEvent) -> State {
    let now = Instant::now();
    State::Held1 {
        seq_col,
        key1_pos,
        pending_press,
        pressed_at: now,
        deadline: now + Duration::from_millis(SEQ_OVERLAP_MS),
    }
}

// イベントを本来の行き先チャンネルへ転送する。
// rmk 0.8.2 の run_devices! マクロと全く同じ振り分けルールを再現している:
// Key イベントは KEY_EVENT_CHANNEL へ、それ以外は EVENT_CHANNEL へ(満杯なら最古を捨てて送る)。
async fn forward(e: Event) {
    match e {
        Event::Key(key_event) => {
            KEY_EVENT_CHANNEL.send(key_event).await;
        }
        _ => {
            if EVENT_CHANNEL.is_full() {
                let _ = EVENT_CHANNEL.receive().await;
            }
            EVENT_CHANNEL.send(e).await;
        }
    }
}

// 列 seq_col の OUT セル位置 (7, seq_col) の合成 KeyboardEvent を KEY_EVENT_CHANNEL に送る。
async fn emit_synth(seq_col: u8, pressed: bool) {
    KEY_EVENT_CHANNEL.send(KeyboardEvent::key(OUT_ROW, seq_col, pressed)).await;
}

// matrix から読んだ生イベントを本状態機械に通してから配送する常駐タスク。
// main.rs の join3(...) の中で(spawnではなく)そのまま await される。
pub(crate) async fn run_sequence_engine<M: InputDevice>(mut matrix: M) -> ! {
    let mut state = State::Idle;
    loop {
        state = match state {
            State::Idle => {
                let e = matrix.read_event().await;
                match e {
                    Event::Key(kb) => match decode_key_event(kb) {
                        Some((r, c, true)) => match find_key1_col(r, c) {
                            // いずれかのシーケンスのキー1 press → 保留して Held1 へ
                            Some(seq_col) => held1(seq_col, (r, c), kb),
                            None => {
                                forward(e).await;
                                State::Idle
                            }
                        },
                        _ => {
                            forward(e).await;
                            State::Idle
                        }
                    },
                    other => {
                        forward(other).await;
                        State::Idle
                    }
                }
            }

            State::Held1 { seq_col, key1_pos, pending_press, pressed_at, deadline } => {
                match with_deadline(deadline, matrix.read_event()).await {
                    Ok(e) => match e {
                        Event::Key(kb) => match decode_key_event(kb) {
                            Some((r, c, true)) => {
                                if is_key2_of(r, c, seq_col) {
                                    // キー2 press(deadline 内)→ 重ね押しパスで発動。
                                    // キー1の press は転送せず破棄し、合成 press を送る。
                                    info!("sequence fired (overlap): col={}", seq_col);
                                    emit_synth(seq_col, true).await;
                                    State::EmittingOverlap {
                                        seq_col,
                                        key1_pos,
                                        key2_pos: (r, c),
                                        key1_released: false,
                                        key2_released: false,
                                    }
                                } else if let Some(new_col) = find_key1_col(r, c) {
                                    // 別シーケンスのキー1 press → 保留 press を転送して
                                    // 新しい Held1 へ(新 press を保留し直す)
                                    forward(Event::Key(pending_press)).await;
                                    held1(new_col, (r, c), kb)
                                } else {
                                    // 無関係の press → 保留 press 転送 → 当該転送 → Idle
                                    forward(Event::Key(pending_press)).await;
                                    forward(e).await;
                                    State::Idle
                                }
                            }
                            Some((r, c, false)) => {
                                if (r, c) == key1_pos {
                                    if Instant::now() < pressed_at + Duration::from_millis(TAP_TERM_MS) {
                                        // TAP_TERM 内の release → タップ成立、キー2待ちへ
                                        // (press/release とも未転送のまま保留)
                                        State::WaitSecond {
                                            seq_col,
                                            tap_press: pending_press,
                                            tap_release: kb,
                                            deadline: Instant::now() + Duration::from_millis(SEQ_GAP_MS),
                                        }
                                    } else {
                                        // 遅い release(TAP_TERM 超~OVERLAP 内)→ タップ不成立。
                                        // press / release をそのまま転送する
                                        forward(Event::Key(pending_press)).await;
                                        forward(Event::Key(kb)).await;
                                        State::Idle
                                    }
                                } else {
                                    // 他キーの release → 保留 press 転送 → 当該転送 → Idle
                                    forward(Event::Key(pending_press)).await;
                                    forward(e).await;
                                    State::Idle
                                }
                            }
                            None => {
                                // キー位置イベント以外(エンコーダ等)は素通しして状態維持
                                forward(e).await;
                                State::Held1 { seq_col, key1_pos, pending_press, pressed_at, deadline }
                            }
                        },
                        other => {
                            forward(other).await;
                            State::Held1 { seq_col, key1_pos, pending_press, pressed_at, deadline }
                        }
                    },
                    Err(_) => {
                        // deadline(SEQ_OVERLAP_MS)超過 = キー1ホールド → 保留 press を
                        // 転送して Idle へ(次イベントを待たずタイマーで即時確定させることで、
                        // ホールド系操作の入力遅延を最小化する)
                        forward(Event::Key(pending_press)).await;
                        State::Idle
                    }
                }
            }

            State::WaitSecond { seq_col, tap_press, tap_release, deadline } => {
                match with_deadline(deadline, matrix.read_event()).await {
                    Ok(e) => match e {
                        Event::Key(kb) => match decode_key_event(kb) {
                            Some((r, c, true)) => {
                                if is_key2_of(r, c, seq_col) {
                                    // キー2 press → タップパスで発動。キー1のタップは破棄し、
                                    // 合成 press を送る
                                    info!("sequence fired (tap): col={}", seq_col);
                                    emit_synth(seq_col, true).await;
                                    State::Emitting { seq_col, key2_pos: (r, c) }
                                } else if let Some(new_col) = find_key1_col(r, c) {
                                    // キー1の連打(new_col == seq_col)または別シーケンスの
                                    // キー1 → 保留タップを転送し、新 press を保留して Held1 へ
                                    forward(Event::Key(tap_press)).await;
                                    forward(Event::Key(tap_release)).await;
                                    held1(new_col, (r, c), kb)
                                } else {
                                    forward(Event::Key(tap_press)).await;
                                    forward(Event::Key(tap_release)).await;
                                    forward(e).await;
                                    State::Idle
                                }
                            }
                            Some(_) => {
                                // release イベント → 保留タップ転送 → 当該転送 → Idle
                                forward(Event::Key(tap_press)).await;
                                forward(Event::Key(tap_release)).await;
                                forward(e).await;
                                State::Idle
                            }
                            None => {
                                forward(e).await;
                                State::WaitSecond { seq_col, tap_press, tap_release, deadline }
                            }
                        },
                        other => {
                            forward(other).await;
                            State::WaitSecond { seq_col, tap_press, tap_release, deadline }
                        }
                    },
                    Err(_) => {
                        // deadline 超過 → 保留タップ(press+release)を転送して Idle へ
                        forward(Event::Key(tap_press)).await;
                        forward(Event::Key(tap_release)).await;
                        State::Idle
                    }
                }
            }

            State::Emitting { seq_col, key2_pos } => {
                // キー2を押し続ければ出力キーのホールド(リピート)になるため、
                // ここにはタイムアウトを設けない(release が来るまで無条件に待つ)。
                let e = matrix.read_event().await;
                match e {
                    Event::Key(kb) => match decode_key_event(kb) {
                        Some((r, c, false)) if (r, c) == key2_pos => {
                            // キー2の release → 合成 release を送って Idle へ
                            emit_synth(seq_col, false).await;
                            State::Idle
                        }
                        _ => {
                            forward(e).await;
                            State::Emitting { seq_col, key2_pos }
                        }
                    },
                    other => {
                        forward(other).await;
                        State::Emitting { seq_col, key2_pos }
                    }
                }
            }

            State::EmittingOverlap { seq_col, key1_pos, key2_pos, key1_released, key2_released } => {
                let e = matrix.read_event().await;
                match e {
                    Event::Key(kb) => match decode_key_event(kb) {
                        Some((r, c, false)) if (r, c) == key1_pos && !key1_released => {
                            // キー1の release → 破棄(press を送っていないので対応する
                            // release も消す)。キー2も処理済みなら Idle へ
                            if key2_released {
                                State::Idle
                            } else {
                                State::EmittingOverlap {
                                    seq_col,
                                    key1_pos,
                                    key2_pos,
                                    key1_released: true,
                                    key2_released,
                                }
                            }
                        }
                        Some((r, c, false)) if (r, c) == key2_pos && !key2_released => {
                            // キー2の release → 合成 release を送る。キー1も処理済みなら Idle へ
                            emit_synth(seq_col, false).await;
                            if key1_released {
                                State::Idle
                            } else {
                                State::EmittingOverlap {
                                    seq_col,
                                    key1_pos,
                                    key2_pos,
                                    key1_released,
                                    key2_released: true,
                                }
                            }
                        }
                        _ => {
                            forward(e).await;
                            State::EmittingOverlap { seq_col, key1_pos, key2_pos, key1_released, key2_released }
                        }
                    },
                    other => {
                        forward(other).await;
                        State::EmittingOverlap { seq_col, key1_pos, key2_pos, key1_released, key2_released }
                    }
                }
            }
        };
    }
}
