use libzeroengine::{
    script::Script,
    ucfb::{Chunk, DecipheredChunk, UCFBFile},
};
use std::{env, fs, path::Path, process::exit};

fn handle_chunks(chunks: Vec<Chunk>, prefix: &str) {
    for (i, chunk) in chunks.iter().enumerate() {
        chunk
            .clone()
            .deciphered_chunk
            .and_then(|c| -> Option<DecipheredChunk> {
                match c {
                    DecipheredChunk::Script(x) => {
                        println!("Found script {} with info {}", x.name, x.info);
                        fs::create_dir_all(prefix).unwrap();
                        fs::write(
                            format!("{}{}.luac", prefix, x.name),
                            x.get_lua_51_bytecode_from_50().unwrap(),
                        )
                        .unwrap();
                    }
                    DecipheredChunk::Movie(x) => {
                        println!("Found movie container at chunk #{}", i);
                        for (j, movie) in x.bink_files.iter().enumerate() {
                            fs::create_dir_all(format!("{}/mvs_block_{}/", prefix, i)).unwrap();
                            fs::write(format!("{}/mvs_block_{}/movie_{}.bik", prefix, i, j), movie)
                                .unwrap();
                        }
                    }
                    DecipheredChunk::UCFB(x) => {
                        handle_chunks(x.chunks, format!("{}/ucfb_{}/", prefix, i).as_str());
                    }
                    DecipheredChunk::Level(x) => {
                        handle_chunks(x.chunks, format!("{}/lvl_{}/", prefix, i).as_str());
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
        file = match UCFBFile::new(filename.clone()) {
            Ok(v) => v,
            Err(e) => panic!("Error: {:?}", e),
        };

        let extract_path = format!(
            "./{}/",
            Path::new(filename.as_str())
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        );
        file.visit_chunks().unwrap();
        handle_chunks(file.chunks, extract_path.as_str());
    }
}
