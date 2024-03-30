use clap::{Parser, ValueEnum};
use glob::glob;
use serde::Deserialize;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
use std::process;
use std::process::Command;
use std::process::Stdio;

#[derive(Parser)]
#[command(name = "sysdep")]
#[command(version = "0.1.0")]
#[command(about = "A simple system dependency tool to list/install the apt/pip dependencies", long_about = None)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[arg(value_enum)]
    command: CliCommand,
    /// Please include the path with the quote, e.g. './**/'
    #[arg(default_value = ".")]
    search_directory: PathBuf,
    #[arg(value_enum, default_value_t=DependencyType::Both)]
    dependency_type: DependencyType,
    #[arg(short, long, default_value = "system_dependencies.toml")]
    file_name: String,
    /// To perform apt update
    #[arg(long, action)]
    apt_update: bool,
    /// Retrieve the large-dependencies
    #[arg(long, action)]
    large_dep: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum CliCommand {
    List,
    Install,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum DependencyType {
    Both,
    Apt,
    Pip,
}

#[derive(Deserialize, Debug)]
struct Depend {
    apt: Option<Vec<String>>,
    pip: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct Dependencies {
    dependencies: Option<Depend>,
    #[serde(rename = "large-dependencies")]
    large_dependencies: Option<Depend>,
}

#[cfg(target_arch = "x86_64")]
#[derive(Deserialize, Debug)]
struct ArchitectureSpecific {
    #[serde(rename = "x86_64-unknown-linux-gnu")]
    arch: Option<Dependencies>,
}

#[cfg(target_arch = "aarch64")]
#[derive(Deserialize, Debug)]
struct ArchitectureSpecific {
    #[serde(rename = "aarch64-unknown-linux-gnu")]
    arch: Option<Dependencies>,
}

#[derive(Deserialize, Debug)]
struct MetaDepend {
    name: Option<String>,
    dependencies: Option<Depend>,
    #[serde(rename = "large-dependencies")]
    large_dependencies: Option<Depend>,
    target: Option<ArchitectureSpecific>,
}

fn get_dependencies(dependencies: Option<Depend>) -> (Vec<String>, Vec<String>) {
    let mut apts: Vec<String> = vec![];
    let mut pips: Vec<String> = vec![];
    dependencies.and_then(|dep| {
        match dep.apt {
            Some(apt) => {
                apts = apt;
            }
            None => (),
        }
        match dep.pip {
            Some(pip) => {
                pips = pip;
                None::<String>
            }
            None => None,
        }
    });

    return (apts, pips);
}

fn get_arch_dependencies(
    dependencies: Option<ArchitectureSpecific>,
    large_dep: bool,
) -> (Vec<String>, Vec<String>) {
    let mut arch_apts: Vec<String> = vec![];
    let mut arch_pips: Vec<String> = vec![];

    dependencies.and_then(|dep| match dep.arch {
        Some(dep_arch) => {
            if large_dep {
                (arch_apts, arch_pips) = get_dependencies(dep_arch.large_dependencies);
            } else {
                (arch_apts, arch_pips) = get_dependencies(dep_arch.dependencies);
            }
            None::<String>
        }
        None => None,
    });

    return (arch_apts, arch_pips);
}

fn run_bash_command(command: &str) {
    if command == "" {
        return;
    }
    let mut child = Command::new("bash")
        .args(["-c", command])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();

    // Stream output.
    let lines = BufReader::new(stdout).lines();
    for line in lines {
        println!("{}", line.unwrap());
    }
}

fn main() -> Result<(), String> {
    let args = Cli::parse();

    if !args.file_name.ends_with(".toml") {
        return Err("file_name must end with .toml!".to_string());
    }

    let glob_paths = args.search_directory.join(args.file_name);

    println!("Searching the following path: {:?}", glob_paths);

    let mut apt_depends: Vec<String> = vec![];
    let mut pip_depends: Vec<String> = vec![];

    for entry in glob(glob_paths.to_str().unwrap()).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let absolute_path = fs::canonicalize(path).expect("Able to get absolute path");
                let package_name = absolute_path
                    .parent()
                    .expect("A parent directory is available")
                    .file_name()
                    .expect("Package name should not be none");

                let mut meta_depend: MetaDepend = match fs::read_to_string(absolute_path.clone()) {
                    // If successful return the files text as `contents`.
                    // `c` is a local variable.
                    Ok(c) => toml::from_str(&c).expect("Able to load toml"),
                    // Handle the `error` case.
                    Err(_) => {
                        // Write `msg` to `stderr`.
                        eprintln!("Could not read file `{:#?}`", absolute_path);
                        // Exit the program with exit code `1`.
                        process::exit(1);
                    }
                };
                if meta_depend.name == None {
                    meta_depend.name = Some(
                        package_name
                            .to_os_string()
                            .into_string()
                            .expect("Name as a string"),
                    );
                }
                println!("{:?} @ {:?}", meta_depend.name.unwrap(), absolute_path);

                if args.large_dep {
                    let (apt_list, pip_list) = get_dependencies(meta_depend.large_dependencies);
                    apt_depends.extend(apt_list);
                    pip_depends.extend(pip_list);
                } else {
                    let (apt_list, pip_list) = get_dependencies(meta_depend.dependencies);
                    apt_depends.extend(apt_list);
                    pip_depends.extend(pip_list);
                }

                let (arch_apt_list, arch_pip_list) =
                    get_arch_dependencies(meta_depend.target, args.large_dep);

                apt_depends.extend(arch_apt_list);
                pip_depends.extend(arch_pip_list);
            }
            Err(e) => println!("{:?}", e),
        }
    }

    apt_depends.sort();
    pip_depends.sort();
    apt_depends.dedup();
    pip_depends.dedup();

    println!("apt: {:#?}\npip: {:#?}\n", apt_depends, pip_depends);

    if args.apt_update {
        run_bash_command("sudo apt update");
    }

    if args.command == CliCommand::Install {
        let mut apt_install_cmd = "".to_string();
        let mut pip_install_cmd = "".to_string();

        if apt_depends.len() > 0 {
            apt_install_cmd = "sudo apt install -y ".to_string() + &apt_depends.join(" ");
        }
        if pip_depends.len() > 0 {
            pip_install_cmd = "pip install ".to_string() + &pip_depends.join(" ");
        }

        match args.dependency_type {
            DependencyType::Both => {
                run_bash_command(&apt_install_cmd);
                run_bash_command(&pip_install_cmd);
            }
            DependencyType::Apt => run_bash_command(&apt_install_cmd),
            DependencyType::Pip => run_bash_command(&pip_install_cmd),
        }
    }

    Ok(())
}
