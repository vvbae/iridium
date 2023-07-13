use std::{fs::File, io::Read, net::SocketAddr, path::Path, thread};

use clap::{arg, Command};
use iridium::{
    assembler,
    error::{IridiumError, Result},
    remote::server::Server,
    repl,
    vm::VM,
};

const DEFAULT_CLIENT_LISTENING_ADDRESS: &str = "127.0.0.1:2244";
const DEFAULT_PEER_LISTENING_HOST: &str = "127.0.0.1";
const DEFAULT_PEER_LISTENING_PORT: &str = "2254";
const DEFAULT_NODE_ALIAS: &str = "";
const DEFAULT_DATA_DIR: &str = "/var/lib/iridium";

/// Attempts to read a file and return the contents. Exits if unable to read the file for any reason.
fn read_file(tmp: &str) -> Result<String> {
    let mut contents = String::new();
    let mut f = File::open(Path::new(tmp))?;
    f.read_to_string(&mut contents)?;
    Ok(contents)
}

/// Start a remote server in a background thread
fn start_remote_server(addr: SocketAddr) {
    thread::spawn(move || -> Result<()> {
        let mut server = Server::new();
        server.run(addr)
    });
}

fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    let default_client_addr = DEFAULT_CLIENT_LISTENING_ADDRESS
        .parse::<SocketAddr>()
        .unwrap();
    let default_peer_host = DEFAULT_PEER_LISTENING_HOST.to_string();
    let default_peer_port = DEFAULT_PEER_LISTENING_PORT.to_string();
    let default_node_alias = DEFAULT_NODE_ALIAS.to_string();

    let args = Command::new("iridium")
        .version("1.0")
        .author("Vivi W. <polarsatellitest@gmail.com>")
        .about("Interpreter for the Iridium language")
        .arg(
            arg!(--file <INPUT_FILE> "Path to the .iasm or .ir file to run")
                .index(1)
                .short('f'),
        )
        .arg(arg!(--threads <THREADS> "Number of OS threads the VM will utilize").short('t'))
        .arg(arg!(--"enable-remote" "Enables the remote server component of Iridium VM"))
        .arg(arg!(--addr <ADDR> "Sets the listening address for remote connections from clients"))
        .arg(arg!(--"peer-host" <PEER_HOST> "Sets the listening address for remote connections from peer nodes").short('h'))
        .arg(arg!(--"peer-port" <PEER_PORT> "Sets the listening port for remote connections from peer nodes").short('p'))
        .arg(arg!(--"data-dir" <DATA_DIR> "Root directory where the Iridium VM should store its data"))
        .arg(arg!(--"node-alias" <NODE_ALIAS> "An alias that can be used to refer to a running VM across a network"))
        .get_matches();

    if args.contains_id("enable-remote") {
        let addr = args
            .get_one::<SocketAddr>("addr")
            .unwrap_or(&default_client_addr);
        start_remote_server(*addr);
    }

    let num_threads = match args.get_one::<usize>("threads") {
        Some(thread_cnt) => *thread_cnt,
        None => num_cpus::get(),
    };

    let node_alias = args
        .get_one::<String>("node-alias")
        .unwrap_or(&default_node_alias);

    let peer_host = args
        .get_one::<String>("peer-host")
        .unwrap_or(&default_peer_host);
    let peer_port = args
        .get_one::<String>("peer-port")
        .unwrap_or(&default_peer_port);

    let mut vm = VM::new()
        .with_alias(node_alias)
        .with_cluster_bind(peer_host, peer_port);
    vm.logical_cores = num_threads;

    if let Some(filename) = args.get_one::<String>("file") {
        match read_file(filename) {
            Ok(program) => {
                let mut asm = assembler::Assembler::new();
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
    } else {
        let mut repl = repl::REPL::new(vm);
        let rx = repl.rx_pipe.take();
        thread::spawn(move || -> Result<()> {
            let chan = rx.unwrap();
            loop {
                match chan.recv() {
                    Ok(msg) => {
                        print!("{}", msg);
                        Ok(())
                    }
                    Err(e) => Err(IridiumError::Recv(e)),
                }?
            }
        });
        repl.run()?;
    }

    Ok(())
}
