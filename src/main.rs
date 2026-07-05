use clap::Parser;

fn main() {
    let cli = agent_it_tools::Cli::parse();

    if let Err(err) = agent_it_tools::run(cli) {
        eprintln!("{}", serde_json::json!({ "error": err.to_string() }));
        std::process::exit(1);
    }
}
