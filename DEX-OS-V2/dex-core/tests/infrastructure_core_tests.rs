// Comprehensive Tests for Infrastructure Core Features
// Tests Blockchain Consensus, Virtual DOM, and State Reducer Pattern
// Security: Layer 22 (Blockchain) and Layer 10 (Frontend)

use dex_core::blockchain_consensus::*;
use dex_core::virtual_dom::*;
use dex_core::state_reducer::*;
use std::sync::Arc;
use parking_lot::RwLock;

// ============================================================================
// Blockchain Consensus Tests
// ============================================================================

#[test]
fn test_blockchain_transaction_creation_and_signing() {
    let mut tx = Transaction::new("alice".to_string(), "bob".to_string(), 1000, 0);
    assert_eq!(tx.from, "alice");
    assert_eq!(tx.to, "bob");
    assert_eq!(tx.amount, 1000);
    assert_eq!(tx.nonce, 0);

    let private_key = b"alice_secret_key";
    let public_key = b"alice_secret_key"; // Simplified for testing

    tx.sign(private_key);
    assert!(!tx.signature.is_empty());
    assert!(tx.verify_signature(public_key));

    // Verify with wrong key fails
    assert!(!tx.verify_signature(b"wrong_key"));
}

#[test]
fn test_blockchain_block_creation_and_merkle_root() {
    let tx1 = Transaction::new("alice".to_string(), "bob".to_string(), 100, 0);
    let tx2 = Transaction::new("bob".to_string(), "charlie".to_string(), 50, 0);
    let tx3 = Transaction::new("charlie".to_string(), "alice".to_string(), 25, 0);

    let block = Block::new(0, vec![], vec![tx1, tx2, tx3], "validator1".to_string());

    assert_eq!(block.height, 0);
    assert_eq!(block.transactions.len(), 3);
    assert!(!block.merkle_root.is_empty());
    assert_eq!(block.merkle_root.len(), 32); // SHA3-256 produces 32 bytes
}

#[test]
fn test_blockchain_consensus_validator_management() {
    let consensus = BlockchainConsensus::new(3, 0.67);

    let val1 = Validator::new("val1".to_string(), vec![1, 2, 3], 1000);
    let val2 = Validator::new("val2".to_string(), vec![4, 5, 6], 2000);
    let val3 = Validator::new("val3".to_string(), vec![7, 8, 9], 1500);

    // Add validators
    assert!(consensus.add_validator(val1.clone()).is_ok());
    assert!(consensus.add_validator(val2).is_ok());
    assert!(consensus.add_validator(val3).is_ok());

    // Duplicate validator should fail
    assert!(consensus.add_validator(val1).is_err());

    let stats = consensus.get_stats();
    assert_eq!(stats.total_validators, 3);
    assert_eq!(stats.active_validators, 3);

    // Remove validator
    assert!(consensus.remove_validator("val1").is_ok());
    assert_eq!(consensus.get_stats().total_validators, 2);

    // Remove non-existent validator should fail
    assert!(consensus.remove_validator("nonexistent").is_err());
}

#[test]
fn test_blockchain_transaction_validation() {
    let consensus = BlockchainConsensus::new(1, 0.51);

    let mut tx = Transaction::new("alice".to_string(), "bob".to_string(), 100, 0);
    tx.sign(b"alice_key");

    let result = consensus.validate_transaction(&tx);
    assert!(result.is_valid());

    // Invalid transaction (empty sender)
    let invalid_tx = Transaction {
        id: "test".to_string(),
        from: "".to_string(),
        to: "bob".to_string(),
        amount: 100,
        nonce: 0,
        timestamp: 0,
        signature: vec![1, 2, 3],
        data: vec![],
    };

    let result = consensus.validate_transaction(&invalid_tx);
    assert!(!result.is_valid());
}

