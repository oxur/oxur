use oxur_smap::{new_node_id, NodeId, SourceMap, SourcePos};

#[test]
fn test_lookup_complete_chain() {
    let mut map = SourceMap::new();

    let surface = new_node_id();
    let core = new_node_id();
    let rust = new_node_id();

    map.record_surface_node(surface, SourcePos::repl(5, 10, 20));
    map.record_expansion(surface, core);
    map.record_lowering(core, rust);

    let pos = map.lookup(&rust).unwrap();
    assert_eq!(pos.line, 5);
    assert_eq!(pos.column, 10);
    assert_eq!(pos.length, 20);
}

#[test]
fn test_lookup_missing_surface() {
    let mut map = SourceMap::new();

    let surface = new_node_id();
    let core = new_node_id();
    let rust = new_node_id();

    // Missing: record_surface_node
    map.record_expansion(surface, core);
    map.record_lowering(core, rust);

    // Should fail because no surface position exists
    assert!(map.lookup(&rust).is_none());
}

#[test]
fn test_lookup_missing_expansion() {
    let mut map = SourceMap::new();

    let surface = new_node_id();
    let rust = new_node_id();

    map.record_surface_node(surface, SourcePos::repl(1, 1, 10));
    // Missing: record_expansion
    // Note: We can't record lowering without core node

    // Should fail because no expansion chain exists
    assert!(map.lookup(&rust).is_none());
}

#[test]
fn test_lookup_missing_lowering() {
    let mut map = SourceMap::new();

    let surface = new_node_id();
    let core = new_node_id();
    let rust = new_node_id();

    map.record_surface_node(surface, SourcePos::repl(1, 1, 10));
    map.record_expansion(surface, core);
    // Missing: record_lowering

    // Should fail because no lowering to this rust node exists
    assert!(map.lookup(&rust).is_none());
}

#[test]
fn test_lookup_wrong_rust_node() {
    let mut map = SourceMap::new();

    let surface = new_node_id();
    let core = new_node_id();
    let rust1 = new_node_id();
    let rust2 = new_node_id();

    map.record_surface_node(surface, SourcePos::repl(1, 1, 10));
    map.record_expansion(surface, core);
    map.record_lowering(core, rust1);

    // Try to lookup a different rust node
    assert!(map.lookup(&rust2).is_none());
}

#[test]
fn test_lookup_multiple_chains_isolated() {
    let mut map = SourceMap::new();

    // Chain 1
    let surface1 = new_node_id();
    let core1 = new_node_id();
    let rust1 = new_node_id();
    map.record_surface_node(surface1, SourcePos::new("file1.ox".to_string(), 1, 1, 5));
    map.record_expansion(surface1, core1);
    map.record_lowering(core1, rust1);

    // Chain 2
    let surface2 = new_node_id();
    let core2 = new_node_id();
    let rust2 = new_node_id();
    map.record_surface_node(surface2, SourcePos::new("file2.ox".to_string(), 10, 20, 15));
    map.record_expansion(surface2, core2);
    map.record_lowering(core2, rust2);

    // Verify lookups are isolated
    let pos1 = map.lookup(&rust1).unwrap();
    assert_eq!(pos1.file, "file1.ox");
    assert_eq!(pos1.line, 1);

    let pos2 = map.lookup(&rust2).unwrap();
    assert_eq!(pos2.file, "file2.ox");
    assert_eq!(pos2.line, 10);
}

#[test]
fn test_direct_accessor_methods() {
    let mut map = SourceMap::new();

    let surface = new_node_id();
    let core = new_node_id();
    let rust = new_node_id();
    let pos = SourcePos::repl(1, 1, 10);

    map.record_surface_node(surface, pos.clone());
    map.record_expansion(surface, core);
    map.record_lowering(core, rust);

    // Test direct accessors
    assert_eq!(map.get_surface_position(&surface).unwrap().line, 1);
    assert_eq!(map.get_core_from_surface(&surface), Some(&core));
    assert_eq!(map.get_rust_from_core(&core), Some(&rust));
}

#[test]
fn test_lookup_with_raw_node_ids() {
    let mut map = SourceMap::new();

    // Use raw IDs to test specific ID values
    let surface = NodeId::from_raw(100);
    let core = NodeId::from_raw(200);
    let rust = NodeId::from_raw(300);

    map.record_surface_node(surface, SourcePos::repl(1, 5, 10));
    map.record_expansion(surface, core);
    map.record_lowering(core, rust);

    let pos = map.lookup(&rust).unwrap();
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 5);
}
