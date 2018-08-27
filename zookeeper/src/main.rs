#[macro_use]
extern crate error_chain;
extern crate replicante_agent_zookeeper;


use replicante_agent_zookeeper::run;
quick_main!(run);
