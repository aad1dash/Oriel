# Security policy

Oriel is an early local tool. Security reports are welcome, especially where untrusted source input, external process execution, local cache data or MCP requests could cross an intended boundary.

## Supported version

Security fixes are made on the latest `main` branch and the latest tagged release. Older versions may not receive backports while the project is in v0.x.

## Report a vulnerability

Please do not open a public issue for a suspected vulnerability.

Use GitHub's private vulnerability-reporting form for this repository when it is available. Include:

- the affected version or commit;
- the operating system and relevant tool versions;
- the expected and observed behaviour;
- the smallest safe reproduction you can provide;
- the impact you believe is possible;
- any suggested mitigation.

Do not include credentials, private source material, downloaded media or another person's data. If the private reporting form is unavailable, open a minimal public issue asking for a private maintainer contact route without disclosing the vulnerability.

The project will acknowledge a report when it is seen, investigate it in good faith and coordinate disclosure after a fix is available. As a small early project, it cannot promise a fixed response time or a bug bounty.

## Scope and boundaries

Useful reports include path traversal, command or argument injection, unsafe cache handling, unintended data retention, denial of service beyond documented limits, and MCP input or output boundary failures.

Oriel deliberately invokes `yt-dlp` as an external provider and accesses YouTube on a cold acquisition or explicit refresh. Upstream availability changes, caption errors and ordinary `yt-dlp` extraction failures are generally not Oriel vulnerabilities unless Oriel turns them into a security boundary failure.

Oriel does not bundle or redistribute transcripts or media. Users remain responsible for source access and use.
