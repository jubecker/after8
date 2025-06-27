use std::{collections::HashSet, process::exit};

use after8::chip8::{
    cpu::CPU,
    screen::{ConsoleRenderer, Renderer, Screen, VoidRenderer},
};
use log::{Metadata, Record};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("usage: {} -u -v <rom file>", args[0]);
        exit(1);
    }

    let params: HashSet<String> = HashSet::from_iter(args.clone());

    let verbose = params.contains("-v");
    let log_level = if verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    log::set_logger(&SimpleLogger)
        .map(|()| log::set_max_level(log_level))
        .unwrap();

    let no_ui = params.contains("-u");
    let renderer: Box<dyn Renderer> = if no_ui {
        Box::new(VoidRenderer)
    } else {
        Box::new(ConsoleRenderer)
    };
    let screen = Screen::new(renderer);

    let file_name = args.last().unwrap();
    let mut cpu = CPU::with_rom(screen, file_name);
    //cpu.run_n_ticks(200);
    cpu.run();
}

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        use std::io::Write;
        if self.enabled(record.metadata()) {
            let mut out = std::io::stderr();

            writeln!(out, "{} - {}", record.level(), record.args()).unwrap();
        }
    }

    fn flush(&self) {}
}
