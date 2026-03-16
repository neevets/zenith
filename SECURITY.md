# Security Policy

## Supported Versions

We currently provide security patches and active support for the following versions:

| Version | Status | Support |
| --- | --- | --- |
| **v1.x** | **Maintenance** | Active |

---

## Reporting a Vulnerability

If you discover a vulnerability in the compiler or in the **Quantum Shield**, we want to know immediately. Please **do not open a public GitHub Issue**; instead, use our private reporting channel to protect users while we work on a fix.

**Contact:**

* **Email:** **security@zenith-lang.org**
* **Subject:** `[VULNERABILITY] report`

### What to include in your report:

1. **Detailed description:** What is happening and why it represents a risk.
2. **PoC:** `.zen` code or precise steps to reproduce the issue.
3. **Impact:** Does it allow RCE, middleware bypass, or file access?
4. **Suggestions:** If you have an idea on how to fix it in `transpiler.rs`, include it.

> **Our commitment:** We will respond to your report within **48 hours** and keep you informed throughout the entire patching process.

---

## Our Disclosure Philosophy

1. **Private Investigation:** We analyze the report internally to prevent the vulnerability from being exploited prematurely.
2. **Patch Development:** We create and test the fix in a private branch.
3. **Publication and Acknowledgment:** Once the update is released, we will publish a security advisory. If you wish, we will publicly credit you for helping strengthen Zenith.
