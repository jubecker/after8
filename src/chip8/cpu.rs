use log;

use super::{font_set::FontSet, screen::Screen};
use rand::random;

const RAM_SIZE: usize = 4048;
const START_ADDR: usize = 0x200;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const TICKS_PER_FRAME: usize = 10;

struct Stack {
    data: [usize; STACK_SIZE],
    sp: usize,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            sp: 0,
            data: [0; STACK_SIZE],
        }
    }

    pub fn push(&mut self, val: usize) {
        self.data[self.sp] = val;
        self.sp += 1;
    }

    pub fn pop(&mut self) -> usize {
        self.sp -= 1;
        self.data[self.sp]
    }
}

pub struct CPU {
    v_reg: [u8; NUM_REGS],
    ram: [u8; RAM_SIZE],
    pc: usize,
    i_reg: u16,
    dt: u8,
    st: u8,
    keys: [bool; NUM_KEYS],
    stack: Stack,
    screen: Screen,
}

impl CPU {
    pub fn new(screen: Screen) -> Self {
        let mut cpu = Self {
            v_reg: [0; NUM_REGS],
            ram: [0; RAM_SIZE],
            pc: START_ADDR,
            i_reg: 0,
            dt: 0,
            st: 0,
            keys: [false; NUM_KEYS],
            stack: Stack::new(),
            screen,
        };
        cpu.preload_fonts();

        cpu
    }

    pub fn with_rom(screen: Screen, filename: &str) -> Self {
        let mut cpu = Self::new(screen);
        let buf: Vec<u8> = std::fs::read(filename).unwrap();
        log::debug!("loaded rom {}, {} bytes", filename, buf.len());
        cpu.ram[START_ADDR..START_ADDR + buf.len()].clone_from_slice(&buf);
        cpu
    }

    fn preload_fonts(&mut self) {
        self.ram[..FontSet::FONTSET_SIZE].copy_from_slice(&FontSet::FONTSET);
    }

    pub fn run(&mut self) {
        loop {
            self.run_single_frame();
        }
    }

    pub fn run_n_ticks(&mut self, ticks: usize) {
        for _ in 0..ticks {
            self.run_single_frame();
        }
    }

    fn run_single_frame(&mut self) {
        for _ in 0..TICKS_PER_FRAME {
            self.tick();
        }
        self.screen.render();
        self.tick_timers();
    }

