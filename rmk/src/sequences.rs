// シーケンスマクロ機能。
//
// 「キー1をタップ(素早く押して離す)→ 素早くキー2をタップ」すると、物理キーの無い
// マス(SEQ スロット)の位置で合成キーイベントを発行する状態機械。
// 出力される実際のキーコードは SEQ スロットに割り当てられた KeyAction(Vial で
// 自由に編集可能)を、通常のキー処理経路(Keyboard::run -> KeyMap::get_action_with_layer_cache)
// がそのまま解決する。つまりトリガー(キー1/キー2)はこのファイルのテーブルで固定するが、
// 出力内容は Vial 側でライブに変更できる。
//
// main.rs は本来 `run_devices!((matrix) => EVENT_CHANNEL)` マクロで matrix を
// つないでいたが、ここではそのマクロを経由せず matrix.read_event() を直接呼び出し、
// 本エンジン(状態機械)を通してから配送する。
//
// [重要] run_devices! マクロの実際の挙動(rmk 0.8.2, src/input_device/mod.rs で確認済み):
// マクロは Event::Key(..) を呼び出し側が指定したチャンネルにではなく、常に
// KEY_EVENT_CHANNEL へ送る。それ以外の Event バリアントだけが指定チャンネル
// (main.rs では EVENT_CHANNEL)へ送られる。Keyboard::run()(src/keyboard.rs)は
// KEY_EVENT_CHANNEL だけを受信するので、本エンジンが作る合成イベント・素通しイベントも
// 同じ振り分けルールに従わせる必要がある(forward() 関数がこれを再現している)。
// EVENT_CHANNEL への分岐は BidirectionalMatrix が Key イベントしか出さないため実質
// 到達しないが、他の Event バリアントが来た場合の安全策として残している。

use defmt::info;
use embassy_time::{Duration, Instant, with_deadline};
use rmk::channel::{EVENT_CHANNEL, KEY_EVENT_CHANNEL};
use rmk::event::{Event, KeyboardEvent};
use rmk::input_device::InputDevice;

// タップ判定時間(ms)。キー1を押してからこの時間以内に離せば「タップ」とみなす。
const TAP_TERM_MS: u64 = 200;
// シーケンス猶予時間(ms)。キー1を離してからキー2を押すまでに許される間隔。
const SEQ_GAP_MS: u64 = 300;

// (キー1の(row,col), キー2の(row,col), 出力スロットの(row,col))
//
// 出力スロットは物理キーの無い6マス(vial.json に追加済み: (1,6) (2,0) (2,1) (2,2) (3,6) (4,3))
// のいずれかを使う。トリガーの組み合わせを変えたい場合はこの配列を編集する。
// 使われていない出力スロットは将来のシーケンス追加用に予約されている。
const SEQUENCES: [((u8, u8), (u8, u8), (u8, u8)); 3] = [
    ((0, 1), (0, 2), (1, 6)), // Home → PgUp ⇒ SEQ1
    ((1, 0), (1, 1), (2, 0)), // Del  → End  ⇒ SEQ2
    ((3, 1), (4, 1), (2, 1)), // ↑    → ↓    ⇒ SEQ3
];

// シーケンスエンジンの状態。
// 保留イベントは(最大2個の) KeyboardEvent フィールドとして固定サイズで保持する(ヒープ不使用)。
enum State {
    // 何も保留していない。
    Idle,
    // キー1を押下中。deadline までに release が来るかを待っている(タップ判定中)。
    WaitRelease {
        seq_idx: usize,
        pending_press: KeyboardEvent,
        deadline: Instant,
    },
    // キー1をタップ済み。deadline までにキー2の press が来るかを待っている。
    WaitSecond {
        seq_idx: usize,
        tap_press: KeyboardEvent,
        tap_release: KeyboardEvent,
        deadline: Instant,
    },
    // 合成キーを出力中。キー2の release を待つ(タイムアウト無し = 押し続ければホールド)。
    Emitting { seq_idx: usize },
}

