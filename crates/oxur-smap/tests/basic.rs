use oxur_smap::{new_node_id, SourceMap, SourcePos};

#[test]
fn test_repl_workflow() {
    // Simulate REPL: parse → expand → lower → error
    let mut map = SourceMap::new();

    // Parse phase
    let surface = new_node_id();
    map.record_surface_node(surface, SourcePos::repl(1, 1, 20));

    // Expansion phase
    let core = new_node_id();
    map.record_expansion(surface, core);

    // Lowering phase
    let rust = new_node_id();
    map.record_lowering(core, rust);

    // Error translation
    let pos = map.lookup(&rust).expect("Lookup failed");
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 1);
    assert_eq!(pos.length, 20);
    assert_eq!(pos.file, "<repl>");
}

#[test]
fn test_file_workflow() {
    // Simulate file compilation
    let mut map = SourceMap::new();

    // Parse phase
    let surface = new_node_id();
    map.record_surface_node(surface, SourcePos::new("test.ox".to_string(), 10, 5, 15));

    // Expansion phase
    let core = new_node_id();
    map.record_expansion(surface, core);

    // Lowering phase
    let rust = new_node_id();
    map.record_lowering(core, rust);

    // Error translation
    let pos = map.lookup(&rust).expect("Lookup failed");
    assert_eq!(pos.file, "test.ox");
    assert_eq!(pos.line, 10);
    assert_eq!(pos.column, 5);
    assert_eq!(pos.length, 15);
}

#[test]
fn test_multiple_expressions() {
    // Simulate compiling multiple expressions in one session
    let mut map = SourceMap::new();

    // First expression
    let surface1 = new_node_id();
    let core1 = new_node_id();
    let rust1 = new_node_id();
    map.record_surface_node(surface1, SourcePos::repl(1, 1, 10));
    map.record_expansion(surface1, core1);
    map.record_lowering(core1, rust1);

    // Second expression
    let surface2 = new_node_id();
    let core2 = new_node_id();
    let rust2 = new_node_id();
    map.record_surface_node(surface2, SourcePos::repl(2, 1, 15));
    map.record_expansion(surface2, core2);
    map.record_lowering(core2, rust2);

    // Verify both lookups work
    let pos1 = map.lookup(&rust1).unwrap();
    assert_eq!(pos1.line, 1);

    let pos2 = map.lookup(&rust2).unwrap();
    assert_eq!(pos2.line, 2);
}

#[test]
fn test_broken_chain_graceful_failure() {
    // Simulate incomplete transformation (e.g., compilation error during expansion)
    let mut map = SourceMap::new();

    let surface = new_node_id();
    map.record_surface_node(surface, SourcePos::repl(1, 1, 10));

    // Expansion never completes
    // Lowering phase tries to look up a rust node that was never created
    let rust = new_node_id();

    // Lookup should fail gracefully
    assert!(map.lookup(&rust).is_none());
}

#[test]
fn test_stats_tracking() {
    let mut map = SourceMap::new();

    // Track stats as we build the map
    let stats0 = map.stats();
    assert_eq!(stats0.surface_nodes, 0);
    assert_eq!(stats0.expansions, 0);
    assert_eq!(stats0.lowerings, 0);

    // Add surface node
    let surface = new_node_id();
    map.record_surface_node(surface, SourcePos::repl(1, 1, 10));
    let stats1 = map.stats();
    assert_eq!(stats1.surface_nodes, 1);

    // Add expansion
    let core = new_node_id();
    map.record_expansion(surface, core);
    let stats2 = map.stats();
    assert_eq!(stats2.expansions, 1);

    // Add lowering
    let rust = new_node_id();
    map.record_lowering(core, rust);
    let stats3 = map.stats();
    assert_eq!(stats3.lowerings, 1);
}
