#[macro_use]
extern crate error_chain;
extern crate replicante_agent_mongodb;


use replicante_agent_mongodb::run;
quick_main!(run);
