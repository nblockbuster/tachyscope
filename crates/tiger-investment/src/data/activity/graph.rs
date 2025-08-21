use tiger_parse::tiger_tag;
use tiger_pkg::TagHash;

#[tiger_tag(id = 0x8080565C, size = 0x18)]
pub struct SActivityGraphMap {
    pub file_size: u64,
    pub activity_graphs: Vec<S80805660>,
}

#[tiger_tag(id = 0x80805660, size = 0x20)]
pub struct S80805660 {
    pub hash: u32,
    #[tag(offset = 0x10)]
    pub graph: TagHash,
}

#[derive(Clone)]
#[tiger_tag(id = 0x80805662, size = 0x98)]
pub struct SActivityGraph {
    pub file_size: u64,
    #[tag(offset = 0x50)]
    pub nodes: Vec<SActivityGraphNode>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x80805671, size = 0x114)]
pub struct SActivityGraphNode {
    pub node_id: u32,
    #[tag(offset = 0x8)]
    pub activities: Vec<()>, // only default hashes?
    pub featuring_states: Vec<()>,
    #[tag(offset = 0x8)]
    pub pos: [u16; 3],
}
