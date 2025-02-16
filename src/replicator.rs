// TODO: Replication server + client implementation
// this should serve as a RAFT like kv where each node is considered
// a leader or a follower. https://arorashu.github.io/posts/raft.html

use std::time;

/// Node indicates the node type for the running server. Leader and
/// Follower are the two node types the ID for each is a [usize].
enum Node {
    Leader { id: usize },
    Follower { id: usize },
    Candidate { id: usize },
}

pub struct Replicator {
    nodes: Vec<Node>,
    replication_log: Vec<u8>,
    current_leader: usize,
    heartbeat: time::Duration,
}
