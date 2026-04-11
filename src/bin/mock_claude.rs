//! Fake `claude` binary used exclusively by dlaunch's e2e test suite.
//!
//! Prints its arguments to stdout so tests can assert what flags dlaunch
//! forwarded, then exits 0.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    println!("mock-claude: {}", args.join(" "));
    std::process::exit(0);
}
