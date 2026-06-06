# Security Policy

## Supported Versions

We actively support the latest minor version of statsd-mock with security updates.

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | :white_check_mark: |
| 0.1.x   | :x:                |

## Security Scope

**Important**: statsd-mock is a **testing utility** designed for use in **development and test environments only**. It is **NOT intended for production use**.

### Expected Usage

- ✅ Unit tests
- ✅ Integration tests
- ✅ Local development
- ✅ CI/CD pipelines
- ✅ Test fixtures

### NOT Intended For

- ❌ Production environments
- ❌ Handling real user data
- ❌ Internet-facing services
- ❌ Security-sensitive applications
- ❌ Long-running services

## Security Considerations

### Network Binding

The mock server binds to `127.0.0.1` (localhost) only, which prevents:
- External network access
- Exposure to the internet
- Cross-machine communication

This is by design for security.

### UDP Protocol

The server uses UDP (connectionless protocol):
- No authentication or encryption
- Packets can be spoofed (in test environment this is acceptable)
- No message delivery guarantees

This is acceptable for a testing mock.

### Resource Limits

The server has built-in safety limits:
- 2-second maximum wait time prevents infinite blocking
- 50ms quiet period prevents unnecessary waiting
- Fixed 1500-byte buffer size (MTU) prevents memory exhaustion
- Thread-based architecture with automatic cleanup

### Input Validation

- Packet parsing is tolerant of malformed data
- Parse errors are gracefully handled (logged, not panicked)
- Invalid UTF-8 is rejected safely
- No code execution from packet data

## Reporting a Vulnerability

If you discover a security vulnerability in statsd-mock, please report it by:

1. **DO NOT** open a public GitHub issue
2. Email the maintainer directly at: me@duyet.net
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if available)

We will respond within 72 hours and work with you to:
- Understand the issue
- Develop a fix
- Release a patched version
- Credit you for the discovery (if desired)

## Security Best Practices

When using statsd-mock in your tests:

### ✅ DO

- Use only in test code
- Bind to localhost (default behavior)
- Use short-lived instances (default: consumed after use)
- Keep test data non-sensitive
- Run in isolated test environments

### ❌ DON'T

- Use in production code
- Bind to public interfaces
- Store real user data in test metrics
- Expose to untrusted networks
- Use for performance-critical code paths

## Dependencies

statsd-mock has minimal dependencies:

- `itertools` (0.10) - Used only for string joining
  - Well-maintained, widely-used crate
  - No known security vulnerabilities

All dependencies are monitored and kept up-to-date.

## Audit Trail

Security-relevant changes are tracked in git commit messages using conventional commits:

- `security:` prefix for security fixes
- `fix:` for bug fixes that may have security implications
- All changes are reviewed before merging

## Compliance

As a testing utility:
- No data persistence
- No external network calls
- No credential handling
- No PII (Personally Identifiable Information) processing
- Runs in-process only

Therefore, statsd-mock has minimal compliance requirements. However:

- Code is open source (MIT license)
- All changes are publicly auditable
- No telemetry or analytics
- No third-party service dependencies

## Contact

- Maintainer: Duyet Le
- Email: me@duyet.net
- GitHub: https://github.com/duyet/statsd-mock-rs

---

**Remember**: This is a testing library. Never use test utilities in production!
