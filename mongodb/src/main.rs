extern crate replicante_agent;
extern crate replicante_agent_mongodb;
extern crate replicante_util_failure;

use replicante_agent_mongodb::run;

fn main() {
    ::replicante_agent::process::main(run);
}
