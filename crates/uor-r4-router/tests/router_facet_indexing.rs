use std::collections::HashMap;
use uor_r4_router::geometry::FacetCoordinates;
use uor_r4_router::UorR4Router;

#[test]
fn test_router_facet_indexing() {
    let mut router = UorR4Router::new(0.5);

    let mut coords = FacetCoordinates {
        coordinates: HashMap::new(),
    };
    coords.coordinates.insert("type".to_string(), vec![1, 2, 3]);
    coords
        .coordinates
        .insert("entity".to_string(), vec![10, 20]);
    coords.coordinates.insert("relation".to_string(), vec![99]);

    router.index_semantic_object(101, &coords);
    router.index_semantic_object(102, &coords);

    // Verify indexing occurred correctly in MultiFacetStore
    let type_postings = router.facet_store.type_index.get(&vec![1, 2, 3]).unwrap();
    assert_eq!(type_postings, &vec![101, 102]);

    let entity_postings = router.facet_store.entity_index.get(&vec![10, 20]).unwrap();
    assert_eq!(entity_postings, &vec![101, 102]);

    let relation_postings = router.facet_store.relation_index.get(&vec![99]).unwrap();
    assert_eq!(relation_postings, &vec![101, 102]);
}
