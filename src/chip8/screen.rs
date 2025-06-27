use log;

pub trait Renderer {
    fn render(&self, screen: &Screen);
}

pub struct Screen {
    pixel: [bool; Self::SCREEN_WIDTH * Self::SCREEN_HEIGHT],
    renderer: Box<dyn Renderer>,
}

impl Screen {
    const SCREEN_WIDTH: usize = 64;
    const SCREEN_HEIGHT: usize = 32;

    pub fn new(renderer: Box<dyn Renderer>) -> Self {
        Self {
            pixel: [false; Self::SCREEN_WIDTH * Self::SCREEN_HEIGHT],
            renderer,
        }
    }

    pub fn clear(&mut self) {
        self.pixel.fill(false);
    }

    pub fn draw_sprite(&mut self, sprite: &[u8], pos_x: usize, pos_y: usize) -> bool {
        log::debug!("draw sprite at {}x{}: {:?}", pos_x, pos_y, sprite);
        let mut pixel_changed = false;
        for (i, sprite_byte) in sprite.iter().enumerate() {
            let y = (pos_y + i) % Self::SCREEN_HEIGHT;
            for bit in 0..8 {
                let px = (sprite_byte & (0b1000_0000 >> bit)) != 0;
                if px {
                    log::debug!("bit {}: {}", bit, px);
                    let x = (pos_x + bit) % Self::SCREEN_WIDTH;
                    let screen_idx = x + y * Self::SCREEN_WIDTH;
                    log::debug!("x: {}, y: {}, screen_idx: {}", x, y, screen_idx);
                    pixel_changed |= self.pixel[screen_idx] != px;
                    self.pixel[screen_idx] ^= px;
                }
            }
        }

        pixel_changed
    }

    pub fn render(&self) {
        self.renderer.render(self);
    }
}

pub struct VoidRenderer;

impl Renderer for VoidRenderer {
    fn render(&self, _screen: &Screen) {}
}

pub struct ConsoleRenderer;

impl ConsoleRenderer {
    const BLANK: char = ' ';
    const PIXEL: char = '█';

    fn update_screen(scr: &str) {
        print!("{esc}c{esc}[1;1H{scr}", esc = 27 as char, scr = scr);
    }
}

impl Renderer for ConsoleRenderer {
    fn render(&self, screen: &Screen) {
        let mut buf = String::with_capacity(Screen::SCREEN_HEIGHT * Screen::SCREEN_WIDTH);
        (0..Screen::SCREEN_HEIGHT).for_each(|r| {
            let row_start = r * Screen::SCREEN_WIDTH;
            screen.pixel[row_start..(row_start + Screen::SCREEN_WIDTH)]
                .iter()
                .for_each(|p| buf.push(if *p { Self::PIXEL } else { Self::BLANK }));
            buf.push('\n');
        });

        Self::update_screen(&buf);
    }
}