    pub fn tick(&mut self) {
        //log::debug!("tick with pc: {}", self.pc);
        let high_byte = self.ram[self.pc] as u16;
        let low_byte = self.ram[self.pc + 1] as u16;
        self.pc += 2;

        self.dispatch(high_byte, low_byte);
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            if self.st == 1 {
                // emit sound
                //log::debug!("Sound not implemented");
                print!(r"\a");
            }
            self.st -= 1;
        }
    }

    fn dispatch(&mut self, high_byte: u16, low_byte: u16) {
        let b3 = ((high_byte & 0xF0) >> 4) as u8;
        let b2 = (high_byte & 0x0F) as u8;
        let b1 = ((low_byte & 0xF0) >> 4) as u8;
        let b0 = (low_byte & 0x0F) as u8;
        let nn: u16 = low_byte & 0xFF;
        let nnn: u16 = ((high_byte << 8) | low_byte) & 0xFFF;

        match b3 {
            0 => match (b2, b1, b0) {
                (0, 0xE, 0xE) => self.oc_00ee(),
                (0, 0xE, 0) => self.oc_00e0(),
                (0, 0, 0) => self.oc_0000(),
                _ => unreachable!(),
            },
            1 => self.oc_1nnn(nnn),
            2 => self.oc_2nnn(nnn),
            3 => self.oc_3xnn(b2, nn),
            4 => self.oc_4xnn(b2, nn),
            5 if b0 == 0 => self.oc_5xy0(b2, b1),
            6 => self.oc_6xnn(b2, nn),
            7 => self.oc_7xnn(b2, nn),
            8 => match b0 {
                0 => self.oc_8xy0(b2, b1),
                1 => self.oc_8xy1(b2, b1),
                2 => self.oc_8xy2(b2, b1),
                3 => self.oc_8xy3(b2, b1),
                4 => self.oc_8xy4(b2, b1),
                5 => self.oc_8xy5(b2, b1),
                6 => self.oc_8xy6(b2, b1),
                7 => self.oc_8xy7(b2, b1),
                0xE => self.oc_8xye(b2, b1),
                _ => unreachable!(),
            },
            9 if b0 == 0 => self.oc_9xy0(b2, b1),
            0xA => self.oc_annn(nnn),
            0xB => self.oc_bnnn(nnn),
            0xC => self.oc_cxnn(b2, nn),
            0xD => self.oc_dxyn(b2, b1, b0),
            0xE => match (b1, b0) {
                (9, 0xE) => self.oc_ex9e(b2),
                (0xA, 1) => self.oc_exa1(b2),
                _ => unreachable!(),
            },
            0xF => match (b1, b0) {
                (0, 7) => self.oc_fx07(b2),
                (0, 8) => self.oc_fx08(b2),
                (1, 5) => self.oc_fx15(b2),
                (1, 8) => self.oc_fx18(b2),
                (1, 0xE) => self.oc_fx1e(b2),
                (2, 9) => self.oc_fx29(b2),
                (3, 3) => self.oc_fx33(b2),
                (5, 5) => self.oc_fx55(b2),
                (6, 5) => self.oc_fx65(b2),
                _ => unreachable!(),
            },
            _ => panic!("unsupported opcode {}{}{}{}", b3, b2, b1, b0),
        }
    }

    // NoOp
    fn oc_0000(&mut self) {
        log::debug!("exec oc_0000");
    }

    // Clear the screen
    fn oc_00e0(&mut self) {
        log::debug!("exec oc_00E0");
        self.screen.clear();
    }

    // Return from a subroutine
    fn oc_00ee(&mut self) {
        log::debug!("exec oc_00EE");
        let addr = self.stack.pop();
        self.pc = addr;
    }

    // Jump to address NNN
    fn oc_1nnn(&mut self, nnn: u16) {
        log::debug!("exec oc_1NNN");
        self.pc = nnn as usize;
    }

    // Execute subroutine starting at address NNN
    fn oc_2nnn(&mut self, nnn: u16) {
        log::debug!("exec oc_2NNN");
        self.stack.push(self.pc);
        self.pc = nnn as usize;
    }

    // Skip the following instruction if the value of register VX equals NN
    fn oc_3xnn(&mut self, x: u8, nn: u16) {
        log::debug!(
            "exec oc_3XNN - x: {}, nn: {}, v_reg: {:?}",
            x,
            nn,
            self.v_reg
        );
        let nn = nn as u8;
        if self.v_reg[x as usize] == nn {
            self.pc += 2;
        }
    }

    // Skip the following instruction if the value of register VX is not equal to NN
    fn oc_4xnn(&mut self, x: u8, nn: u16) {
        log::debug!("exec oc_4xNN");
        let nn = nn as u8;
        if self.v_reg[x as usize] != nn {
            self.pc += 2;
        }
    }

    // Skip the following instruction if the value of register VX is equal to the value of register VY
    fn oc_5xy0(&mut self, x: u8, y: u8) {
        log::debug!("exec oc_5XY0");
        if self.v_reg[x as usize] == self.v_reg[y as usize] {
            self.pc += 2;
        }
    }

    // Store number NN in register VX
    fn oc_6xnn(&mut self, x: u8, nn: u16) {
        log::debug!("exec oc_6XNN");
        let nn = nn as u8;
        self.v_reg[x as usize] = nn;
    }

    // Add the value NN to register VX
    fn oc_7xnn(&mut self, x: u8, nn: u16) {
        log::debug!("exec oc_7XNN");
        let nn = nn as u8;
        self.v_reg[x as usize] = self.v_reg[x as usize].wrapping_add(nn);
    }

    // Store the value of register VY in register VX
    fn oc_8xy0(&mut self, x: u8, y: u8) {
        log::debug!("exec oc_8XY0");
        self.v_reg[x as usize] = self.v_reg[y as usize];
    }

    // Set VX to VX OR VY
    fn oc_8xy1(&mut self, x: u8, y: u8) {
        log::debug!("exec oc_8XY1");
        let vx = self.v_reg[x as usize];
        let vy = self.v_reg[y as usize];
        self.v_reg[x as usize] = vx | vy;
    }

    // Set VX to VX AND VY
    fn oc_8xy2(&mut self, x: u8, y: u8) {
        log::debug!("exec oc_8XY2");
        let vx = self.v_reg[x as usize];
        let vy = self.v_reg[y as usize];
        self.v_reg[x as usize] = vx & vy;
    }

    // Set VX to VX XOR VY
    fn oc_8xy3(&mut self, x: u8, y: u8) {
        log::debug!("exec oc_8XY3");
        let vx = self.v_reg[x as usize];
        let vy = self.v_reg[y as usize];
        self.v_reg[x as usize] = vx ^ vy;
    }

    // Add the value of register VY to register VX
    // Set VF to 01 if a carry occurs
    // Set VF to 00 if a carry does not occur
    fn oc_8xy4(&mut self, x: u8, y: u8) {
        log::debug!("exec oc_8XY4");
        let vx = self.v_reg[x as usize];
        let vy = self.v_reg[y as usize];
        let (sum, over) = vx.overflowing_add(vy);
        log::debug!("oc_8xy4, {}+{}={}, overflow: {}", vx, vy, sum, over);
        let vf = if over { 1 } else { 0 };
        self.v_reg[x as usize] = sum;
        self.v_reg[0xF] = vf;
    }

    // Subtract the value of register VY from register VX
    // Set VF to 00 if a borrow occurs
    // Set VF to 01 if a borrow does not occur
    fn oc_8xy5(&mut self, x: u8, y: u8) {
        log::debug!("exec oc_8XY5");
        let vx = self.v_reg[x as usize];
        let vy = self.v_reg[y as usize];
        let (sub, over) = vx.overflowing_sub(vy);
        let vf = if over { 0 } else { 1 };
        self.v_reg[x as usize] = sub;
        self.v_reg[0xF] = vf;
    }

    // Store the value of register VY shifted right one bit in register VX
    // Set register VF to the least significant bit prior to the shift
    // VY is unchanged
    fn oc_8xy6(&mut self, x: u8, y: u8) {
        log::debug!("exec oc_8XY6");
        let vy = self.v_reg[y as usize];
        let lsb = vy & 1;
        self.v_reg[x as usize] = vy >> 1;
        self.v_reg[0xF] = lsb;
    }

    // Set register VX to the value of VY minus VX
    // Set VF to 00 if a borrow occurs
    // Set VF to 01 if a borrow does not occur
    fn oc_8xy7(&mut self, x: u8, y: u8) {
        log::debug!("exec oc_8XY7");
        let vx = self.v_reg[x as usize];
        let vy = self.v_reg[y as usize];
        let (sub, over) = vy.overflowing_sub(vx);
        let vf = if over { 0 } else { 1 };
        self.v_reg[x as usize] = sub;
        self.v_reg[0xF] = vf;
    }

    // Store the value of register VY shifted left one bit in register VX¹
    // Set register VF to the most significant bit prior to the shift
    // VY is unchanged
    fn oc_8xye(&mut self, x: u8, y: u8) {
        log::debug!("exec oc_8XYE");
        let vy = self.v_reg[y as usize];
        let msb = (vy >> 7) & 1;
        self.v_reg[x as usize] = vy << 1;
        self.v_reg[0xF] = msb;
    }

    // Skip the following instruction if the value of register VX is
    // not equal to the value of register VY
    fn oc_9xy0(&mut self, x: u8, y: u8) {
        log::debug!("exec oc_9XY0");
        if self.v_reg[x as usize] != self.v_reg[y as usize] {
            self.pc += 2;
        }
    }

    // Store memory address NNN in register I
    fn oc_annn(&mut self, nnn: u16) {
        log::debug!("exec oc_ANNN, set i_reg: {}", nnn);
        self.i_reg = nnn;
    }

    // Jump to address NNN + V0
    fn oc_bnnn(&mut self, nnn: u16) {
        log::debug!("exec oc_BNNN");
        self.pc = (nnn + self.v_reg[0] as u16) as usize;
    }

    // Set VX to a random number with a mask of NN
    fn oc_cxnn(&mut self, x: u8, nn: u16) {
        log::debug!("exec oc_CNNN");
        let nn = nn as u8;
        let rng: u8 = random();
        self.v_reg[x as usize] = rng & nn;
    }

    // Draw a sprite at position VX, VY with N bytes of sprite data starting
    // at the address stored in I.
    // Set VF to 01 if any set pixels are changed to unset, and 00 otherwise
    fn oc_dxyn(&mut self, x: u8, y: u8, n: u8) {
        log::debug!("exec oc_DXYN");
        let pos_x = self.v_reg[x as usize] as usize;
        let pos_y = self.v_reg[y as usize] as usize;
        let sprite_start = self.i_reg as usize;
        let sprite_end = sprite_start + n as usize;
        let sprite_bytes = &self.ram[sprite_start..sprite_end];
        let changed = self.screen.draw_sprite(sprite_bytes, pos_x, pos_y);
        self.v_reg[0xF] = if changed { 1 } else { 0 };
    }

    // Skip the following instruction if the key corresponding to
    // the hex value currently stored in register VX is pressed
    fn oc_ex9e(&mut self, x: u8) {
        log::debug!("exec oc_EX9E");
        if self.keys[self.v_reg[x as usize] as usize] {
            self.pc += 2;
        }
    }

    // Skip the following instruction if the key corresponding to
    // the hex value currently stored in register VX is not pressed
    fn oc_exa1(&mut self, x: u8) {
        log::debug!("exec oc_EXA1");
        if !self.keys[self.v_reg[x as usize] as usize] {
            self.pc += 2;
        }
    }

    // Store the current value of the delay timer in register VX
    fn oc_fx07(&mut self, x: u8) {
        log::debug!("exec oc_fx07");
        self.v_reg[x as usize] = self.dt;
    }

    // Wait for a keypress and store the result in register VX
    fn oc_fx08(&mut self, _x: u8) {
        log::debug!("exec oc_fx08");
        unimplemented!("Keypress nit implemented");
    }

    // Set the delay timer to the value of register VX
    fn oc_fx15(&mut self, x: u8) {
        log::debug!("exec oc_fx15");
        self.dt = self.v_reg[x as usize];
    }

    // Set the sound timer to the value of register VX
    fn oc_fx18(&mut self, x: u8) {
        log::debug!("exec oc_fx18");
        self.st = self.v_reg[x as usize];
    }

    // Add the value stored in register VX to register I
    fn oc_fx1e(&mut self, x: u8) {
        log::debug!("exec oc_fx1E");
        self.i_reg += self.v_reg[x as usize] as u16;
    }

    // Set I to the memory address of the sprite data corresponding to
    // the hexadecimal digit stored in register VX
    fn oc_fx29(&mut self, x: u8) {
        log::debug!("exec oc_fx29");
        self.i_reg = self.v_reg[x as usize] as u16 * FontSet::FONT_SPRITE_SIZE as u16;
    }

    // Store the binary-coded decimal equivalent of the value stored in
    // register VX at addresses I, I + 1, and I + 2
    fn oc_fx33(&mut self, x: u8) {
        log::debug!("exec oc_fx33");
        let vx = self.v_reg[x as usize];

        let hundreds = vx / 100;
        let rem = vx - (hundreds * 100);
        let tens = rem / 10;
        let ones = rem - (tens * 10);

        self.ram[self.i_reg as usize] = hundreds;
        self.ram[(self.i_reg + 1) as usize] = tens;
        self.ram[(self.i_reg + 2) as usize] = ones;
    }

    // Store the values of registers V0 to VX inclusive in memory starting at address I
    // I is set to I + X + 1 after operation²
    fn oc_fx55(&mut self, x: u8) {
        log::debug!("exec oc_fx55");
        for idx in 0..=x {
            self.ram[(self.i_reg + idx as u16) as usize] = self.v_reg[idx as usize];
        }
        self.i_reg += (x + 1) as u16;
    }

    // Fill registers V0 to VX inclusive with the values stored in memory starting at address I
    // I is set to I + X + 1 after operation²
    fn oc_fx65(&mut self, x: u8) {
        log::debug!("exec oc_fx65");
        for idx in 0..=x {
            self.v_reg[idx as usize] = self.ram[(self.i_reg + idx as u16) as usize];
        }
        self.i_reg += (x + 1) as u16;
    }
}
