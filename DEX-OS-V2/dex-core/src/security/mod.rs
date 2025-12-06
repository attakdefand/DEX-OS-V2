//! Security module for the DEX-OS core engine

// Re-export all the security modules
pub mod access_control;
pub mod api_gateway;
pub mod api_key_manager;
pub mod api_rate_limiter;
pub mod bloom_filter;
pub mod bloom_filter_test;
pub mod client_protection;
pub mod contract_security;
pub mod cors_policy;
pub mod data_classification;
pub mod data_encryption;
pub mod data_sanitization;
pub mod field_encryption;
pub mod key_rotation;
pub mod kyber_encryption;
pub mod load_validation;
pub mod output_encoding;
pub mod quantum_signatures;
pub mod request_throttling;
pub mod ring_buffer;
pub mod security_manager;
pub mod self_healing;
pub mod threat_assessment;
pub mod token_validation;
pub mod tokenization;
pub mod whitelist_blacklist;
pub mod certificate_manager;
// Re-export commonly used types
pub use access_control::{
    AccessControlError, AccessControlManager, Action, Permission, Role, User,
    AccessDecision, AccessControlStatistics,
};
pub use api_gateway::{APIGateway, RouteConfig};
pub use api_key_manager::{APIKey, APIKeyManager, APIKeyError};
pub use api_rate_limiter::{APIRateLimiter, RateLimit, RateLimitError};
pub use bloom_filter::BloomFilter;
pub use client_protection::{
    ClientProtectionError, ClientProtectionManager, ContentSecurityPolicy, CsrfToken,
    SecureCookie, Session, SameSitePolicy,
};
pub use cors_policy::{CORSPolicy, HttpMethod};
pub use data_classification::{DataClassificationManager, ClassificationPolicy};
pub use data_encryption::{
    DataEncryptionManager, EncryptionError, EncryptionAlgorithm,
    EncryptionKey, EncryptionStatistics,
};
pub use field_encryption::FieldEncryptionManager;
pub use key_rotation::KeyRotationManager as DataKeyRotationManager;
pub use kyber_encryption::{
    KyberDecryptionResult, KyberEncryptedPackage, KyberEncryptionManager,
    KyberEncryptionOutput, KyberEncryptionStats, KyberError, KyberKeyPair,
    KyberSharedSecret,
};
pub use load_validation::{
    LoadValidationManager, LoadTestConfig, LoadTestResult,
};
pub use output_encoding::{
    OutputEncoder, EncodingError, EncodingContext, EncodedOutput,
};
pub use quantum_signatures::{
    DilithiumError, DilithiumKeyPair, DilithiumLevel, DilithiumSignature, DilithiumSignatureEngine,
};
pub use request_throttling::{
    RequestThrottler, ThrottlingError, ThrottlingAction, RequestMetadata,
    AdaptiveConfig, SystemLoad, ThrottlingStatistics,
};
pub use ring_buffer::{
    RingBuffer, RingBufferManager,
};
pub use self_healing::{
    SelfHealingSecuritySystem, SecurityEvent as HealingSecurityEvent, SecurityEventType, HealingAction,
    AnomalyDetectionResult, HealingResponse, HealingMetrics
};
pub use threat_assessment::{
    ThreatAssessmentManager, Vulnerability, Severity, VulnerabilityStatus, ThreatAssessmentReport,
};
pub use token_validation::{    TokenValidator, TokenValidationError, TokenClaims, TokenManager,
};
pub use tokenization::{TokenizationManager, TokenDataType};
pub use whitelist_blacklist::{
    WhitelistBlacklistManager, ListType, EntityType, ListError,
};
// Re-export security manager types
pub use security_manager::{
    EventType, SecurityManager, SeverityLevel, SecurityError, ClassificationLevel,
    Certificate, SecurityEvent, PIIDetection, Key,
};
pub use certificate_manager::CertificateManager;