// kb が SEQUENCES のいずれかの「キー1 press」に一致するか調べ、一致すれば index を返す。
//
// KeyboardEvent の内部フィールド(pressed / pos)は rmk クレート内 pub(crate) で外部からは
// 読めないため、KeyboardEvent::key() で比較candidateを構築し PartialEq で比較する
// (これで pos と pressed の両方を一度に判定できる)。
fn find_key1_press(kb: KeyboardEvent) -> Option<usize> {
    SEQUENCES.iter().position(|&((r, c), _, _)| kb == KeyboardEvent::key(r, c, true))
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

// SEQ スロット位置の合成 KeyboardEvent を KEY_EVENT_CHANNEL に送る。
async fn emit_synth(pos: (u8, u8), pressed: bool) {
    KEY_EVENT_CHANNEL.send(KeyboardEvent::key(pos.0, pos.1, pressed)).await;
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
                    Event::Key(kb) => match find_key1_press(kb) {
                        Some(seq_idx) => State::WaitRelease {
                            seq_idx,
                            pending_press: kb,
                            deadline: Instant::now() + Duration::from_millis(TAP_TERM_MS),
                        },
                        None => {
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

            State::WaitRelease { seq_idx, pending_press, deadline } => {
                match with_deadline(deadline, matrix.read_event()).await {
                    Ok(e) => match e {
                        Event::Key(kb) => {
                            let (key1, _key2, _slot) = SEQUENCES[seq_idx];
                            if kb == KeyboardEvent::key(key1.0, key1.1, false) {
                                // 同じキーの release → タップ成立、キー2待ちへ(まだ転送しない)
                                State::WaitSecond {
                                    seq_idx,
                                    tap_press: pending_press,
                                    tap_release: kb,
                                    deadline: Instant::now() + Duration::from_millis(SEQ_GAP_MS),
                                }
                            } else {
                                // 別のキーイベント → 保留pressを転送 → そのイベントも転送 → Idle
                                forward(Event::Key(pending_press)).await;
                                forward(e).await;
                                State::Idle
                            }
                        }
                        other => {
                            // Key 以外のイベントは状態機械に影響させず素通しする
                            forward(other).await;
                            State::WaitRelease { seq_idx, pending_press, deadline }
                        }
                    },
                    Err(_) => {
                        // deadline 超過(ホールド操作) → 保留pressを転送してIdleへ
                        // (次イベントを待たずタイマーで即時に確定させることで、ホールド系
                        //  操作の入力遅延を最小化する)
                        forward(Event::Key(pending_press)).await;
                        State::Idle
                    }
                }
            }

            State::WaitSecond { seq_idx, tap_press, tap_release, deadline } => {
                match with_deadline(deadline, matrix.read_event()).await {
                    Ok(e) => match e {
                        Event::Key(kb) => {
                            let (key1, key2, slot) = SEQUENCES[seq_idx];
                            if kb == KeyboardEvent::key(key2.0, key2.1, true) {
                                // キー2 press 一致 → キー1のpress/release(および物理キー2自身の
                                // press)は破棄し、出力スロット位置の合成pressを送る
                                info!("sequence fired: idx={}", seq_idx as u8);
                                emit_synth(slot, true).await;
                                State::Emitting { seq_idx }
                            } else if kb == KeyboardEvent::key(key1.0, key1.1, true) {
                                // キー1の連打 → 保留タップを転送し、新pressを保留してWaitReleaseへ
                                forward(Event::Key(tap_press)).await;
                                forward(Event::Key(tap_release)).await;
                                State::WaitRelease {
                                    seq_idx,
                                    pending_press: kb,
                                    deadline: Instant::now() + Duration::from_millis(TAP_TERM_MS),
                                }
                            } else {
                                // それ以外の press/release → 保留タップは常に転送する
                                forward(Event::Key(tap_press)).await;
                                forward(Event::Key(tap_release)).await;
                                match find_key1_press(kb) {
                                    Some(new_idx) => {
                                        // ただしこのpressが別シーケンスのキー1なら、当該イベントは
                                        // 転送せずに保留してWaitReleaseへ
                                        State::WaitRelease {
                                            seq_idx: new_idx,
                                            pending_press: kb,
                                            deadline: Instant::now() + Duration::from_millis(TAP_TERM_MS),
                                        }
                                    }
                                    None => {
                                        forward(e).await;
                                        State::Idle
                                    }
                                }
                            }
                        }
                        other => {
                            forward(other).await;
                            State::WaitSecond { seq_idx, tap_press, tap_release, deadline }
                        }
                    },
                    Err(_) => {
                        // deadline 超過 → 保留タップ(press+release)を転送してIdleへ
                        forward(Event::Key(tap_press)).await;
                        forward(Event::Key(tap_release)).await;
                        State::Idle
                    }
                }
            }

            State::Emitting { seq_idx } => {
                // キー2を押し続ければ出力キーのホールド(リピート)になるため、
                // ここにはタイムアウトを設けない(release が来るまで無条件に待つ)。
                let e = matrix.read_event().await;
                match e {
                    Event::Key(kb) => {
                        let (_key1, key2, slot) = SEQUENCES[seq_idx];
                        if kb == KeyboardEvent::key(key2.0, key2.1, false) {
                            // キー2の release → 出力スロットの合成releaseを送ってIdleへ
                            emit_synth(slot, false).await;
                            State::Idle
                        } else {
                            forward(e).await;
                            State::Emitting { seq_idx }
                        }
                    }
                    other => {
                        forward(other).await;
                        State::Emitting { seq_idx }
                    }
                }
            }
        };
    }
}
