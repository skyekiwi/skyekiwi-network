use std::{
    path::PathBuf, 
    convert::TryInto, 
    io::{Read, Write},
    fs::File,
};
use clap::Clap;

#[derive(Clap)]
struct CliArgs {
    #[clap(long)]
    state_file: PathBuf,
    
    #[clap(long)]
    state_patch: Option<String>,

    #[clap(long)]
    output: PathBuf
}

pub fn unpad_size(size: &[u8; 4]) -> usize {
    if size.len() != 4 {
        panic!("Invalid size");
    }
    return (
        size[3] as usize | 
        ((size[2] as usize) << 8) | 
        ((size[1] as usize) << 16) | 
        ((size[0] as usize) << 24)
    ).into();
}

fn main() -> Result<(), std::io::Error> {
    let cli_args = CliArgs::parse();

    let state_patch: Vec<u8> = bs58::decode(&cli_args.state_patch.unwrap_or_default()).into_vec().unwrap();

    let mut origin_file = File::open(cli_args.state_file)?;
    let mut origin = Vec::new();
    origin_file.read_to_end(&mut origin)?;

    let state_patch_len = state_patch.len();
    let mut p = 0;

    let mut output = Vec::new();
    while p < state_patch_len {
        let patch_len = unpad_size(&state_patch[p..p + 4].try_into().unwrap());
        let patch_bytes = &state_patch[p + 4..p + 4 + patch_len];
        let patch = skw_myers_diff::bytes_to_diff_ops(patch_bytes);
        let patched = skw_myers_diff::patch(patch, &origin);
        output.extend_from_slice(&patched[..]);

        p += 4 + patch_len;
    }

    let mut output_file = File::create(cli_args.output)?;
    output_file.write_all(&output)?;

    Ok(())
}
