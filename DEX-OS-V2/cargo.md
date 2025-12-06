PS D:\DEX-OS-V2\DEX-OS-V2> cargo test
   Compiling bloom_filter_access_control v0.1.0 (D:\DEX-OS-V2\DEX-OS-V2)
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
  --> tests\quantum_consensus_test.rs:10:5
   |
10 | use dex_core::quantum_consensus::{QuantumConsensusEngine, QVRF, LatticeBFTCore, QuantumConse... 
   |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
   |
   = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your 
`Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
  --> tests\quantum_consensus_test.rs:11:5
   |
11 | use dex_core::types::{Block, Transaction, Validator};
   |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
   |
   = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your 
`Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\security_tests.rs:5:5
  |
5 | use dex_core::security::{SecurityManager, ClassificationLevel, EventType};
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\security_tests.rs:6:5
  |
6 | use dex_core::identity::IdentityManager;
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\security_tests.rs:7:5
  |
7 | use dex_core::governance::GlobalDAO;
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:410:13
    |
410 |         use dex_core::snapshot::{SnapshotManager, SnapshotMetadata};
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:411:13
    |
411 |         use dex_core::types::TraderId;
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:452:13
    |
452 |         use dex_core::snapshot::{SnapshotManager, SnapshotMetadata};
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:453:13
    |
453 |         use dex_core::types::TraderId;
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:454:13
    |
454 |         use dex_core::governance::{Proposal, ProposalStatus, ProposalType};
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:510:13
    |
510 |         use dex_core::snapshot::{SnapshotManager, SnapshotMetadata};
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:511:13
    |
511 |         use dex_core::types::TraderId;
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:569:13
    |
569 |         use dex_core::snapshot::{SnapshotManager, SnapshotMetadata, SnapshotError};
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:570:13
    |
570 |         use dex_core::types::TraderId;
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:589:13
    |
589 |         use dex_core::snapshot::{SnapshotManager, SnapshotMetadata};
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:590:13
    |
590 |         use dex_core::types::TraderId;
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:629:13
    |
629 |         use dex_core::snapshot::{SnapshotManager, SnapshotMetadata};
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:630:13
    |
630 |         use dex_core::security::{SecurityManager, EventType};
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:682:13
    |
682 |         use dex_core::keeper::{KeeperService, HealthStatus, AlertConfig};
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:714:13
    |
714 |         use dex_core::keeper::{KeeperService, HealthStatus, AlertConfig};
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\bloom_filter_tests.rs:3:5
  |
3 | use dex_core::security::BloomFilter;
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:735:13
    |
735 |         use dex_core::keeper::{KeeperService, HealthStatus, AlertConfig};
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\bplus_tree_certificate_test.rs:7:5
  |
7 | use dex_core::security::{Certificate, CertificateManager, SecurityError};
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:774:13
    |
774 |         use dex_core::keeper::{KeeperService, HealthStatus};
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:789:13
    |
789 |         use dex_core::keeper::{KeeperService, HealthStatus, AlertConfig};
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:832:13
    |
832 |         use dex_core::keeper::{KeeperService, HealthStatus};
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:833:13
    |
833 |         use dex_core::security::{SecurityManager, EventType};
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\test_runner.rs:3:5
  |
3 | use dex_core::test_results::{TestResultsManager, TestSuiteResult, IndividualTestResult, TestS... 
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:879:13
    |
879 |         use dex_core::indexer::{IndexerService, DataFilter, FilterCriteria};
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:914:13
    |
914 |         use dex_core::indexer::{IndexerService, DataFilter, FilterCriteria, IndexerError};     
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:952:13
    |
952 |         use dex_core::indexer::{IndexerService, DataFilter, FilterCriteria};
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
    --> tests\security_tests.rs:1007:13
     |
1007 |         use dex_core::indexer::{IndexerService, IndexerError};
     |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
     |
     = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
    --> tests\security_tests.rs:1030:13
     |
1030 |         use dex_core::indexer::{IndexerService, DataFilter, FilterCriteria};
     |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
     |
     = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
    --> tests\security_tests.rs:1081:13
     |
1081 |         use dex_core::indexer::{IndexerService, DataFilter, FilterCriteria};
     |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
     |
     = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
    --> tests\security_tests.rs:1082:13
     |
1082 |         use dex_core::security::{SecurityManager, EventType};
     |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
     |
     = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

For more information about this error, try `rustc --explain E0433`.
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\security_comprehensive_tests.rs:6:5
  |
6 | use dex_core::crypto::zk_proof::PrivacyProtectionService;
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\security_comprehensive_tests.rs:7:5
  |
7 | use dex_core::network::gossip_sync::{GossipSyncConfig, GossipSyncNode, SyncData};
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\security_comprehensive_tests.rs:8:5
  |
8 | use dex_core::security::{SecurityManager, EventType, SeverityLevel};
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\security_gossip_sync_tests.rs:6:5
  |
6 | use dex_core::network::gossip_sync::{GossipSyncConfig, GossipSyncNode, SyncData};
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\bloom_filter_test_coverage_runner.rs:6:5
  |
6 | use dex_core::test_coverage::{TestCoverageTracker, TestCoverageStats};
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\bloom_filter_test_coverage_runner.rs:7:5
  |
7 | use dex_core::test_results::{TestResultsManager, TestSuiteResult, IndividualTestResult, TestS... 
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error: could not compile `bloom_filter_access_control` (test "quantum_consensus_test") due to 2 previous errors
warning: build failed, waiting for other jobs to finish...
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\security_event_logging_tests.rs:6:5
  |
6 | use dex_core::security::{EventLogger, EventType, SecurityEvent, SeverityLevel};
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\global_identity_security_tests.rs:8:5
  |
8 | use dex_core::identity::{IdentityManager, QuantumSecureCrypto};
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\global_identity_security_tests.rs:9:5
  |
9 | use dex_core::security::SecurityManager;
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\global_identity_security_tests.rs:198:9
    |
198 |     use dex_core::test_results::{TestResultsManager, TestSuiteResult, IndividualTestResult,... 
    |         ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\access_control_bloom_tests.rs:3:5
  |
3 | use dex_core::security::{SecurityManager, ClassificationLevel};
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error: could not compile `bloom_filter_access_control` (test "bplus_tree_certificate_test") due to 1 
previous error
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\bloom_filter_security_coverage_tests.rs:6:5
  |
6 | use dex_core::test_coverage::{TestCoverageTracker, TestCoverageStats};
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\bloom_filter_security_coverage_tests.rs:7:5
  |
7 | use dex_core::test_results::{TestResultsManager, TestSuiteResult, IndividualTestResult, TestS... 
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error: could not compile `bloom_filter_access_control` (test "bloom_filter_tests") due to 1 previous 
error
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\security_test_coverage_tests.rs:6:5
  |
6 | use dex_core::test_coverage::{TestCoverageTracker, TestCoverageStats};
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_test_coverage_tests.rs:141:9
    |
141 |     use dex_core::test_results::{TestResultsManager, TestSuiteResult, IndividualTestResult,... 
    |         ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\security_zk_proof_tests.rs:6:5
  |
6 | use dex_core::crypto::zk_proof::{PrivacyProtectionService, ZkProofSystem};
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\integration_test.rs:7:9
  |
7 |     use dex_core::{
  |         ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\test_coverage_integration_tests.rs:2:5
  |
2 | use dex_core::test_coverage::TestCoverageTracker;
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\test_coverage_integration_tests.rs:3:5
  |
3 | use dex_core::test_results::{TestResultsManager, TestSuiteResult};
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\test_coverage_integration_tests.rs:4:5
  |
4 | use dex_core::types::TestResult;
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error: could not compile `bloom_filter_access_control` (test "access_control_bloom_tests") due to 1 previous error
error: could not compile `bloom_filter_access_control` (test "test_runner") due to 1 previous error  
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\global_identity_feature_tests.rs:8:5
  |
8 | use dex_core::identity::{IdentityManager, QuantumSecureCrypto};
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
 --> tests\security_bplus_tree_certificate_tests.rs:6:5
  |
6 | use dex_core::security::{Certificate, CertificateManager, SecurityError, SecurityManager};       
  |     ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
  |
  = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
  --> tests\security_comprehensive_tests.rs:26:24
   |
26 |     let public_input = dex_core::crypto::zk_proof::ZkProofSystem::new().compute_public_input... 
   |                        ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
   |
   = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your 
`Cargo.toml`

error[E0432]: unresolved import `dex_db`
  --> tests\integration_test.rs:12:9
   |
12 |     use dex_db::{DatabaseManager, migrations};
   |         ^^^^^^ use of unresolved module or unlinked crate `dex_db`
   |
   = help: if you wanted to use a crate named `dex_db`, use `cargo add dex_db` to add it to your `Cargo.toml`

error[E0432]: unresolved import `sqlx`
  --> tests\integration_test.rs:13:9
   |
13 |     use sqlx::{PgPool, Row};
   |         ^^^^ use of unresolved module or unlinked crate `sqlx`
   |
   = help: if you wanted to use a crate named `sqlx`, use `cargo add sqlx` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
  --> tests\global_identity_security_tests.rs:87:26
   |
87 |         let credential = dex_core::identity::VerifiableCredential {
   |                          ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
   |
   = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your 
`Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:176:27
    |
176 |         let certificate = dex_core::security::Certificate {
    |                           ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`       
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:302:27
    |
302 |         let certificate = dex_core::security::Certificate {
    |                           ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`       
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

warning: unused import: `std::collections::HashMap`
  --> tests\integration_test.rs:14:9
   |
14 |     use std::collections::HashMap;
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_comprehensive_tests.rs:100:24
    |
100 |     let public_input = dex_core::crypto::zk_proof::ZkProofSystem::new().compute_public_inpu... 
    |                        ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

warning: unused import: `std::collections::HashMap`
 --> tests\security_gossip_sync_tests.rs:7:5
  |
7 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_comprehensive_tests.rs:136:24
    |
136 |     let public_input = dex_core::crypto::zk_proof::ZkProofSystem::new().compute_public_inpu... 
    |                        ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error: could not compile `bloom_filter_access_control` (test "security_event_logging_tests") due to 1 previous error
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:354:13
    |
354 |             dex_core::governance::ProposalType::ParameterChange,
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:355:13
    |
355 |             dex_core::governance::Proposer::AI {
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error: could not compile `bloom_filter_access_control` (test "bloom_filter_security_coverage_tests") 
due to 2 previous errors
error: could not compile `bloom_filter_access_control` (test "bloom_filter_test_coverage_runner") due to 2 previous errors
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_comprehensive_tests.rs:180:24
    |
180 |     let public_input = dex_core::crypto::zk_proof::ZkProofSystem::new().compute_public_inpu... 
    |                        ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_tests.rs:367:13
    |
367 |             dex_core::governance::Proposer::AI { model_id, confidence, .. } => {
    |             ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_bplus_tree_certificate_tests.rs:220:9
    |
220 |         dex_core::security::EventType::AuditTrail,
    |         ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
  --> tests\global_identity_feature_tests.rs:85:26
   |
85 |         let credential = dex_core::identity::VerifiableCredential {
   |                          ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
   |
   = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your 
`Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `tokio_test`
  --> tests\security_gossip_sync_tests.rs:26:5
   |
26 |     tokio_test::block_on(async {
   |     ^^^^^^^^^^ use of unresolved module or unlinked crate `tokio_test`
   |
   = help: if you wanted to use a crate named `tokio_test`, use `cargo add tokio_test` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_bplus_tree_certificate_tests.rs:225:9
    |
225 |         dex_core::security::SeverityLevel::Info,
    |         ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `tokio_test`
  --> tests\security_gossip_sync_tests.rs:48:5
   |
48 |     tokio_test::block_on(async {
   |     ^^^^^^^^^^ use of unresolved module or unlinked crate `tokio_test`
   |
   = help: if you wanted to use a crate named `tokio_test`, use `cargo add tokio_test` to add it to your `Cargo.toml`

warning: unused import: `std::collections::HashMap`
   --> tests\security_tests.rs:571:13
    |
571 |         use std::collections::HashMap;
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `std::collections::HashMap`
   --> tests\security_tests.rs:736:13
    |
736 |         use std::collections::HashMap;
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::HashMap`
   --> tests\security_tests.rs:775:13
    |
775 |         use std::collections::HashMap;
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::HashMap`
   --> tests\security_tests.rs:880:13
    |
880 |         use std::collections::HashMap;
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::HashMap`
   --> tests\security_tests.rs:915:13
    |
915 |         use std::collections::HashMap;
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::HashMap`
   --> tests\security_tests.rs:953:13
    |
953 |         use std::collections::HashMap;
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::HashMap`
    --> tests\security_tests.rs:1008:13
     |
1008 |         use std::collections::HashMap;
     |             ^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dex_core`
   --> tests\security_bplus_tree_certificate_tests.rs:231:54
    |
231 | ...ents_by_type(dex_core::security::EventType::AuditTrail);
    |                 ^^^^^^^^ use of unresolved module or unlinked crate `dex_core`
    |
    = help: if you wanted to use a crate named `dex_core`, use `cargo add dex_core` to add it to your `Cargo.toml`

Some errors have detailed explanations: E0432, E0433.
For more information about an error, try `rustc --explain E0432`.
error: could not compile `bloom_filter_access_control` (test "test_coverage_integration_tests") due to 3 previous errors
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `tokio_test`
  --> tests\security_comprehensive_tests.rs:57:5
   |
57 |     tokio_test::block_on(async {
   |     ^^^^^^^^^^ use of unresolved module or unlinked crate `tokio_test`
   |
   = help: if you wanted to use a crate named `tokio_test`, use `cargo add tokio_test` to add it to your `Cargo.toml`

warning: `bloom_filter_access_control` (test "integration_test") generated 1 warning
error: could not compile `bloom_filter_access_control` (test "integration_test") due to 3 previous errors; 1 warning emitted
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `tokio_test`
   --> tests\security_comprehensive_tests.rs:148:5
    |
148 |     tokio_test::block_on(async {
    |     ^^^^^^^^^^ use of unresolved module or unlinked crate `tokio_test`
    |
    = help: if you wanted to use a crate named `tokio_test`, use `cargo add tokio_test` to add it to 
your `Cargo.toml`

error: could not compile `bloom_filter_access_control` (test "security_test_coverage_tests") due to 2 previous errors
error: could not compile `bloom_filter_access_control` (test "security_zk_proof_tests") due to 1 previous error
error: could not compile `bloom_filter_access_control` (test "global_identity_security_tests") due to 4 previous errors
error: could not compile `bloom_filter_access_control` (test "security_bplus_tree_certificate_tests") due to 4 previous errors
error: could not compile `bloom_filter_access_control` (test "security_comprehensive_tests") due to 9 previous errors
error: could not compile `bloom_filter_access_control` (test "global_identity_feature_tests") due to 
2 previous errors
warning: `bloom_filter_access_control` (test "security_gossip_sync_tests") generated 1 warning       
error: could not compile `bloom_filter_access_control` (test "security_gossip_sync_tests") due to 3 previous errors; 1 warning emitted
warning: `bloom_filter_access_control` (test "security_tests") generated 7 warnings
error: could not compile `bloom_filter_access_control` (test "security_tests") due to 35 previous errors; 7 warnings emitted
PS D:\DEX-OS-V2\DEX-OS-V2> 