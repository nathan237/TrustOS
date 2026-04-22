# Security Policy

## Reporting a vulnerability

**Do not open a public GitHub issue for security problems.**

Please report privately via one of:

- GitHub **Security Advisories** → [Report a vulnerability](https://github.com/nathan237/TrustOS/security/advisories/new)
- Direct message to [@nathan237](https://github.com/nathan237) on GitHub

Please include:

1. A description of the issue and the affected component (kernel module, driver, userland, JARVIS subsystem).
2. Steps to reproduce — ideally a QEMU / VirtualBox setup or a specific board.
3. Impact assessment (privilege escalation, memory corruption, info disclosure, DoS).
4. Any suggested mitigation.

## Response targets

This is a solo project — best effort, no SLA. Realistic expectations:

| Severity | Acknowledgement | First fix attempt |
|----------|-----------------|-------------------|
| Critical (RCE, full kernel compromise) | within 72 h | within 2 weeks |
| High (privilege escalation, sandbox escape) | within 1 week | within 1 month |
| Medium / Low | within 2 weeks | next release cycle |

## Scope

In scope:
- Memory safety bugs in kernel code.
- Privilege boundary issues (Ring 3 → Ring 0).
- Network stack parsing bugs (`netstack/`).
- JARVIS guardian bypass.
- Boot-time trust issues.

Out of scope:
- Bugs requiring physical access to firmware / SPI flash.
- Findings that only apply to the `debugonly/` kernel.
- DoS via deliberately malformed input from a privileged shell user (the shell is trusted by design).

## Disclosure

Coordinated disclosure preferred. Once a fix lands and a release is cut, a credited advisory will be published.
