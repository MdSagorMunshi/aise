use std::env;
use std::io::{self, Read};
use std::fs::File;
use aise_core::sponge::{aise_hash, aise_xof};

fn print_help() {
    println!("AEGIS-Ω (AISE) Cryptographic Hash Family CLI");
    println!("Usage:");
    println!("  aise-cli [OPTIONS]");
    println!("");
    println!("Options:");
    println!("  -h, --help, -help            Print this help message");
    println!("  -s <string>                  Hash the provided string");
    println!("  -f <file>                    Hash the provided file");
    println!("  -l <output_len_bytes>        Specify output length in bytes (default 64)");
    println!("");
    println!("If neither -s nor -f is provided, the tool will read from standard input.");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.iter().any(|a| a == "--help" || a == "-h" || a == "-help" || a == "help") {
        print_help();
        return;
    }
    
    let mut data = Vec::new();
    let mut output_len = 64; // Default to 512 bits (64 bytes)
    let mut use_xof = false;

    let mut i = 1;
    let mut input_provided = false;

    while i < args.len() {
        match args[i].as_str() {
            "-s" => {
                if i + 1 < args.len() {
                    data = args[i + 1].as_bytes().to_vec();
                    input_provided = true;
                    i += 2;
                } else {
                    eprintln!("Error: -s requires a string argument.");
                    return;
                }
            }
            "-f" => {
                if i + 1 < args.len() {
                    let path = &args[i + 1];
                    match File::open(path) {
                        Ok(mut file) => {
                            file.read_to_end(&mut data).expect("Failed to read file");
                            input_provided = true;
                        }
                        Err(e) => {
                            eprintln!("Error opening file {}: {}", path, e);
                            return;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("Error: -f requires a file path argument.");
                    return;
                }
            }
            "-l" => {
                if i + 1 < args.len() {
                    output_len = args[i + 1].parse().unwrap_or(64);
                    // If they specifically ask for more than default, we can use XOF
                    if output_len > 64 {
                        use_xof = true;
                    }
                    i += 2;
                } else {
                    eprintln!("Error: -l requires a length argument.");
                    return;
                }
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                print_help();
                return;
            }
        }
    }

    if !input_provided {
        // Read from stdin if no -s or -f was provided
        io::stdin().read_to_end(&mut data).expect("Failed to read stdin");
    }
    
    let result = if use_xof || output_len > 64 {
        aise_xof(&data, output_len)
    } else {
        aise_hash(&data, output_len)
    };
    
    for b in result {
        print!("{:02x}", b);
    }
    println!();
}
