use nix::mount::{mount, MsFlags};
use nix::unistd::{chdir, chroot, sethostname};
use std::env;
use std::fs;
use unshare::{Command, Namespace};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} run|child [command]", args[0]);
        return;
    }

    let command = args[2].clone();

    match args[1].as_str() {
        "run" => run(command),
        "child" => child(command),
        _ => println!("Usage: {} run|child [command]", args[0]),
    }
}

fn run(command: String) {
    Command::new("/proc/self/exe")
        .arg("child")
        .arg(command)
        .unshare(&[Namespace::Uts, Namespace::Pid, Namespace::Mount])
        .spawn()
        .expect("Failed to spawn child process")
        .wait()
        .expect("Failed to wait for child process");
}

fn child(command: String) {
    sethostname("container").expect("Failed to set hostname");
    cg();
    chroot("/tmp/containers/ubuntu").expect("Failed to chroot");
    chdir("/").expect("Failed to chdir");
    mount(
        Some("proc"),
        "proc",
        Some("proc"),
        MsFlags::empty(),
        None::<&str>,
    )
    .expect("Failed to mount proc");
    Command::new(command)
        .spawn()
        .expect("Failed to spawn process")
        .wait()
        .expect("Failed to wait for process");
}

fn cg() {
    let cgroup_dir = "/sys/fs/cgroup/test";
    // mkdir -p cgroup_dir
    match fs::create_dir_all(cgroup_dir) {
        Ok(_) => println!("Created cgroup directory"),
        Err(e) => println!("Failed to create cgroup directory: {}", e),
    }

    // write 10 to pids.max
    let pids_max = format!("{}/pids.max", cgroup_dir);
    match fs::write(pids_max, "10") {
        Ok(_) => println!("Set pids.max to 10"),
        Err(e) => println!("Failed to set pids.max: {}", e),
    }

    // write pid to cgroup.procs
    let cgroup_procs = format!("{}/cgroup.procs", cgroup_dir);
    let pid = std::process::id();
    match fs::write(cgroup_procs, pid.to_string()) {
        Ok(_) => println!("Set cgroup.procs to {}", pid),
        Err(e) => println!("Failed to set cgroup.procs: {}", e),
    }
}