#[test]
fn test_blockchain_mempool_operations() {
    let consensus = BlockchainConsensus::new(1, 0.51);

    let mut tx1 = Transaction::new("alice".to_string(), "bob".to_string(), 100, 0);
    tx1.sign(b"alice_key");

    let mut tx2 = Transaction::new("bob".to_string(), "charlie".to_string(), 50, 0);
    tx2.sign(b"bob_key");

    // Add to mempool
    assert!(consensus.add_to_mempool(tx1.clone()).is_ok());
    assert!(consensus.add_to_mempool(tx2.clone()).is_ok());

    let mempool_txs = consensus.get_mempool_transactions(10);
    assert_eq!(mempool_txs.len(), 2);

    // Clear mempool
    consensus.clear_mempool();
    assert_eq!(consensus.get_mempool_transactions(10).len(), 0);
}

#[test]
fn test_blockchain_nonce_validation() {
    let consensus = BlockchainConsensus::new(1, 0.51);

    let mut tx1 = Transaction::new("alice".to_string(), "bob".to_string(), 100, 0);
    tx1.sign(b"alice_key");
    assert!(consensus.add_to_mempool(tx1).is_ok());

    // Next transaction must have nonce 1
    let mut tx2 = Transaction::new("alice".to_string(), "charlie".to_string(), 50, 1);
    tx2.sign(b"alice_key");
    assert!(consensus.add_to_mempool(tx2).is_ok());

    // Wrong nonce should fail
    let mut tx3 = Transaction::new("alice".to_string(), "dave".to_string(), 25, 5);
    tx3.sign(b"alice_key");
    assert!(consensus.add_to_mempool(tx3).is_err());
}

#[test]
fn test_blockchain_block_validation() {
    let consensus = BlockchainConsensus::new(1, 0.51);

    let val = Validator::new("val1".to_string(), vec![1, 2, 3], 1000);
    consensus.add_validator(val).unwrap();

    let mut tx = Transaction::new("alice".to_string(), "bob".to_string(), 100, 0);
    tx.sign(b"alice_key");

    let block = Block::new(0, vec![], vec![tx], "val1".to_string());
    let result = consensus.validate_block(&block);
    assert!(result.is_valid());

    // Invalid block height
    let invalid_block = Block::new(5, vec![], vec![], "val1".to_string());
    let result = consensus.validate_block(&invalid_block);
    assert!(!result.is_valid());
}

#[test]
fn test_blockchain_chain_operations() {
    let consensus = BlockchainConsensus::new(1, 0.51);

    let val = Validator::new("val1".to_string(), vec![1, 2, 3], 1000);
    consensus.add_validator(val).unwrap();

    // Add genesis block
    let mut tx1 = Transaction::new("alice".to_string(), "bob".to_string(), 100, 0);
    tx1.sign(b"alice_key");
    let block1 = Block::new(0, vec![], vec![tx1], "val1".to_string());
    assert!(consensus.add_block(block1.clone()).is_ok());

    assert_eq!(consensus.get_height(), 1);

    // Add second block
    let mut tx2 = Transaction::new("bob".to_string(), "charlie".to_string(), 50, 0);
    tx2.sign(b"bob_key");
    let block2 = Block::new(1, block1.hash(), vec![tx2], "val1".to_string());
    assert!(consensus.add_block(block2).is_ok());

    assert_eq!(consensus.get_height(), 2);

    // Retrieve blocks
    let retrieved_block = consensus.get_block(0).unwrap();
    assert_eq!(retrieved_block.height, 0);

    let latest = consensus.get_latest_block().unwrap();
    assert_eq!(latest.height, 1);
}

#[test]
fn test_blockchain_consensus_achievement() {
    let consensus = BlockchainConsensus::new(3, 0.67);

    // Add 5 validators
    for i in 0..5 {
        let val = Validator::new(format!("val{}", i), vec![i as u8], 1000);
        consensus.add_validator(val).unwrap();
    }

    let mut tx = Transaction::new("alice".to_string(), "bob".to_string(), 100, 0);
    tx.sign(b"alice_key");

    let block = Block::new(0, vec![], vec![tx], "val0".to_string());
    let consensus_reached = consensus.achieve_consensus(&block).unwrap();
    assert!(consensus_reached);
}

