//! Test runner for our new security tests

mod security_test_functions;

fn main() {
    println!("Running security tests for new modules...");

    // Run snapshot tests
    println!("Running snapshot tests...");
    security_test_functions::test_security__governance_and_policy__snapshot__enforces__on_request();
    security_test_functions::test_security__governance_and_policy__snapshot__validates__on_request(
    );
    security_test_functions::test_security__governance_and_policy__snapshot__rotates__on_request();
    security_test_functions::test_security__governance_and_policy__snapshot__blocks__on_request();
    security_test_functions::test_security__governance_and_policy__snapshot__detects__on_request();
    security_test_functions::test_security__governance_and_policy__snapshot__logs_evidence__on_request();

    // Run keeper tests
    println!("Running keeper tests...");
    security_test_functions::test_security__governance_and_policy__keeper__enforces__on_request();
    security_test_functions::test_security__governance_and_policy__keeper__validates__on_request();
    security_test_functions::test_security__governance_and_policy__keeper__rotates__on_request();
    security_test_functions::test_security__governance_and_policy__keeper__blocks__on_request();
    security_test_functions::test_security__governance_and_policy__keeper__detects__on_request();
    security_test_functions::test_security__governance_and_policy__keeper__logs_evidence__on_request();

    // Run indexer tests
    println!("Running indexer tests...");
    security_test_functions::test_security__governance_and_policy__indexer__enforces__on_request();
    security_test_functions::test_security__governance_and_policy__indexer__validates__on_request();
    security_test_functions::test_security__governance_and_policy__indexer__rotates__on_request();
    security_test_functions::test_security__governance_and_policy__indexer__blocks__on_request();
    security_test_functions::test_security__governance_and_policy__indexer__detects__on_request();
    security_test_functions::test_security__governance_and_policy__indexer__logs_evidence__on_request();

    println!("All security tests passed!");
}
