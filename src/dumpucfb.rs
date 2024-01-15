use libzeroengine::{
    script::Script,
    ucfb::{Chunk, DecipheredChunk, UCFBFile, UCFBHeader},
};
use std::{env, fs, process::exit};

fn handle_chunks(chunks: Vec<Chunk>, prefix: &str) {
    for (i, chunk) in chunks.iter().enumerate() {
        chunk
            .clone()
            .deciphered_chunk
            .and_then(|c| -> Option<DecipheredChunk> {
                match c {
                    DecipheredChunk::Script(x) => {
                        println!("Found script {} with info {}", x.name, x.info);
                        fs::write(format!("{}{}.luac", prefix, x.name), x.get_lua_51_bytecode_from_50())
                            .unwrap();
                    }
                    DecipheredChunk::Movie(x) => {
                        println!("Found container at chunk #{}", i);
                        for (j, movie) in x.bink_files.iter().enumerate() {
                            fs::write(format!("{}mvs_file_{}_movie_{}.bik", prefix, i, j), movie).unwrap();
                        }
                    }
                    DecipheredChunk::UCFB(x) => {
                        handle_chunks(x.chunks, format!("{}sub_ucfb_{}", prefix, i).as_str());
                    }
                };
                None
            });
    }
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!(
            "Usage: {} [ucfb files]",
            env::current_exe()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        );
        exit(1);
    }
    let mut file: UCFBFile;
    args.remove(0);
    for filename in args {
        file = match UCFBFile::new(filename) {
            Ok(v) => v,
            Err(e) => panic!("Error: {:?}", e),
        };
        /*for chunk in file.chunks {
            println!("Found chunk: {} with size: {}", chunk.header.name, chunk.header.size);
            let s: Script = Script::from_chunk(chunk).unwrap();
            println!("Script name: {} Info: {}", s.name, s.info);
            fs::write(format!("{}.luac", s.name), s.get_lua_51_bytecode_from_50()).unwrap();
        }*/
        file.visit_chunks();
        handle_chunks(file.chunks, "");
    }
}
