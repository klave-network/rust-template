pub use block::BlockCommand;
use clap::Parser;
pub use header::HeaderCommand;
pub use init::InitCommand;
pub use update::UpdateCommand;
pub use persist::PersistCommand;

mod block;
mod header;
mod init;
mod update;
mod persist;

#[derive(Parser, Debug)]
pub enum Command {
    #[clap(about = "Initialize light client")]
    Init(InitCommand),
    #[clap(about = "Persist light client")]
    Persist(PersistCommand),
    #[clap(about = "Update light client")]
    Update(UpdateCommand),
    #[clap(about = "Fetch specific header")]
    Header(HeaderCommand),
    #[clap(about = "Fetch specific block")]
    Block(BlockCommand),
}