// ============================================================================
// Virtual DOM Tests
// ============================================================================

#[test]
fn test_vdom_node_creation() {
    let element = VNode::element("div")
        .prop("class", "container")
        .prop("id", "app")
        .child(VNode::text("Hello, World!"))
        .into_vnode();

    match element {
        VNode::Element(el) => {
            assert_eq!(el.tag, "div");
            assert_eq!(el.props.get("class"), Some(&"container".to_string()));
            assert_eq!(el.props.get("id"), Some(&"app".to_string()));
            assert_eq!(el.children.len(), 1);
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_vdom_initial_render() {
    let mut vdom = VirtualDOM::new();

    let tree = VNode::element("div")
        .prop("id", "root")
        .child(VNode::text("Initial content"))
        .into_vnode();

    let patches = vdom.render(tree);
    assert_eq!(patches.len(), 1);

    match &patches[0] {
        Patch::Create { path, .. } => {
            assert!(path.is_empty());
        }
        _ => panic!("Expected Create patch"),
    }
}

#[test]
fn test_vdom_no_changes() {
    let mut vdom = VirtualDOM::new();

    let tree = VNode::element("div")
        .child(VNode::text("Content"))
        .into_vnode();

    vdom.render(tree.clone());
    let patches = vdom.render(tree);

    assert_eq!(patches.len(), 0);
}

#[test]
fn test_vdom_text_update() {
    let mut vdom = VirtualDOM::new();

    let tree1 = VNode::element("div")
        .child(VNode::text("Old text"))
        .into_vnode();

    let tree2 = VNode::element("div")
        .child(VNode::text("New text"))
        .into_vnode();

    vdom.render(tree1);
    let patches = vdom.render(tree2);

    assert_eq!(patches.len(), 1);
    match &patches[0] {
        Patch::UpdateText { content, path } => {
            assert_eq!(content, "New text");
            assert_eq!(path, &vec![0]);
        }
        _ => panic!("Expected UpdateText patch"),
    }
}

#[test]
fn test_vdom_prop_updates() {
    let mut vdom = VirtualDOM::new();

    let tree1 = VNode::element("div")
        .prop("class", "old-class")
        .prop("id", "app")
        .into_vnode();

    let tree2 = VNode::element("div")
        .prop("class", "new-class")
        .prop("data-value", "123")
        .into_vnode();

    vdom.render(tree1);
    let patches = vdom.render(tree2);

    assert_eq!(patches.len(), 1);
    match &patches[0] {
        Patch::UpdateProps { props, .. } => {
            assert_eq!(props.get("class"), Some(&Some("new-class".to_string())));
            assert_eq!(props.get("id"), Some(&None)); // Removed
            assert_eq!(props.get("data-value"), Some(&Some("123".to_string()))); // Added
        }
        _ => panic!("Expected UpdateProps patch"),
    }
}

#[test]
fn test_vdom_add_remove_children() {
    let mut vdom = VirtualDOM::new();

    let tree1 = VNode::element("ul")
        .child(VNode::element("li").child(VNode::text("Item 1")).into_vnode())
        .into_vnode();

    let tree2 = VNode::element("ul")
        .child(VNode::element("li").child(VNode::text("Item 1")).into_vnode())
        .child(VNode::element("li").child(VNode::text("Item 2")).into_vnode())
        .child(VNode::element("li").child(VNode::text("Item 3")).into_vnode())
        .into_vnode();

    vdom.render(tree1);
    let patches = vdom.render(tree2.clone());

    // Should have 2 Create patches for new children
    assert_eq!(patches.len(), 2);

    // Now remove children
    let tree3 = VNode::element("ul")
        .child(VNode::element("li").child(VNode::text("Item 1")).into_vnode())
        .into_vnode();

    let patches = vdom.render(tree3);
    assert_eq!(patches.len(), 2); // 2 Remove patches
}

#[test]
fn test_vdom_replace_node() {
    let mut vdom = VirtualDOM::new();

    let tree1 = VNode::element("div")
        .child(VNode::text("Content"))
        .into_vnode();

    let tree2 = VNode::element("span")
        .child(VNode::text("Content"))
        .into_vnode();

    vdom.render(tree1);
    let patches = vdom.render(tree2);

    assert_eq!(patches.len(), 1);
    match &patches[0] {
        Patch::Replace { path, .. } => {
            assert!(path.is_empty());
        }
        _ => panic!("Expected Replace patch"),
    }
}

#[test]
fn test_vdom_to_html() {
    let mut vdom = VirtualDOM::new();

    let tree = VNode::element("div")
        .prop("class", "container")
        .prop("id", "app")
        .child(
            VNode::element("h1")
                .child(VNode::text("Title"))
                .into_vnode(),
        )
        .child(
            VNode::element("p")
                .child(VNode::text("Paragraph text"))
                .into_vnode(),
        )
        .into_vnode();

    vdom.render(tree);
    let html = vdom.to_html();

    assert!(html.contains("<div"));
    assert!(html.contains("class=\"container\""));
    assert!(html.contains("id=\"app\""));
    assert!(html.contains("<h1>Title</h1>"));
    assert!(html.contains("<p>Paragraph text</p>"));
}

#[test]
fn test_vdom_html_escaping() {
    let mut vdom = VirtualDOM::new();

    let tree = VNode::element("div")
        .child(VNode::text("<script>alert('XSS')</script>"))
        .child(VNode::text("Normal & text"))
        .into_vnode();

    vdom.render(tree);
    let html = vdom.to_html();

    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("&amp;"));
}

#[test]
fn test_vdom_component_nodes() {
    let comp = VNode::component("Button")
        .prop("variant", "primary")
        .prop("size", "large")
        .child(VNode::text("Click me"))
        .into_vnode();

    match comp {
        VNode::Component(c) => {
            assert_eq!(c.name, "Button");
            assert_eq!(c.props.get("variant"), Some(&"primary".to_string()));
            assert_eq!(c.props.get("size"), Some(&"large".to_string()));
            assert_eq!(c.children.len(), 1);
        }
        _ => panic!("Expected component node"),
    }
}

// ============================================================================
// State Reducer Tests
// ============================================================================

#[test]
fn test_state_store_creation() {
    let initial_state = AppState::default();
    let store = Store::new(initial_state.clone(), Arc::new(app_reducer));

    assert_eq!(store.get_state(), initial_state);
    assert_eq!(store.get_state().user, None);
    assert!(!store.get_state().ui.loading);
}

#[test]
fn test_state_user_actions() {
    let store = Store::new(AppState::default(), Arc::new(app_reducer));

    let user = User {
        id: "1".to_string(),
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        role: "admin".to_string(),
    };

    // Login
    store.dispatch(&AppAction::LoginUser { user: user.clone() });
    assert_eq!(store.get_state().user, Some(user.clone()));

    // Update profile
    store.dispatch(&AppAction::UpdateUserProfile {
        name: "Alice Smith".to_string(),
        email: "alice.smith@example.com".to_string(),
    });

    let state = store.get_state();
    assert_eq!(state.user.as_ref().unwrap().name, "Alice Smith");
    assert_eq!(state.user.as_ref().unwrap().email, "alice.smith@example.com");

    // Logout
    store.dispatch(&AppAction::LogoutUser);
    assert_eq!(store.get_state().user, None);
}

#[test]
fn test_state_ui_actions() {
    let store = Store::new(AppState::default(), Arc::new(app_reducer));

    // Set loading
    store.dispatch(&AppAction::SetLoading { loading: true });
    assert!(store.get_state().ui.loading);

    store.dispatch(&AppAction::SetLoading { loading: false });
    assert!(!store.get_state().ui.loading);

    // Set error
    store.dispatch(&AppAction::SetError {
        error: Some("An error occurred".to_string()),
    });
    assert_eq!(
        store.get_state().ui.error,
        Some("An error occurred".to_string())
    );

    // Set theme
    store.dispatch(&AppAction::SetTheme {
        theme: "dark".to_string(),
    });
    assert_eq!(store.get_state().ui.theme, "dark");

    // Toggle sidebar
    assert!(store.get_state().ui.sidebar_open);
    store.dispatch(&AppAction::ToggleSidebar);
    assert!(!store.get_state().ui.sidebar_open);
    store.dispatch(&AppAction::ToggleSidebar);
    assert!(store.get_state().ui.sidebar_open);
}

#[test]
fn test_state_data_actions() {
    let store = Store::new(AppState::default(), Arc::new(app_reducer));

    // Add items
    store.dispatch(&AppAction::AddItem {
        item: "item1".to_string(),
    });
    store.dispatch(&AppAction::AddItem {
        item: "item2".to_string(),
    });
    store.dispatch(&AppAction::AddItem {
        item: "item3".to_string(),
    });

    assert_eq!(store.get_state().data.items.len(), 3);

    // Select item
    store.dispatch(&AppAction::SelectItem {
        item: Some("item2".to_string()),
    });
    assert_eq!(
        store.get_state().data.selected_item,
        Some("item2".to_string())
    );

    // Remove selected item
    store.dispatch(&AppAction::RemoveItem {
        item: "item2".to_string(),
    });
    assert_eq!(store.get_state().data.items.len(), 2);
    assert_eq!(store.get_state().data.selected_item, None);
}

#[test]
fn test_state_cache_operations() {
    let store = Store::new(AppState::default(), Arc::new(app_reducer));

    // Update cache
    store.dispatch(&AppAction::UpdateCache {
        key: "user:1".to_string(),
        value: "Alice".to_string(),
    });
    store.dispatch(&AppAction::UpdateCache {
        key: "user:2".to_string(),
        value: "Bob".to_string(),
    });

    let state = store.get_state();
    assert_eq!(state.data.cache.len(), 2);
    assert_eq!(state.data.cache.get("user:1"), Some(&"Alice".to_string()));

    // Clear cache
    store.dispatch(&AppAction::ClearCache);
    assert!(store.get_state().data.cache.is_empty());
}

#[test]
fn test_state_subscribers() {
    let store = Store::new(AppState::default(), Arc::new(app_reducer));

    let call_count = Arc::new(RwLock::new(0));
    let call_count_clone = Arc::clone(&call_count);

    let _sub_id = store.subscribe(Arc::new(move |_state| {
        let mut count = call_count_clone.write();
        *count += 1;
    }));

    store.dispatch(&AppAction::SetLoading { loading: true });
    store.dispatch(&AppAction::SetTheme {
        theme: "dark".to_string(),
    });
    store.dispatch(&AppAction::ToggleSidebar);

    assert_eq!(*call_count.read(), 3);
}

#[test]
fn test_state_middleware() {
    let mut store = Store::new(AppState::default(), Arc::new(app_reducer));

    let action_log = Arc::new(RwLock::new(Vec::new()));
    let action_log_clone = Arc::clone(&action_log);

    store.add_middleware(Arc::new(move |_state, action, next| {
        let mut log = action_log_clone.write();
        log.push(action.action_type().to_string());
        next(action);
    }));

    store.dispatch(&AppAction::SetLoading { loading: true });
    store.dispatch(&AppAction::SetTheme {
        theme: "dark".to_string(),
    });

    let log = action_log.read();
    assert_eq!(log.len(), 2);
    assert_eq!(log[0], "SET_LOADING");
    assert_eq!(log[1], "SET_THEME");
}

#[test]
fn test_state_complex_workflow() {
    let store = Store::new(AppState::default(), Arc::new(app_reducer));

    // Simulate a complete user workflow
    let user = User {
        id: "1".to_string(),
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        role: "user".to_string(),
    };

    // Login
    store.dispatch(&AppAction::LoginUser { user });
    store.dispatch(&AppAction::SetLoading { loading: true });

    // Load data
    store.dispatch(&AppAction::AddItem {
        item: "Task 1".to_string(),
    });
    store.dispatch(&AppAction::AddItem {
        item: "Task 2".to_string(),
    });

    store.dispatch(&AppAction::SetLoading { loading: false });

    // Update UI
    store.dispatch(&AppAction::SetTheme {
        theme: "dark".to_string(),
    });

    // Verify final state
    let state = store.get_state();
    assert!(state.user.is_some());
    assert!(!state.ui.loading);
    assert_eq!(state.ui.theme, "dark");
    assert_eq!(state.data.items.len(), 2);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_integration_blockchain_with_multiple_validators() {
    let consensus = BlockchainConsensus::new(5, 0.80);

    // Setup validators
    for i in 0..10 {
        let val = Validator::new(
            format!("validator_{}", i),
            vec![i as u8; 32],
            (i + 1) * 1000,
        );
        consensus.add_validator(val).unwrap();
    }

    // Create and validate transactions
    for i in 0..5 {
        let mut tx = Transaction::new(
            format!("user_{}", i),
            format!("user_{}", i + 1),
            (i + 1) * 100,
            0,
        );
        tx.sign(format!("key_{}", i).as_bytes());
        consensus.add_to_mempool(tx).unwrap();
    }

    // Create block with transactions
    let txs = consensus.get_mempool_transactions(5);
    let block = Block::new(0, vec![], txs, "validator_0".to_string());

    // Achieve consensus
    let consensus_reached = consensus.achieve_consensus(&block).unwrap();
    assert!(consensus_reached);

    // Add block to chain
    consensus.add_block(block).unwrap();
    assert_eq!(consensus.get_height(), 1);
}

#[test]
fn test_integration_vdom_with_state() {
    let mut vdom = VirtualDOM::new();
    let store = Store::new(AppState::default(), Arc::new(app_reducer));

    // Initial render
    let tree1 = VNode::element("div")
        .prop("class", "app")
        .child(VNode::element("h1").child(VNode::text("Welcome")).into_vnode())
        .child(
            VNode::element("p")
                .child(VNode::text("Loading: false"))
                .into_vnode(),
        )
        .into_vnode();

    vdom.render(tree1);

    // Update state
    store.dispatch(&AppAction::SetLoading { loading: true });

    // Re-render with new state
    let tree2 = VNode::element("div")
        .prop("class", "app")
        .child(VNode::element("h1").child(VNode::text("Welcome")).into_vnode())
        .child(
            VNode::element("p")
                .child(VNode::text("Loading: true"))
                .into_vnode(),
        )
        .into_vnode();

    let patches = vdom.render(tree2);
    assert!(!patches.is_empty());
}

#[test]
fn test_integration_full_stack() {
    // Blockchain layer
    let consensus = BlockchainConsensus::new(3, 0.67);
    for i in 0..5 {
        let val = Validator::new(format!("val{}", i), vec![i as u8], 1000);
        consensus.add_validator(val).unwrap();
    }

    // State management layer
    let store = Store::new(AppState::default(), Arc::new(app_reducer));

    // Virtual DOM layer
    let mut vdom = VirtualDOM::new();

    // Simulate transaction
    let mut tx = Transaction::new("alice".to_string(), "bob".to_string(), 100, 0);
    tx.sign(b"alice_key");
    consensus.add_to_mempool(tx.clone()).unwrap();

    // Update state
    store.dispatch(&AppAction::SetLoading { loading: true });
    store.dispatch(&AppAction::AddItem {
        item: format!("Transaction: {}", tx.id),
    });
    store.dispatch(&AppAction::SetLoading { loading: false });

    // Render UI
    let state = store.get_state();
    let tree = VNode::element("div")
        .child(
            VNode::element("div")
                .child(VNode::text(format!("Items: {}", state.data.items.len())))
                .into_vnode(),
        )
        .into_vnode();

    vdom.render(tree);
    let html = vdom.to_html();

    assert!(html.contains("Items: 1"));
}
