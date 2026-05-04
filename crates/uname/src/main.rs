use std::{env, process};

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let output = match args.as_slice() {
        [] => "Linux",
        [arg] if arg == "-s" || arg == "--kernel-name" => "Linux",
        [arg] if arg == "-a" || arg == "--all" => "Linux wasix 0.0.0 wasix wasm32 GNU/Linux",
        [arg] if arg == "-m" || arg == "--machine" => "wasm32",
        [arg] if arg == "-o" || arg == "--operating-system" => "GNU/Linux",
        [arg] if arg == "-r" || arg == "--kernel-release" => "0.0.0",
        [arg] if arg == "-v" || arg == "--kernel-version" => "wasix",
        [arg] if arg == "-n" || arg == "--nodename" => "wasix",
        _ => {
            eprintln!("uname: unsupported arguments: {}", args.join(" "));
            process::exit(1);
        }
    };

    println!("{output}");
}
