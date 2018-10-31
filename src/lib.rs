extern crate cfg_if;
extern crate js_sys;
extern crate rustednes_core;
extern crate wasm_bindgen;

use rustednes_core::cartridge::*;
use rustednes_core::cpu::CPU_FREQUENCY;
use rustednes_core::input::*;
use rustednes_core::nes::*;
use rustednes_core::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};
use rustednes_core::sink::*;

mod utils;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

use std::io::Cursor;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

const CYCLES_PER_MICROSECOND: u64 = (CPU_FREQUENCY as f64 / 1e6) as u64 + 1;

const DISPLAY_PIXELS: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

const KEY_SPACE: u8 = 32;
const KEY_ENTER: u8 = 13;
const KEY_UP: u8 = 38;
const KEY_DOWN: u8 = 40;
const KEY_LEFT: u8 = 37;
const KEY_RIGHT: u8 = 39;
const KEY_Z: u8 = 90;
const KEY_X: u8 = 88;

struct NullAudioSink;

impl AudioSink for NullAudioSink {
    fn write_sample(&mut self, _frame: f32) {
        // Do nothing
    }

    fn samples_written(&self) -> usize {
        0
    }
}

#[wasm_bindgen]
pub struct System {
    nes: Nes,
    emulated_cycles: u64,
    video_frame_buffer: Vec<u32>,
    audio_frame_sink: Box<AudioSink>,
}

#[wasm_bindgen]
impl System {
    pub fn new(rom_data: &[u8]) -> System {
        let cartridge = Cartridge::load(&mut Cursor::new(rom_data)).expect("couldn't load rom");

        System {
            nes: Nes::new(cartridge),
            emulated_cycles: 0,
            video_frame_buffer: vec![0; DISPLAY_PIXELS],
            audio_frame_sink: Box::new(NullAudioSink {}),
        }
    }

    pub fn run_for(&mut self, time_microseconds: u32) -> bool {
        let target_cycles =
            self.emulated_cycles + time_microseconds as u64 * CYCLES_PER_MICROSECOND;

        let mut frame_rendered = false;
        while self.emulated_cycles < target_cycles {
            let mut video_frame_sink = WebVideoSink::new(&mut self.video_frame_buffer);
            let (cycles, _) = self
                .nes
                .step(&mut video_frame_sink, self.audio_frame_sink.as_mut());

            self.emulated_cycles += cycles as u64;

            if video_frame_sink.frame_written() {
                frame_rendered = true;
            }
        }

        frame_rendered
    }

    pub fn get_frame(&self) -> *const u32 {
        self.video_frame_buffer.as_slice().as_ptr()
    }

    pub fn frame_width() -> u32 {
        SCREEN_WIDTH as u32
    }

    pub fn frame_height() -> u32 {
        SCREEN_HEIGHT as u32
    }

    pub fn key_down(&mut self, key_code: u8) {
        self.handle_key_event(key_code, true);
    }

    pub fn key_up(&mut self, key_code: u8) {
        self.handle_key_event(key_code, false);
    }

    fn handle_key_event(&mut self, key_code: u8, key_down: bool) {
        let button = match key_code {
            KEY_X => Button::A,
            KEY_Z => Button::B,
            KEY_SPACE => Button::Select,
            KEY_ENTER => Button::Start,
            KEY_UP => Button::Up,
            KEY_DOWN => Button::Down,
            KEY_LEFT => Button::Left,
            KEY_RIGHT => Button::Right,
            _ => return,
        };

        let game_pad_1 = &mut self.nes.interconnect.input.game_pad_1;
        game_pad_1.set_button_pressed(button, key_down);
    }
}
