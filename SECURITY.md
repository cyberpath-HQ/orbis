# Security Policy

## Supported Versions

Currently, only the latest version of the `v1.x.x` release series is supported with security updates.

| Version | Supported          |
| ------- | ------------------ |
| 1.x.x   | :white_check_mark: |
| < 1.0.0 | :x:                |

If you are using an older version, we strongly recommend upgrading to the latest `v1.x.x` release to receive security patches.

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability in Orbis, please report it responsibly and confidentially.

### How to Report

**Do not** open a public GitHub issue for security vulnerabilities. Instead, please report the vulnerability to:

**Email:** me@ebalo.xyz

Include the following information in your report:

- **Type of vulnerability** (e.g., memory safety, sandbox escape, privilege escalation)
- **Affected component** (e.g., core runtime, plugin API, UI system)
- **Affected version(s)** of Orbis
- **Description** of the vulnerability
- **Steps to reproduce** the issue (be as detailed as possible)
- **Proof of concept (PoC)** demonstrating the vulnerability
- **Impact assessment** (what an attacker could achieve)
- **Your contact information** (email, optionally PGP key)

### What to Expect

1. **Acknowledgment** – We will acknowledge receipt of your report within **5 business days**
2. **Assessment** – We will assess the report and confirm whether it qualifies as a security vulnerability
3. **Status Updates** – You will receive periodic updates on our progress (at least every 2 weeks)
4. **Patch Development** – We will develop and test a patch in a private setting
5. **Coordinated Disclosure** – We will work with you to coordinate public disclosure
6. **Public Release** – The fix will be released and publicly disclosed

### Embargo Period

We follow a **coordinated disclosure process** with a maximum embargo period of **90 days** from the time we receive your report. This allows us time to:

- Develop and test fixes
- Prepare security advisories
- Notify users and downstream projects

If 90 days is insufficient, we will discuss an extended timeline with you before proceeding.

## Disclosure Policy

Once a security vulnerability is confirmed and a patch is available:

1. We will publish a **GitHub Security Advisory** describing the vulnerability
2. A new patch release will be issued in the `v1.x.x` series
3. All users will be notified through:
   - GitHub Security Advisories
   - Release notes
   - Our project website and Discord channel

The advisory will include:

- Description of the vulnerability
- CVSS score (if applicable)
- Affected versions
- Patched version
- Workarounds (if applicable before upgrade)
- Credit to the reporter (unless anonymity is requested)

## Security Considerations

### Known Limitations

Orbis sandboxes WASM plugins through WebAssembly's security model. While this provides strong isolation guarantees, please be aware of the following:

- **Host-Plugin Communication:** The plugin API uses explicit message passing. Ensure your plugins follow the documented API contract
- **Resource Limits:** Plugins can consume CPU and memory. The host runtime enforces some limits, but CPU-intensive plugins may impact responsiveness
- **File System Access:** Plugins do not have direct file system access. All I/O must go through the host-provided API
- **Network Access:** Plugins cannot make direct network requests. All network communication is handled by the host

### Security Testing

We appreciate security research on Orbis. If you're conducting authorized security testing:

- **Scope:** Testing should be limited to Orbis itself, not the infrastructure running it
- **Good Faith:** We will not pursue legal action against researchers acting in good faith
- **No DoS/Disruption:** Do not conduct tests that could disrupt service or compromise data integrity
- **Respect Privacy:** Do not access, modify, or exfiltrate data beyond what's needed to demonstrate the vulnerability

## Safe Harbor

We acknowledge that researchers may discover security issues while conducting authorized testing. We will not pursue legal action against individuals who:

- Discover and report vulnerabilities responsibly
- Act in good faith
- Do not access, modify, or exfiltrate data
- Follow our disclosure policy

## Code of Conduct

All security reports and discussions are handled with professionalism and respect. We are committed to:

- Taking all reports seriously
- Treating reporters with respect and dignity
- Protecting reporter privacy and anonymity (if requested)
- Providing regular communication throughout the process
- Giving credit to researchers (unless they request anonymity)

## Recognition

We recognize and appreciate security researchers who help improve Orbis. With your permission, we will:

- Thank you in the GitHub Security Advisory
- List you in our contributors documentation
- Provide a special mention in release notes

If you prefer to remain anonymous, we fully respect that choice.

## Related Resources

- [GitHub Security Advisories](https://docs.github.com/en/code-security/security-advisories)
- [Coordinated Vulnerability Disclosure](https://github.blog/security/vulnerability-research/coordinated-vulnerability-disclosure-cvd-open-source-projects/)
- [OWASP Responsible Disclosure](https://owasp.org/www-community/Responsible_Disclosure)
- [ISO/IEC 29147:2018 Vulnerability Disclosure](https://www.iso.org/standard/72675.html)

## Questions?

If you have questions about this security policy, please reach out to the maintainers:

- **Discord:** [CyberPath Community](https://discord.gg/WmPc56hYut)

Thank you for helping keep Orbis and our community secure.
