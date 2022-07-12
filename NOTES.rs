
// important ideas 4 hours before i need to wake up
/*
let each vertex in the CFG correspond to a node on the tree, no duplicates
will implement custom grapher without daggy i think
*/
pub struct FlowGraph {
    nodes: HashMap<(NodeId, &File), Path>
    edges: HashMap<(NodeId, &File), (NodeId, &File, ContextStack)>
    leaves: HashMap<Taint, (NodeId, File)> 
    // maybe?? or can figure out in analyzer what the leaves are
    // by storing parent taint nodeid/file to add to 
    // (this also means less diff between two systems)
}
