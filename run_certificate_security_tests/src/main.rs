//! Test runner for B+ Tree Certificate Management security tests

mod security_bplus_tree_certificate_tests;

fn main() {
    println!("Running security tests for B+ Tree Certificate Management...");
    
    // Run all security tests for B+ Tree Certificate Management
    security_bplus_tree_certificate_tests::test_security__security__security__bplus_tree__certificate_management__enforces__on_request();
    println!("✓ Enforces test passed");
    
    security_bplus_tree_certificate_tests::test_security__security__security__bplus_tree__certificate_management__validates__on_request();
    println!("✓ Validates test passed");
    
    security_bplus_tree_certificate_tests::test_security__security__security__bplus_tree__certificate_management__blocks__on_request();
    println!("✓ Blocks test passed");
    
    security_bplus_tree_certificate_tests::test_security__security__security__bplus_tree__certificate_management__detects__on_request();
    println!("✓ Detects test passed");
    
    security_bplus_tree_certificate_tests::test_security__security__security__bplus_tree__certificate_management__logs_evidence__on_request();
    println!("✓ Logs evidence test passed");
    
    security_bplus_tree_certificate_tests::test_security__security__security__bplus_tree__certificate_management__rotates__on_request();
    println!("✓ Rotates test passed");
    
    println!("All B+ Tree Certificate Management security tests passed!");
}