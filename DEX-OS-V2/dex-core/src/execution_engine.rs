//! GPU kernel matcher for the execution engine.
//!
//! Implements the Priority 2 feature from DEX-OS-V2.csv:
//! - Components,Execution Engine,Engine,GPU Kernel Matcher,Kernel Matching,High

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Compute capabilities that GPU kernels can expose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GpuOperation {
    TensorCore,
    MatrixMultiply,
    FP64,
    Integer,
    AsyncCopy,
    // Future-proof hook for custom kernel features.
    Custom(u16),
}

/// Priority classes for kernel requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestPriority {
    Low,
    Medium,
    High,
}

/// Request description for a GPU kernel assignment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelRequest {
    pub request_id: String,
    pub required_compute_major: u8,
    pub required_compute_minor: u8,
    pub required_memory_mib: u32,
    pub required_shared_memory_mib: u32,
    pub required_ops: Vec<GpuOperation>,
    pub desired_warp_size: Option<u16>,
    pub priority: RequestPriority,
    pub expected_runtime_ms: u64,
}

/// Assignment metadata returned when a kernel is matched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuKernelAssignment {
    pub request_id: String,
    pub kernel_id: String,
    pub assigned_at_ms: u128,
    pub priority: RequestPriority,
}

/// GPU kernel descriptor with capacity and capability metadata.
#[derive(Debug, Clone)]
pub struct GpuKernel {
    pub kernel_id: String,
    pub device_name: String,
    pub compute_major: u8,
    pub compute_minor: u8,
    pub total_memory_mib: u32,
    pub shared_memory_mib: u32,
    pub warp_size: u16,
    pub supported_ops: HashSet<GpuOperation>,
    pub max_concurrent_jobs: usize,
    pub performance_score: u32,
    pub power_watts: u16,
    active_jobs: usize,
}

impl GpuKernel {
    /// Create a new GPU kernel descriptor.
    pub fn new(
        kernel_id: impl Into<String>,
        device_name: impl Into<String>,
        compute_major: u8,
        compute_minor: u8,
        total_memory_mib: u32,
        shared_memory_mib: u32,
        warp_size: u16,
        supported_ops: HashSet<GpuOperation>,
        max_concurrent_jobs: usize,
        performance_score: u32,
        power_watts: u16,
    ) -> Self {
        Self {
            kernel_id: kernel_id.into(),
            device_name: device_name.into(),
            compute_major,
            compute_minor,
            total_memory_mib,
            shared_memory_mib,
            warp_size,
            supported_ops,
            max_concurrent_jobs: max_concurrent_jobs.max(1),
            performance_score,
            power_watts,
            active_jobs: 0,
        }
    }

    fn compute_capability_score(&self) -> u32 {
        (self.compute_major as u32) * 100 + (self.compute_minor as u32)
    }

    fn available_slots(&self) -> usize {
        self.max_concurrent_jobs.saturating_sub(self.active_jobs)
    }

    fn can_host(&self, request: &KernelRequest) -> bool {
        if self.available_slots() == 0 {
            return false;
        }
        if self.compute_major < request.required_compute_major {
            return false;
        }
        if self.compute_major == request.required_compute_major
            && self.compute_minor < request.required_compute_minor
        {
            return false;
        }
        if self.total_memory_mib < request.required_memory_mib {
            return false;
        }
        if self.shared_memory_mib < request.required_shared_memory_mib {
            return false;
        }
        if let Some(warp) = request.desired_warp_size {
            if self.warp_size < warp {
                return false;
            }
        }
        request
            .required_ops
            .iter()
            .all(|op| self.supported_ops.contains(op))
    }

    fn match_score(&self, request: &KernelRequest) -> (u32, usize, u32, u32) {
        let headroom = self
            .total_memory_mib
            .saturating_sub(request.required_memory_mib);
        (
            self.performance_score,
            self.available_slots(),
            u32::MAX.saturating_sub(headroom),
            self.compute_capability_score(),
        )
    }

    fn reserve_slot(&mut self) -> bool {
        if self.available_slots() == 0 {
            return false;
        }
        self.active_jobs += 1;
        true
    }

    fn release_slot(&mut self) {
        if self.active_jobs > 0 {
            self.active_jobs -= 1;
        }
    }
}

/// Errors raised by the execution engine.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ExecutionEngineError {
    #[error("Duplicate request: {0}")]
    DuplicateRequest(String),
    #[error("No matching kernel for request: {0}")]
    NoMatchingKernel(String),
    #[error("Assignment not found for request: {0}")]
    AssignmentNotFound(String),
}

/// Scheduler that matches kernel requests to available GPUs.
#[derive(Debug, Clone)]
pub struct GpuKernelMatcher {
    kernels: Vec<GpuKernel>,
    assignments: HashMap<String, GpuKernelAssignment>,
}

impl Default for GpuKernelMatcher {
    fn default() -> Self {
        Self {
            kernels: Vec::new(),
            assignments: HashMap::new(),
        }
    }
}

impl GpuKernelMatcher {
    /// Create a fresh matcher.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new GPU kernel with the matcher.
    pub fn register_kernel(&mut self, kernel: GpuKernel) {
        self.kernels.push(kernel);
    }

    /// Return the number of registered kernels.
    pub fn kernel_count(&self) -> usize {
        self.kernels.len()
    }

