use std::{fs::File, io::Read, net::SocketAddr, path::Path};

use clap::{arg, Command};
use iridium::{assembler, error::Result, remote::server::Server, repl, vm::VM};

const DEFAULT_LISTENING_ADDRESS: &str = "127.0.0.1:2244";

/// Attempts to read a file and return the contents. Exits if unable to read the file for any reason.
fn read_file(tmp: &str) -> Result<String> {
    let mut contents = String::new();
    let mut f = File::open(Path::new(tmp))?;
    f.read_to_string(&mut contents)?;
    Ok(contents)
}

/// Starts a REPL that will run until the user kills it
fn start_repl() -> Result<()> {
    let mut repl = repl::REPL::new();
    repl.run()?;

    Ok(())
}

/// Start a remote server in a background thread
fn start_remote_server(addr: SocketAddr) {
    let _t = std::thread::spawn(move || {
        let mut server = Server::new();
        server.run(addr)
    });
}

fn main() -> Result<()> {
    let args = Command::new("iridium")
        .version("1.0")
        .author("Vivi W. <polarsatellitest@gmail.com>")
        .about("Interpreter for the Iridium language")
        .arg(
            arg!(--file [INPUT_FILE] "Path to the .iasm or .ir file to run")
                .index(1)
                .short('f'),
        )
        .arg(arg!(--threads [THREADS] "Number of OS threads the VM will utilize").short('t'))
        .arg(arg!(--"enable-remote" "Enables the remote server component of Iridium VM"))
        .arg(arg!(--addr [ADDR] "Sets the listening address"))
        .get_matches();

    let num_threads = match args.get_one::<usize>("THREADS") {
        Some(thread_cnt) => *thread_cnt,
        None => num_cpus::get(),
    };

    if args.contains_id("enable-remote") {
        let default_addr = DEFAULT_LISTENING_ADDRESS.parse::<SocketAddr>().unwrap();
        let addr = args.get_one::<SocketAddr>("ADDR").unwrap_or(&default_addr);
        start_remote_server(*addr);
    }

    if let Some(filename) = args.get_one::<String>("INPUT_FILE") {
        match read_file(filename) {
            Ok(program) => {
                let mut asm = assembler::Assembler::new();
                let mut vm = VM::new();
                vm.logical_cores = num_threads;
                let program = asm.assemble(&program)?;
                vm.add_bytes(program);
                let events = vm.run();
                println!("VM Events");
                println!("--------------------------");
                for event in &events {
                    println!("{:#?}", event);
                }
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("There was an error opening that file: {:?}", e);
            }
        }
    }
    start_repl()?;

    Ok(())
}
