# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| latest on `main` | Yes |
| older releases | No |

## Reporting a Vulnerability

If you discover a security vulnerability in QuartzDB, please report it responsibly.

**Do not open a public GitHub issue for security vulnerabilities.**

Instead, email **security@quartzdb.io** with:

1. A description of the vulnerability.
2. Steps to reproduce.
3. The potential impact.
4. Any suggested fix (optional).

You will receive an acknowledgement within **48 hours** and a detailed response within **5 business days** indicating next steps.

## Scope

The following are in scope:

- The QuartzDB Rust backend (`quartz-faas/`).
- API authentication and authorisation logic.
- Stripe webhook signature verification.
- Data isolation between tenants.
- The Next.js dashboard (`quartz-dashboard/`).

Out of scope:

- Cloudflare infrastructure itself (report to [Cloudflare](https://www.cloudflare.com/disclosure/)).
- Third-party dependencies (report upstream; we monitor advisories via `cargo-deny`).

## Security Practices

- All API keys use the `qdb_` prefix and are validated via constant-time comparison.
- Stripe webhooks are verified with HMAC-SHA256 signature checks.
- No `unsafe` code in the codebase (the single previous use has been replaced with safe alternatives).
- `#![deny(warnings)]` and `#![warn(clippy::pedantic)]` are enforced at the crate level.
- CI runs `cargo-deny` (license + advisory audit) and `cargo audit` on every push.
- Dependencies are kept minimal; unused crates are removed from the workspace.

## Disclosure Timeline

- **Day 0**: Report received.
- **Day 1-2**: Acknowledgement sent.
- **Day 3-5**: Triage and impact assessment.
- **Day 6-30**: Fix developed, tested, and deployed.
- **Day 30+**: Public disclosure coordinated with reporter.
