// SPDX-License-Identifier: GPL-3.0-or-later

use crate::graphics::screen::Screen;
use alloc::collections::vec_deque::VecDeque;
use conquer_once::spin::Lazy;
use rgb::RGB8;
use spinning_top::Spinlock;
use vek::Vec2;
use x86_64::instructions::port::Port;
