use oxur_smap::{new_node_id, NodeId, SourceMap, SourcePos};

#[test]
fn test_cache_key_generation() {
    let mut map = SourceMap::new();

    let surface = new_node_id();
    let core = new_node_id();
    let rust = new_node_id();

    map.record_surface_node(surface, SourcePos::repl(1, 1, 10));
    map.record_expansion(surface, core);
    map.record_lowering(core, rust);

    // Generate cache key
    let hash = map.content_hash();

    // Hash should be non-zero
    assert_ne!(hash, 0);
}

#[test]
fn test_cache_key_stability() {
    let mut map1 = SourceMap::new();
    let mut map2 = SourceMap::new();

    // Build identical maps
    let surface = NodeId::from_raw(100);
    let core = NodeId::from_raw(200);
    let rust = NodeId::from_raw(300);

    map1.record_surface_node(surface, SourcePos::repl(1, 1, 10));
    map1.record_expansion(surface, core);
    map1.record_lowering(core, rust);

    map2.record_surface_node(surface, SourcePos::repl(1, 1, 10));
    map2.record_expansion(surface, core);
    map2.record_lowering(core, rust);

    // Hashes should be identical
    assert_eq!(map1.content_hash(), map2.content_hash());
}

#[test]
fn test_cache_key_position_independence() {
    let mut map1 = SourceMap::new();
    let mut map2 = SourceMap::new();

    let surface = NodeId::from_raw(100);
    let core = NodeId::from_raw(200);

    // Same structure, different source positions
    map1.record_surface_node(surface, SourcePos::repl(1, 1, 10));
    map1.record_expansion(surface, core);

    map2.record_surface_node(surface, SourcePos::repl(100, 50, 10));
    map2.record_expansion(surface, core);

    // Hashes should be identical (positions don't affect cache key)
    assert_eq!(map1.content_hash(), map2.content_hash());
}

#[test]
fn test_cache_key_structure_sensitivity() {
    let mut map1 = SourceMap::new();
    let mut map2 = SourceMap::new();

    let surface = NodeId::from_raw(100);
    let core1 = NodeId::from_raw(200);
    let core2 = NodeId::from_raw(201);

    map1.record_expansion(surface, core1);
    map2.record_expansion(surface, core2);

    // Different structure → different hash
    assert_ne!(map1.content_hash(), map2.content_hash());
}

#[test]
fn test_end_to_end_compilation_simulation() {
    // Simulate a complete compilation with multiple nodes
    let mut map = SourceMap::new();

    // Parse: 3 surface forms
    let surface1 = new_node_id();
    let surface2 = new_node_id();
    let surface3 = new_node_id();

    map.record_surface_node(surface1, SourcePos::repl(1, 1, 15));
    map.record_surface_node(surface2, SourcePos::repl(2, 1, 20));
    map.record_surface_node(surface3, SourcePos::repl(3, 1, 10));

    // Expand: 3 core forms
    let core1 = new_node_id();
    let core2 = new_node_id();
    let core3 = new_node_id();

    map.record_expansion(surface1, core1);
    map.record_expansion(surface2, core2);
    map.record_expansion(surface3, core3);

    // Lower: 3 rust nodes
    let rust1 = new_node_id();
    let rust2 = new_node_id();
    let rust3 = new_node_id();

    map.record_lowering(core1, rust1);
    map.record_lowering(core2, rust2);
    map.record_lowering(core3, rust3);

    // Verify all lookups work
    assert_eq!(map.lookup(&rust1).unwrap().line, 1);
    assert_eq!(map.lookup(&rust2).unwrap().line, 2);
    assert_eq!(map.lookup(&rust3).unwrap().line, 3);

    // Verify stats
    let stats = map.stats();
    assert_eq!(stats.surface_nodes, 3);
    assert_eq!(stats.expansions, 3);
    assert_eq!(stats.lowerings, 3);

    // Verify cache key is generated
    assert_ne!(map.content_hash(), 0);
}

#[test]
fn test_complex_nested_structure() {
    // Simulate nested structure (e.g., function with nested expressions)
    let mut map = SourceMap::new();

    // Function definition (surface)
    let fn_surface = new_node_id();
    map.record_surface_node(fn_surface, SourcePos::repl(1, 1, 50));

    // Function body expression 1
    let expr1_surface = new_node_id();
    map.record_surface_node(expr1_surface, SourcePos::repl(2, 3, 10));

    // Function body expression 2
    let expr2_surface = new_node_id();
    map.record_surface_node(expr2_surface, SourcePos::repl(3, 3, 15));

    // Expansion phase
    let fn_core = new_node_id();
    let expr1_core = new_node_id();
    let expr2_core = new_node_id();

    map.record_expansion(fn_surface, fn_core);
    map.record_expansion(expr1_surface, expr1_core);
    map.record_expansion(expr2_surface, expr2_core);

    // Lowering phase
    let fn_rust = new_node_id();
    let expr1_rust = new_node_id();
    let expr2_rust = new_node_id();

    map.record_lowering(fn_core, fn_rust);
    map.record_lowering(expr1_core, expr1_rust);
    map.record_lowering(expr2_core, expr2_rust);

    // Verify error in nested expression maps correctly
    let pos = map.lookup(&expr2_rust).unwrap();
    assert_eq!(pos.line, 3);
    assert_eq!(pos.column, 3);
    assert_eq!(pos.length, 15);
}

#[test]
fn test_source_pos_display() {
    let pos = SourcePos::new("test.ox".to_string(), 10, 20, 5);
    assert_eq!(format!("{}", pos), "test.ox:10:20");
}

#[test]
fn test_node_id_display() {
    let id = NodeId::from_raw(42);
    assert_eq!(format!("{}", id), "NodeId(42)");
}
