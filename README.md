# Remarko

Another "my first rust project" project. WIP and unreliable until further notice.

A rust CLI for doing some file syncing with remarkable.

Table of contents:

- [Prerequisites](#prerequisites)
- [CLI usage](#cli-usage)
- [TODO](#todo)

## Prerequisites

1. set up ssh access to your remarkable using an rsa key
1. an ssh profile in $HOME/.ssh/config called `remarkable` which uses the rsa key

### How to setup ssh access and `remarkable` ssh profile

First, confirm you can access your remarkable over ssh.
Connect your remarkable to the same wifi network you are running this from.

Menu -> Settings -> Help -> Copyrights and licenses

At the bottom of the page, you will find the ip address and password for ssh access.
Use these to ssh into the device. You will be prompted to enter the password.

```bash
ssh root@<ip-address>
```

Assuming this succeeded, the next step is to create a key pair.
(You could also use an existing one if you are the type to throw caution to the wind.)

To generate an rsa key pair:

```bash
ssh-keygen -b 4096 -t rsa -f ~/.ssh/remarkable
```

**Note**: You can choose an empty passphrase to avoid having to enter it every time.

Now create the entry in ~/.ssh/config by adding the following with the correct ip address:

```
Host remarkable
    HostName <ip-address>
    User root
    HostKeyAlgorithms +ssh-rsa
    PubkeyAcceptedKeyTypes +ssh-rsa
```

Copy the public key across to your remarkable.

Replace the ip address in the command below:

```bash
scp ~/.ssh/remarkable.pub remarkable:/home/root/.ssh/authorized_keys
```

This will be the last time you need to enter the password.

Update the `remarkable` profile in ~/.ssh/config to include `IdentityFile`:

```
Host remarkable
    ...
    IdentityFile ~/.ssh/remarkable
```

Final check:

```bash
ssh remarkable
```

Nice.

## CLI usage

To see the CLI help, run:

```bash
cargo run
```

To list the files on your remarkble:

```bash
cargo run list
```

To show the diff between a local directory and your remarkable:

```bash
cargo run diff <local-directory>
```

## TODO

- [x] Add a `list` command to list files on the remarkable
- [x] Add a `diff` command to compare files on the remarkable with local files
- [ ] Add a `pull` command to pull missing files from the remarkable to a directory on the local machine
- [ ] Extend `pull` with a flag to blindly pull everything
- [ ] Add a `push` command to push missing files from a directory on the local machine to the remarkable
- [ ] Combine files with annotations into a single pdf and pull to local machine
