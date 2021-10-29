extern crate clap;

use clap::{App, Arg};

struct ExecTest {
    name: String,
}

impl ExecTest {
    // fn new() -> Self {
    //     let app = App::new("test-executable")
    //         .version("1.0")
    //         .about("Tests Command Line Arguements")
    //         .author("Nobody");
    //     let name = matches.value_of("name").expect("Was Required");
    // }
}

fn main() {
    //println!("Main");
    let app = App::new("test-exec")
        .version("1.0")
        .about("Tests Command Line Arguements")
        .author("Nobody")
        .arg(Arg::with_name("Arg1").index(1).help("Checking Executable"))
        .get_matches();

    // Checking for executable file
    // Unsure of how file is sent
    // let permissions = file.metadata()?.permissions();
    // let is_executable = permissions.mode() & 0o111 != 0;
}

#[cfg(test)]
mod test {

    #[test]
    fn test_no_args() {
        // HelloArgs::new_from(["exename"].iter()).unwrap_err();
    }

    #[test]
    fn test_incomplete_name() {
        // HelloArgs::new_from(["exename", "--name"].iter()).unwrap_err();
    }
}
