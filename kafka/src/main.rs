#[macro_use]
extern crate error_chain;
extern crate replicante_agent_kafka;


use replicante_agent_kafka::run;
quick_main!(run);
