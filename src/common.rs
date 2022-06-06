use std::{
    io::{self, BufRead, BufReader, Write},
    path::Path,
    process::{Command, Output, Stdio},
};

pub fn check_output(output: Output) {
    println!("status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
    assert!(output.status.success());
}

pub fn exec_stream<P: AsRef<Path>>(binary: P, args: &[&str]) {
    let mut cmd = Command::new(binary.as_ref())
        .args(args)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    {
        let stdout = cmd.stdout.as_mut().unwrap();
        let stdout_reader = BufReader::new(stdout);
        let stdout_lines = stdout_reader.lines();

        for line in stdout_lines {
            //println!("Read: {:?}", line);
            println!("{}", line.unwrap());
        }
    }

    let res = cmd.wait().expect("failed to execute process");
    assert!(res.success());
}
