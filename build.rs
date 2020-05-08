use protoc_rust::Customize;

fn main () {
    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/raft/raftpb",
        input: &["src/raft/raftpb/raft.proto"],
        includes: &["src/raft/raftpb"],
        customize: Customize{
            carllerche_bytes_for_bytes: Some(true),
            carllerche_bytes_for_string: Some(true),
            ..Default::default()
        },
    }).expect("protoc");
}
