// Copyright 2023 yushakobo (@yushakobo)
// SPDX-License-Identifier: GPL-2.0-or-later

#pragma once

/* Duplex matrix pins (ProMicro RP2040)
 *
 * The PCB is wired for a Pro Micro footprint. AVR -> RP2040 GPIO mapping
 * (identical on SparkFun Pro Micro RP2040 and RP2040-CE compatible boards):
 *   rows: C6->GP5, D7->GP6, E6->GP7, B4->GP8, B5->GP9
 *   cols: F4->GP29, F5->GP28, F6->GP27, F7->GP26
 */
#define MATRIX_ROW_PINS { GP5, GP6, GP7, GP8, GP9 }
#define MATRIX_COL_PINS { GP29, GP28, GP27, GP26 }

/* Double-tap the reset button to enter the RP2040 bootloader */
#define RP2040_BOOTLOADER_DOUBLE_TAP_RESET
#define RP2040_BOOTLOADER_DOUBLE_TAP_RESET_TIMEOUT 500U
