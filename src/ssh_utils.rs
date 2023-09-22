use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;

use dirs::home_dir;
use ssh2::Session;
use ssh2_config::{HostParams, ParseRule, SshConfig};

use crate::nodes::Metadata;

pub fn get_ssh_config() -> SshConfig {
    let config_path = format!("{}/.ssh/config", home_dir().unwrap().to_str().unwrap());
    let config_file = File::open(config_path).unwrap();
    //.expect("Unable to open ~/.ssh/config");
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

pub fn read_remote_json_file(sess: &mut Session, file_name: &str) -> Metadata {
    let command = format!("cat {}", file_name);
    let output = run_remote_command(sess, &command);
    let metadata: Metadata = serde_json::from_str(&output).unwrap();
    return metadata;
}

pub fn send_to_remote(sess: &mut Session, local_file_path: &str, remote_file_path: &str) {
    sess.set_blocking(true);

    let mut local_file = File::open(local_file_path).unwrap();

    // get file metadata to retrieve the file size
    let metadata = local_file.metadata().unwrap();

    // write the file
    let mut remote_file = sess
        .scp_send(
            Path::new(remote_file_path),
            0o644,
            metadata.len(),
            None,
        )
        .unwrap();

    // create a buffer to write the contents of the local file to the remote file
    let mut buffer = Vec::new();
    local_file.read_to_end(&mut buffer).expect("Failed to read local file");
    remote_file.write_all(&buffer).expect("Failed to write to remote file");

    // close the channel and wait for the whole content to be transferred
    remote_file.send_eof().unwrap();
    remote_file.wait_eof().unwrap();
    remote_file.close().unwrap();
    remote_file.wait_close().unwrap();
}
