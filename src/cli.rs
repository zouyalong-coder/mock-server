use clap::Parser;

#[derive(Parser, Clone, Debug)]
#[clap(version, author, about)]
pub struct Cli {
    /// workspace. default is current directory.
    #[clap(short = 'W', long, default_value = ".")]
    pub work_dir: String,
    /// Whether to upload the file to the server.
    #[clap(short='U', long, default_value_t = false, action=clap::ArgAction::SetTrue)]
    pub up_file: bool,
    /// port.
    #[clap(short = 'P', long, default_value = "8080")]
    pub port: u16,
    /// verbose mode. Show all information from requests.
    #[clap(short, long, action=clap::ArgAction::SetTrue, default_value="false")]
    pub verbose: bool,
    /// Prefix of static files.
    #[clap(short = 'S', long, default_value = "static")]
    pub static_prefix: String,
}
