use memory_rs::external::process::Process;
use simple_injector::inject_dll;
use std::env::current_exe;
use colored::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut targetprocess = "Cemu.exe";
    let mut next_is_target_process = false;

    println!("{}", "Arguments:".bright_blue());

    // Process command line args
    for argitem in &args {
        println!("\t{}", argitem.to_string().white());

        // Current arg is process name
        if next_is_target_process {
            targetprocess = argitem;
        }

        // Next arg is process name
        if argitem == "--targetprocess"
        || argitem == "--targetproc"  
        || argitem == "--tarprocess"  
        || argitem == "--tarproc" {
            next_is_target_process = true;
        } else {
            next_is_target_process = false;
        }
    }

    println!("{}{}{}", "Waiting for the process [".bright_white(), targetprocess.bright_blue(), "] to start".bright_white());
    println!("{}{}{}", "You can change the target process by using the".bright_white().dimmed() , " --targetprocess ".bright_blue().bold().dimmed(), "launch argument".bright_white().dimmed());

    let p = loop {
        if let Ok(p) = Process::new(targetprocess) {
            break p;
        }

        std::thread::sleep(std::time::Duration::from_secs(5));
    };

    println!("{}", "Game found!".bright_cyan());

    let mut path = current_exe().unwrap();
    path.pop();
    let path_string = path.to_string_lossy();

    let dll_path = format!("{}\\botw_freecam.dll", path_string).to_string();
    println!("{} {}", "DLL PATH:".bright_white(), dll_path.bright_blue());

    inject_dll(&p, &dll_path);
}
