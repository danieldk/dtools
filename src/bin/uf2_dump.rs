use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use dtools::uf2::IntoU2FBlockIter;

fn main() -> anyhow::Result<()> {
    let flags = xflags::parse_or_exit!(
        /// UF2 firmware path.
        required uf2_path: PathBuf
    );

    let f = BufReader::new(File::open(flags.uf2_path)?);

    for block in f.u2f_block_iter() {
        let block = block?;
        print!(
            "Block: {} ({}), payload len: {}",
            block.block_no, block.num_blocks, block.payload_size
        );
        if let Some(family_id) = block.family_id() {
            print!(" family id: {:x}", family_id);
        }
        println!();
    }

    Ok(())
}
