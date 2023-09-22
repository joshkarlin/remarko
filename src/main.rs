use clap::{arg, Command};
use colored::*;
use dirs::home_dir;

use remarko::constants::DIR;
use remarko::nodes::{DirectoryNode, Node};
use remarko::remarkable_trees::{build_tree, get_hashes_from_ls_output, print_tree};
use remarko::ssh_utils::{connect_to_remote, get_ssh_config, run_remote_command, send_to_remote};

fn cli() -> Command {
    Command::new("remarko")
        .author("Josh Karlin")
        .about("A tool to interact with remarkable")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("list").about("Lists files on the remote filesystem"),
        )
        .subcommand(Command::new("backup").about("Backs up the remote filesystem"))
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
            send_to_remote(&mut sess, local_file_path, "/home/root/test_file");
        }
        Some(("backup", _))  => {
            let ssh_config = get_ssh_config();
            let params = ssh_config.query("remarkable");
            let host_name = params
                .host_name
                .as_ref()
                .expect("No HostName in ssh config")
                .to_string();

            println!(
                "\n{} {}\n",
                "Backing up".bold().yellow(),
                host_name.bold().yellow()
            );

            let mut sess = connect_to_remote(params);
            let output = run_remote_command(&mut sess, format!("ls {}", DIR).as_str());
            let hashes = get_hashes_from_ls_output(&output);
            let (root_directory, _) = build_tree(hashes, &mut sess);

            let a_file = root_directory.get_files().last().unwrap().clone();
            let a_file_hash = a_file.get_hash().to_string();
            let a_file_name = a_file.get_visible_name().to_string();
            let split_file_name = a_file_name.split(".").collect::<Vec<&str>>();

            let mut extension = "".to_string();
            if split_file_name.len() > 1 {
                extension = format!(".{}", split_file_name.last().unwrap());
                println!("extension: {:?}", extension);
            }
            
            let a_file_source = format!("{}/{}{}", DIR, a_file_hash, extension);
            let a_file_destination = format!("{}/{}", home_dir().unwrap().to_str().unwrap(), a_file_name);
            println!("Would copy {} to {}", a_file_source, a_file_destination);


        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
    };
}
