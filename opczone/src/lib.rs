pub mod brand;
pub mod build;
pub mod dladm;
pub mod image;
pub mod machine;
mod util;
pub mod vmext;

use anyhow::{bail, Context, Result};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};

pub use util::*;

use common::*;

pub fn get_zone(zonename: &str) -> Result<zone::Zone> {
    let zones = zone::Adm::list()?;
    for zone in zones {
        if zone.name() == zonename {
            return Ok(zone);
        }
    }
    bail!("zone {} does not exist", zonename)
}

fn spawn_reader<T>(name: &str, stream: Option<T>) -> Option<std::thread::JoinHandle<()>>
where
    T: Read + Send + 'static,
{
    let name = name.to_string();
    let stream = match stream {
        Some(stream) => stream,
        None => return None,
    };

    Some(std::thread::spawn(move || {
        let mut r = BufReader::new(stream);

        loop {
            let mut buf = String::new();

            match r.read_line(&mut buf) {
                Ok(0) => {
                    /*
                     * EOF.
                     */
                    return;
                }
                Ok(_) => {
                    let s = buf.trim();

                    if !s.is_empty() {
                        info!(target: "illumos-rs", "{}| {}", name, s);
                    }
                }
                Err(e) => {
                    error!(target: "illumos-rs", "failed to read {}: {}", name, e);
                    std::process::exit(100);
                }
            }
        }
    }))
}

fn build_env<S: AsRef<str>>(env: Option<&[(S, S)]>) -> Option<Vec<(&str, &str)>> {
    if let Some(env) = env {
        let env: Vec<(&str, &str)> = env.iter().map(|(k, v)| (k.as_ref(), v.as_ref())).collect();
        Some(env)
    } else {
        None
    }
}

fn build_cmd(args: Vec<&str>, env: Option<Vec<(&str, &str)>>) -> Command {
    let mut cmd = Command::new(&args[0]);
    cmd.env_remove("LANG");
    cmd.env_remove("LC_CTYPE");
    cmd.env_remove("LC_NUMERIC");
    cmd.env_remove("LC_TIME");
    cmd.env_remove("LC_COLLATE");
    cmd.env_remove("LC_MONETARY");
    cmd.env_remove("LC_MESSAGES");
    cmd.env_remove("LC_ALL");

    if args.len() > 1 {
        cmd.args(&args[1..]);
    }

    if let Some(env) = env {
        cmd.envs(env.clone());
        debug!(target: "opczone", "exec: {:?} env={:?}", &args, &env);
    } else {
        debug!(target: "opczone", "exec: {:?}", &args);
    }
    cmd
}

pub fn run_with_stdin<S: AsRef<str>>(
    args: &[S],
    env: Option<&[(S, S)]>,
    stdin: String,
) -> Result<()> {
    let args: Vec<&str> = args.iter().map(|s| s.as_ref()).collect();
    let env = build_env(env);
    let mut cmd = build_cmd(args.clone(), env);

    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().context(format!(
        "could not spawn process {}",
        args[0].clone().to_owned()
    ))?;
    let mut child_stdin = child.stdin.take().unwrap();
    std::thread::spawn(move || {
        child_stdin.write_all(stdin.as_bytes()).unwrap();
    });

    let readout = spawn_reader("O", child.stdout.take());
    let readerr = spawn_reader("E", child.stderr.take());

    if let Some(t) = readout {
        t.join().expect("join stdout thread");
    }
    if let Some(t) = readerr {
        t.join().expect("join stderr thread");
    }

    match child.wait() {
        Err(e) => Err(e.into()),
        Ok(es) => {
            if !es.success() {
                bail!("exec {:?}: failed {:?}", &args, &es)
            } else {
                Ok(())
            }
        }
    }
}

pub fn run<S: AsRef<str>>(args: &[S], env: Option<&[(S, S)]>) -> Result<()> {
    let args: Vec<&str> = args.iter().map(|s| s.as_ref()).collect();
    let env = build_env(env);
    let mut cmd = build_cmd(args.clone(), env);

    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .context(format!("could not spawn process {}", args[0]))?;

    let readout = spawn_reader("O", child.stdout.take());
    let readerr = spawn_reader("E", child.stderr.take());

    if let Some(t) = readout {
        t.join().expect("join stdout thread");
    }
    if let Some(t) = readerr {
        t.join().expect("join stderr thread");
    }

    match child.wait() {
        Err(e) => Err(e.into()),
        Ok(es) => {
            if !es.success() {
                bail!("exec {:?}: failed {:?}", &args, &es)
            } else {
                Ok(())
            }
        }
    }
}

pub fn run_capture_stdout<S: AsRef<str>>(args: &[S], env: Option<&[(S, S)]>) -> Result<String> {
    let args: Vec<&str> = args.iter().map(|s| s.as_ref()).collect();
    let env = build_env(env);
    let mut cmd = build_cmd(args.clone(), env);

    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd
        .output()
        .context(format!("could not spawn process {}", args[0]))?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        bail!(
            "exec {:?}: failed {:?}",
            &args,
            String::from_utf8(output.stderr)?
        )
    }
}
