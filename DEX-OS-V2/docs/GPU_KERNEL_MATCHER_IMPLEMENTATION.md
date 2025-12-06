# GPU Kernel Matcher

Priority 2 feature: `Components, Execution Engine, Engine, GPU Kernel Matcher, Kernel Matching, High`

- **Module**: `dex-core/src/execution_engine.rs`
- **Purpose**: Register GPU kernels with hardware metadata, describe kernel requests (compute capability, memory, warp size, and required operations), and select the best fitting kernel slot using a preference score that considers performance, available concurrency, and memory headroom.
- **Resilience**: Each kernel tracks concurrent job slots to limit overcommitment and allows releases to make capacity available again.
- **Replayability**: Assignments are tracked by request ID so higher layers can safely `release_assignment` when work completes; the same matcher instance can be used by nodes that need to verify allocation state.

## Example

```rust
use dex_core::execution_engine::{GpuKernel, GpuKernelMatcher, KernelRequest, RequestPriority, GpuOperation};

let mut matcher = GpuKernelMatcher::new();
matcher.register_kernel(GpuKernel::new(
    "kernel-1",
    "Ampere-X",
    8,
    6,
    16_384,
    96,
    32,
    [GpuOperation::TensorCore, GpuOperation::FP64]
        .into_iter()
        .collect(),
    8,
    9_800,
    250,
));

let request = KernelRequest {
    request_id: "job-42".into(),
    required_compute_major: 7,
    required_compute_minor: 0,
    required_memory_mib: 4_096,
    required_shared_memory_mib: 64,
    required_ops: vec![GpuOperation::TensorCore],
    desired_warp_size: Some(32),
    priority: RequestPriority::High,
    expected_runtime_ms: 1_000,
};

let assignment = matcher.match_kernel(request)?;
println!("Job {} assigned to {}", assignment.request_id, assignment.kernel_id);
matcher.release_assignment(&assignment.request_id)?;
```
