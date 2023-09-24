use std::env::current_dir;
use std::fs::create_dir_all;
use std::path::Path;

use clap::{arg, ArgAction, Command};
use colored::*;

use remarko::constants::DIR;
use remarko::local_fs::{build_local_directory, remove_common_files_and_directories};
use remarko::nodes::{DirectoryNode, Node};
use remarko::remarkable_trees::{build_tree, get_hashes_from_ls_output, print_tree};
use remarko::ssh_utils::{
    connect_to_remote, copy_directory_from_remote, get_ssh_config, run_remote_command,
    send_to_remote,
};

fn cli() -> Command {
    Command::new("remarko")
        .author("Josh Karlin")
        .about("A tool to interact with remarkable")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .arg(arg!(verbose: -v --verbose "Print verbose output")
            .global(true)
            .action(ArgAction::SetTrue))
        .subcommand(Command::new("list").about("Lists files on the remote filesystem"))
        .subcommand(
            Command::new("diff")
                .about("Compares the local filesystem to the remote filesystem")
                .arg(arg!(local_directory: <LOCAL_DIRECTORY> "The local directory to compare").required(false).default_value(""))
                .arg(arg!(remote_directory: -d --remote_directory <REMOTE_DIRECTORY>  "The remote directory to compare").required(false).default_value("")),
        )
        .subcommand(
            Command::new("pull")
                .about("Pull any files from the remote filesystem which are not in the destination directory")
                .arg(arg!(remote_directory: -d --directory <DIRECTORY> "The remote directory to pull from")
                    .required(false)
                    .default_value(""))
                .arg(arg!(destination: <DESTINATION> "The local directory to pull to"))
        )
        .subcommand(
            Command::new("push")
                .about("Push a file to the remote filesystem")
                .arg(arg!(<FILE> "The file to push")),
        )
}

fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("list", _)) => {
            let ssh_config = get_ssh_config();
            let params = ssh_config.query("remarkable");
            let host_name = params
                .host_name
                .as_ref()
                .expect("No HostName in ssh config")
                .to_string();

            println!(
                "\n{} {}\n",
                "Listing files on".bold().yellow(),
                host_name.bold().yellow()
            );

            let mut sess = connect_to_remote(params);
            let output = run_remote_command(&mut sess, format!("ls {}", DIR).as_str());
            let hashes = get_hashes_from_ls_output(&output);
            let (root_directory, trash_directory) = build_tree(hashes, &mut sess);
            print_tree(&root_directory, 0);
            println!();
            print_tree(&trash_directory, 0);
            println!();
        }
        Some(("diff", sub_matches)) => {
            let local_directory_path_input = sub_matches
                .get_one::<String>("local_directory")
                .expect("required");
            let remote_directory_path = sub_matches
                .get_one::<String>("remote_directory")
                .expect("required");

            let current_dir = current_dir().unwrap().to_str().unwrap().to_string();
            let local_directory_path_input =
                format!("{}/{}", current_dir, local_directory_path_input);
            let local_directory_path = Path::new(&local_directory_path_input).to_path_buf();

            // TODO pull out this repeated code
            let ssh_config = get_ssh_config();
            let params = ssh_config.query("remarkable");
            let host_name = params
                .host_name
                .as_ref()
                .expect("No HostName in ssh config")
                .to_string();

            let mut sess = connect_to_remote(params);
            let output = run_remote_command(&mut sess, format!("ls {}", DIR).as_str());
            let hashes = get_hashes_from_ls_output(&output);
            let (remote_root_directory, _) = build_tree(hashes, &mut sess);
            let local_directory = build_local_directory(&local_directory_path).unwrap();

            // get the sub-directory on remote if specified
            let mut remote_directory = remote_root_directory.clone();
            if remote_directory_path == "" {
                println!(
                    "\n{} {} {} {}",
                    "Comparing all files on".bold().yellow(),
                    host_name.bold().yellow(),
                    "with".bold().yellow(),
                    local_directory_path.to_str().unwrap().bold().yellow(),
                );
            } else {
                println!(
                    "\n{} {} {} {} {} {}",
                    "Comparing files from".bold().yellow(),
                    remote_directory_path.bold().yellow(),
                    "on".bold().yellow(),
                    host_name.bold().yellow(),
                    "to".bold().yellow(),
                    local_directory_path.to_str().unwrap().bold().yellow(),
                );
                for dir in remote_directory_path.split("/") {
                    remote_directory = remote_directory
                        .get_directories()
                        .iter()
                        .find(|d| d.get_visible_name() == dir)
                        .expect(&format!(
                            "{} {} {}\n",
                            "Error: Directory".bold().red(),
                            dir.to_string().bold().red(),
                            "not found in remote directory".bold().red(),
                        ))
                        .clone();
                }
            }

            let (unique_on_remote, unique_on_local) =
                remove_common_files_and_directories(&remote_directory, &local_directory);

            println!("\nUnique on remote:");
            print_tree(&unique_on_remote, 0);
            println!("\nUnique on local:");
            print_tree(&unique_on_local, 0);
        }
        Some(("push", sub_matches)) => {
            let local_file_path = sub_matches.get_one::<String>("FILE").expect("required");

            // TODO pull out this repeated code
            let ssh_config = get_ssh_config();
            let params = ssh_config.query("remarkable");
            let host_name = params
                .host_name
                .as_ref()
                .expect("No HostName in ssh config")
                .to_string();

            println!(
                "\n{} {}\n",
                "Pushing test file to".bold().yellow(),
                host_name.bold().yellow()
            );

            let mut sess = connect_to_remote(params);
            send_to_remote(local_file_path, "/home/root/test_file", &mut sess);
        }
        Some(("pull", sub_matches)) => {
            let remote_directory_path = sub_matches
                .get_one::<String>("remote_directory")
                .expect("required");
            let local_directory_path = sub_matches
                .get_one::<String>("destination")
                .expect("required");
            let verbose = sub_matches.get_flag("verbose");

            let ssh_config = get_ssh_config();
            let params = ssh_config.query("remarkable");
            let host_name = params
                .host_name
                .as_ref()
                .expect("No HostName in ssh config")
                .to_string();

            let mut sess = connect_to_remote(params);
            let output = run_remote_command(&mut sess, format!("ls {}", DIR).as_str());
            let hashes = get_hashes_from_ls_output(&output);
            let (remote_root_directory, _) = build_tree(hashes, &mut sess);

            // get the sub-directory on remote if specified
            let mut remote_directory = remote_root_directory.clone();
            if remote_directory_path == "" {
                println!(
                    "\n{} {} {} {}",
                    "Pulling all files from".bold().yellow(),
                    host_name.bold().yellow(),
                    "to".bold().yellow(),
                    local_directory_path.bold().yellow(),
                );
            } else {
                println!(
                    "\n{} {} {} {} {} {}",
                    "Pulling files from".bold().yellow(),
                    remote_directory_path.bold().yellow(),
                    "on".bold().yellow(),
                    host_name.bold().yellow(),
                    "to".bold().yellow(),
                    local_directory_path.bold().yellow(),
                );
                for dir in remote_directory_path.split("/") {
                    remote_directory = remote_directory
                        .get_directories()
                        .iter()
                        .find(|d| d.get_visible_name() == dir)
                        .expect(&format!(
                            "{} {} {}\n",
                            "Error: Directory".bold().red(),
                            dir.to_string().bold().red(),
                            "not found in remote directory".bold().red(),
                        ))
                        .clone();
                }
            }

            let local_directory_path_ = Path::new(&local_directory_path);
            if local_directory_path_.exists() == false {
                create_dir_all(local_directory_path_).unwrap();
                println!(
                    "\n{} {} {}",
                    "Success:".bold().green(),
                    "created local directory",
                    local_directory_path_.to_str().unwrap().italic().purple(),
                );
            }

            let local_directory = build_local_directory(local_directory_path_).unwrap();

            let (unique_on_remote, _) =
                remove_common_files_and_directories(&remote_directory, &local_directory);

            if verbose {
                println!();
                print_tree(&unique_on_remote, 0);
                println!();
            }

            // copy unique_on_remote to local
            copy_directory_from_remote(
                &unique_on_remote,
                Path::new(&local_directory_path),
                &mut sess,
            )
            .unwrap();
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
    };
}
