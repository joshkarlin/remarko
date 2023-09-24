use std::fs::{create_dir_all, File};
use std::io::{BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;

use colored::*;
use dirs::home_dir;
use ssh2::Session;
use ssh2_config::{HostParams, ParseRule, SshConfig};

use crate::constants;
use crate::nodes::{Directory, DirectoryNode, Metadata, Node};

pub fn get_ssh_config() -> SshConfig {
    let config_path = format!("{}/.ssh/config", home_dir().unwrap().to_str().unwrap());
    let config_file = File::open(&config_path)
        .expect(format!("Unable to open {}/.ssh/config", config_path).as_str());
    let mut reader = BufReader::new(config_file);
    let config = SshConfig::default()
        .parse(&mut reader, ParseRule::STRICT)
        .expect("Failed to parse configuration");
    return config;
}

pub fn connect_to_remote(params: HostParams) -> Session {
    let host_name = params.host_name.unwrap();
    let port = params.port.unwrap_or(22);
    let user = params.user.as_ref().unwrap();
    let identity_files = params.identity_file.unwrap();
    let identity_file = identity_files.first().unwrap();

    let tcp = TcpStream::connect((host_name, port)).expect("Failed to connect to remote");
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.userauth_pubkey_file(user, None, Path::new(&identity_file), None)
        .expect("authentication failed");
    return sess;
}

pub fn run_remote_command(sess: &mut Session, command: &str) -> String {
    let mut channel = sess.channel_session().unwrap();
    channel.exec(command).unwrap();
    let mut output = String::new();
    channel.read_to_string(&mut output).unwrap();
    return output;
}

pub fn read_remote_metadata(sess: &mut Session, file_name: &str) -> Metadata {
    let command = format!("cat {}", file_name);
    let output = run_remote_command(sess, &command);
    let metadata: Metadata = serde_json::from_str(&output).unwrap();
    return metadata;
}

pub fn check_remote_file_exists(sess: &mut Session, file_path: &str) -> bool {
    let command = format!("test -f {} && echo 'nice'", file_path);
    let output = run_remote_command(sess, &command);
    return output.trim() == "nice";
}

// TODO be consistent with whether sess is first or last argument
pub fn send_to_remote(local_file_path: &str, remote_file_path: &str, sess: &mut Session) {
    let mut local_file = File::open(local_file_path).unwrap();

    // get file metadata to retrieve the file size
    let metadata = local_file.metadata().unwrap();

    // write the file
    let mut remote_file = sess
        .scp_send(Path::new(remote_file_path), 0o644, metadata.len(), None)
        .unwrap();

    // create a buffer to write the contents of the local file to the remote file
    let mut buffer = Vec::new();
    local_file
        .read_to_end(&mut buffer)
        .expect("Failed to read local file");
    remote_file
        .write_all(&buffer)
        .expect("Failed to write to remote file");

    // close the channel and wait for the whole content to be transferred
    remote_file.send_eof().unwrap();
    remote_file.wait_eof().unwrap();
    remote_file.close().unwrap();
    remote_file.wait_close().unwrap();
}

/// Copies a file from the remote to the local file system.
///
/// # Arguments
///
/// * `remote_file_path` - The path to the remote file.
/// * `local_file_path` - The path to the local file.
/// * `sess` - The SSH session.
///
/// # Example
///
/// ```no_run
/// use remarko::ssh_utils::{connect_to_remote, copy_from_remote, get_ssh_config};
///
/// let ssh_config = get_ssh_config();
/// let params = ssh_config.query("remarkable");
/// let mut sess = connect_to_remote(params);
/// copy_from_remote("/home/root/test_file", "/home/user/test_file", &mut sess);
/// ```
///
/// # Panics
///
/// Panics if the remote file cannot be read or if the local file cannot be written.
///
/// # Remarks
///
/// This function is used to copy a file from the remote to the local file system.
/// The remote file is read into a buffer and then written to the local file.
pub fn copy_from_remote(remote_file_path: &str, local_file_path: &str, sess: &mut Session) {
    // read remote file to buffer
    let (mut remote_file, _) = sess.scp_recv(Path::new(remote_file_path)).unwrap();
    let mut buffer = Vec::new();
    remote_file.read_to_end(&mut buffer).unwrap();

    // close the channel and wait for the whole content to be transferred
    remote_file.send_eof().unwrap();
    remote_file.wait_eof().unwrap();
    remote_file.close().unwrap();
    remote_file.wait_close().unwrap();

    // write buffer to file
    let mut local_file = File::create(local_file_path).unwrap();
    local_file.write_all(&buffer).unwrap();
}

pub fn copy_directory_from_remote(
    directory: &Directory,
    local_path: &Path,
    sess: &mut Session,
) -> Result<(), Box<dyn std::error::Error>> {
    // ensure the directory exists locally
    if !local_path.exists() {
        create_dir_all(local_path)?;
    }

    // copy files from the directory
    for file in directory.get_files() {
        let pdf_file_name = format!("{}.pdf", file.get_hash());
        let remote_file_path = format!("{}/{}", constants::DIR, &pdf_file_name);

        // check remote file path exists
        if check_remote_file_exists(sess, &remote_file_path) == false {
            println!(
                "{} {} ({}) does not exist on remote",
                "Error:".bold().red(),
                &pdf_file_name.purple(),
                file.get_visible_name()
            );
            continue;
        }

        let local_file_path = local_path.join(file.get_visible_name());
        copy_from_remote(&remote_file_path, local_file_path.to_str().unwrap(), sess);
    }

    // recursively copy sub-directories
    for sub_directory in directory.get_directories() {
        let sub_local_path = local_path.join(sub_directory.get_visible_name());
        copy_directory_from_remote(sub_directory, &sub_local_path, sess)?;
    }

    Ok(())
}