    /// Execute kernel matching for the provided request.
    pub fn match_kernel(
        &mut self,
        request: KernelRequest,
    ) -> Result<GpuKernelAssignment, ExecutionEngineError> {
        if self.assignments.contains_key(&request.request_id) {
            return Err(ExecutionEngineError::DuplicateRequest(
                request.request_id.clone(),
            ));
        }

        let candidate_index = self
            .kernels
            .iter_mut()
            .enumerate()
            .filter(|(_, kernel)| kernel.can_host(&request))
            .max_by_key(|(_, kernel)| kernel.match_score(&request))
            .map(|(idx, _)| idx);

        let index = match candidate_index {
            Some(idx) => idx,
            None => return Err(ExecutionEngineError::NoMatchingKernel(request.request_id)),
        };

        let kernel = &mut self.kernels[index];
        if !kernel.reserve_slot() {
            return Err(ExecutionEngineError::NoMatchingKernel(request.request_id));
        }

        let assignment = GpuKernelAssignment {
            request_id: request.request_id.clone(),
            kernel_id: kernel.kernel_id.clone(),
            assigned_at_ms: now_ms(),
            priority: request.priority,
        };

        self.assignments
            .insert(request.request_id.clone(), assignment.clone());

        Ok(assignment)
    }

    /// Release the kernel slot for a previously matched request.
    pub fn release_assignment(
        &mut self,
        request_id: &str,
    ) -> Result<(), ExecutionEngineError> {
        let assignment = self
            .assignments
            .remove(request_id)
            .ok_or_else(|| ExecutionEngineError::AssignmentNotFound(request_id.to_string()))?;

        if let Some(kernel) = self
            .kernels
            .iter_mut()
            .find(|kernel| kernel.kernel_id == assignment.kernel_id)
        {
            kernel.release_slot();
        }

        Ok(())
    }

    /// Get the number of active assignments.
    pub fn active_assignments(&self) -> usize {
        self.assignments.len()
    }

    /// Inspect registered kernels (read-only).
    pub fn kernels(&self) -> &[GpuKernel] {
        &self.kernels
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_kernel(id: &str, ops: &[GpuOperation]) -> GpuKernel {
        GpuKernel::new(
            id,
            "RADEON-X",
            8,
            6,
            16_384,
            96,
            32,
            ops.iter().copied().collect(),
            4,
            9_600,
            250,
        )
    }

    fn sample_request(priority: RequestPriority, ops: &[GpuOperation]) -> KernelRequest {
        KernelRequest {
            request_id: "job-kw".to_string(),
            required_compute_major: 7,
            required_compute_minor: 0,
            required_memory_mib: 4_096,
            required_shared_memory_mib: 64,
            required_ops: ops.to_vec(),
            desired_warp_size: Some(32),
            priority,
            expected_runtime_ms: 1_000,
        }
    }

    #[test]
    fn selects_highest_performance_kernel() {
        let mut matcher = GpuKernelMatcher::new();
        matcher.register_kernel(GpuKernel::new(
            "kernel-legacy",
            "Legacy GPU",
            6,
            1,
            8_192,
            48,
            32,
            [GpuOperation::FP64].into_iter().collect(),
            2,
            5_000,
            180,
        ));
        matcher.register_kernel(default_kernel("kernel-top", &[GpuOperation::TensorCore]));

        let request = sample_request(RequestPriority::High, &[GpuOperation::TensorCore]);
        let assignment = matcher.match_kernel(request).expect("match should succeed");

        assert_eq!(assignment.kernel_id, "kernel-top");
        assert_eq!(matcher.active_assignments(), 1);
    }

    #[test]
    fn fails_when_operation_unavailable() {
        let mut matcher = GpuKernelMatcher::new();
        matcher.register_kernel(default_kernel("kernel-safe", &[GpuOperation::FP64]));

        let request = sample_request(RequestPriority::Medium, &[GpuOperation::TensorCore]);
        let err = matcher.match_kernel(request).unwrap_err();
        assert_eq!(
            err,
            ExecutionEngineError::NoMatchingKernel("job-kw".to_string())
        );
    }

    #[test]
    fn releases_slots_for_reuse() {
        let mut matcher = GpuKernelMatcher::new();
        matcher.register_kernel(GpuKernel::new(
            "kernel-limited",
            "Edge GPU",
            7,
            5,
            4_096,
            48,
            32,
            [GpuOperation::FP64, GpuOperation::Integer]
                .into_iter()
                .collect(),
            2,
            3_200,
            120,
        ));

        for idx in 0..2 {
            let request = KernelRequest {
                request_id: format!("job-{}", idx),
                required_compute_major: 7,
                required_compute_minor: 5,
                required_memory_mib: 2_000,
                required_shared_memory_mib: 32,
                required_ops: vec![GpuOperation::FP64],
                desired_warp_size: Some(32),
                priority: RequestPriority::Low,
                expected_runtime_ms: 500,
            };
            matcher.match_kernel(request).expect("should fit");
        }

        let request = KernelRequest {
            request_id: "job-3".to_string(),
            required_compute_major: 7,
            required_compute_minor: 5,
            required_memory_mib: 2_000,
            required_shared_memory_mib: 32,
            required_ops: vec![GpuOperation::FP64],
            desired_warp_size: Some(32),
            priority: RequestPriority::Low,
            expected_runtime_ms: 500,
        };

        assert!(matches!(
            matcher.match_kernel(request.clone()),
            Err(ExecutionEngineError::NoMatchingKernel(_))
        ));

        matcher
            .release_assignment("job-0")
            .expect("release should succeed");
        let assignment = matcher.match_kernel(request).expect("retry should pass");
        assert_eq!(assignment.kernel_id, "kernel-limited");
        assert_eq!(matcher.active_assignments(), 2);
    }
}
