extern crate replicante_agent;
extern crate replicante_agent_kafka;
extern crate replicante_util_failure;

use replicante_agent_kafka::run;

fn main() {
    ::replicante_agent::process::main(run);
}
