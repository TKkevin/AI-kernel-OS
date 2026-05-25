# Contributing to NEXUS

Welcome to the NEXUS project! We're building the next-generation operating system through collaborative engineering. This guide will help you contribute effectively.

## Code of Conduct

We are committed to providing a welcoming and inspiring community for all. Read our [Code of Conduct](CODE_OF_CONDUCT.md).

## Getting Started

### Prerequisites
- Rust 1.70+
- Cargo package manager
- Linux/macOS development environment
- Basic understanding of OS concepts

### Setup

```bash
# Clone the repository
git clone https://github.com/TKkevin/AI-kernel-OS
cd AI-kernel-OS

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the project
make build

# Run tests
make test
```

## Development Workflow

### 1. Choose an Issue

Start with issues labeled:
- `good-first-issue` - Perfect for newcomers
- `help-wanted` - Community contributions needed
- `enhancement` - New features
- `bug` - Bug fixes

### 2. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
```

Branch naming convention:
- `feature/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation
- `perf/description` - Performance improvements
- `refactor/description` - Code refactoring

### 3. Write Code with Stark Engineering Principles

#### Code Quality Standards

**Before you commit:**

```bash
# Format your code
make fmt

# Run linter
make lint

# Run tests
make test

# Build release version
make build
```

#### Code Style

- Follow Rust naming conventions
- Write self-documenting code
- Comment the "why", not the "what"
- Add rustdoc comments to public APIs

```rust
/// Brief description of what this function does.
///
/// # Arguments
///
/// * `param1` - Description of first parameter
///
/// # Returns
///
/// Description of return value
///
/// # Examples
///
/// ```
/// let result = my_function(42);
/// assert_eq!(result, 84);
/// ```
pub fn my_function(param1: u32) -> u32 {
    param1 * 2
}
```

### 4. Performance Awareness

For kernel code, performance is mandatory:

- Measure everything
- No performance regressions allowed
- Include benchmarks for critical paths
- Document performance characteristics

```rust
// Bad: No measurement
let result = expensive_operation();

// Good: Measured performance
let start = std::time::Instant::now();
let result = expensive_operation();
let elapsed = start.elapsed();
assert!(elapsed.as_micros() < 1000, "Operation exceeded 1ms budget");
```

### 5. Security Consciousness

For security-critical code:

- Use safe abstractions
- Avoid unsafe blocks when possible
- Document all safety invariants
- Include security tests

```rust
/// This is safe because:
/// 1. The pointer is always valid
/// 2. No concurrent access occurs
/// 3. Alignment is guaranteed by the allocator
unsafe fn critical_operation(ptr: *mut u8) {
    // Implementation
}
```

### 6. Testing

Write tests for your code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_case() {
        let result = my_function(10);
        assert_eq!(result, 20);
    }

    #[test]
    fn test_edge_case() {
        let result = my_function(0);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_performance() {
        let start = std::time::Instant::now();
        for _ in 0..10000 {
            my_function(42);
        }
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 1000, "Performance regression detected");
    }
}
```

### 7. Commit Messages

Write clear, descriptive commit messages:

```
format: subject line (50 chars max)

Detailed explanation of what changed and why.
Wrap at 72 characters for readability.

If this fixes an issue:
Fixes #123

If this relates to performance:
Performance: +15% throughput in scheduler

If this includes breaking changes:
BREAKING CHANGE: Old API no longer supported
```

Good examples:
- `feat(scheduler): Add LSTM prediction model with 85% accuracy`
- `fix(memory): Prevent page fault on access to freed memory`
- `perf(telemetry): Reduce collection overhead from 3% to 0.8%`
- `docs(architecture): Update memory management design`

### 8. Pull Request

```bash
# Push your branch
git push origin feature/your-feature-name

# Open a PR on GitHub
# Title: Clear, descriptive
# Description: Why this change, what problem it solves
```

#### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] New feature
- [ ] Bug fix
- [ ] Performance improvement
- [ ] Documentation update

## Motivation
Why is this change needed? What problem does it solve?

## Testing
How did you test this change?

## Performance Impact
- Before: [metric]
- After: [metric]
- Change: [improvement]

## Checklist
- [ ] Code follows style guidelines
- [ ] Tests pass locally
- [ ] Documentation updated
- [ ] No performance regressions
- [ ] Benchmarks included (if performance-critical)
```

## Areas for Contribution

### High-Priority

1. **AI Scheduler** (`scheduler/src/lib.rs`)
   - LSTM model implementation
   - Process profiling
   - Prediction accuracy improvement
   - Status: Active development

2. **Memory System** (`memory/src/lib.rs`)
   - Predictive prefetching
   - Page replacement algorithm
   - NUMA support
   - Status: Core framework ready

3. **Security Engine** (`security/src/lib.rs`)
   - Anomaly detection models
   - Threat response automation
   - Hardware attestation
   - Status: Framework stage

4. **Telemetry** (`telemetry/src/lib.rs`)
   - Metrics collection
   - Dashboard integration
   - Performance analysis tools
   - Status: Infrastructure ready

### Medium-Priority

- Hardware abstraction layer improvements
- Boot sequence hardening
- Interrupt handling optimization
- Documentation and examples

### Lower-Priority (But Valuable)

- Visualization tools
- Tutorial content
- Community discussion facilitation
- Bug triage and documentation

## Review Process

1. **Automated Checks**
   - Tests pass
   - Code formatting valid
   - No security warnings
   - Performance benchmarks within budget

2. **Code Review**
   - At least one core maintainer reviews
   - Addresses engineering principles
   - Checks performance impact
   - Verifies security assumptions

3. **Approval**
   - Maintainer approves changes
   - CI/CD pipeline passes
   - PR is merged to main

## Questions?

- **Discord**: [Join our community]()
- **Discussions**: GitHub Discussions tab
- **Issues**: File issues for bugs/features
- **Email**: Contact maintainers for detailed questions

## Recognition

Contributors are recognized in:
- CONTRIBUTORS.md file
- Release notes
- Project documentation
- Community highlights

Thank you for building the future of operating systems! 🚀
