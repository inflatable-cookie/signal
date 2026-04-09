mod broker;

use std::io;

use broker::SandboxBrokerProcess;

fn main() {
    let stdin = io::stdin();
    let mut broker = SandboxBrokerProcess::default();
    broker
        .serve(stdin.lock(), io::stdout().lock())
        .expect("sandbox broker serve");
}
