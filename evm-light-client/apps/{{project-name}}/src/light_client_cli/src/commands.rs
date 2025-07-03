pub use block::BlockCommand;
use clap::Parser;
pub use header::HeaderCommand;
pub use init::InitCommand;
pub use persist::PersistCommand;
pub use update::UpdateCommand;

mod block;
mod header;
mod init;
mod persist;
mod update;

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
