extern crate libaeon;
extern crate getopts;

use std::io::prelude::*;
use std::env;
use std::fs::File;
use std::process;

use libaeon::bytecode_parser;
use libaeon::config::Config;
use libaeon::vm::machine::Machine;
use libaeon::vm::state::State;

fn print_usage(options: &getopts::Options) -> ! {
    println!("{}", options.usage("Usage: aeonvm FILE [OPTIONS]"));
    process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut options = getopts::Options::new();

    options.optflag("h", "help", "Shows this help message");
    options.optflag("v", "version", "Prints the version number");

    options.optmulti("I",
                     "include",
                     "A directory to search for bytecode files",
                     "DIR");

    let matches = match options.parse(&args[1..]) {
        Ok(matches) => matches,
        Err(error) => {
            println!("{}", error.to_string());
            print_usage(&options);
        }
    };

    if matches.opt_present("h") {
        print_usage(&options);
    }

    if matches.opt_present("v") {
        println!("aeonvm {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if matches.free.is_empty() {
        print_usage(&options);
    } else {
        let mut config = Config::new();
        let ref path = matches.free[0];

        if matches.opt_present("I") {
            for dir in matches.opt_strs("I") {
                config.add_directory(dir);
            }
        }

        config.populate_from_env();

        match File::open(path) {
            Ok(file) => {
                let mut bytes = file.bytes();

                match bytecode_parser::parse(&mut bytes) {
                    Ok(code) => {
                        let state = State::new(config);
                        let vm = Machine::new(state);
                        let status = vm.start(code);

                        if status.is_err() {
                            process::exit(1);
                        }
                    }
                    Err(error) => {
                        println!("Failed to parse file {}: {:?}", path, error);
                        process::exit(1);
                    }
                }
            }
            Err(error) => {
                println!("Failed to execute {}: {}", path, error.to_string());
                process::exit(1);
            }
        }
    }
}
