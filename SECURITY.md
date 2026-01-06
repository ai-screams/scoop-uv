# 🍨 Security Policy

> Report security issues before they melt away — like ice cream in the sun!

---

## Supported Versions

| Version | Supported |
|---------|:---------:|
| 0.x.x (latest) | ✅ 🍨 |
| < 0.1.0 | ❌ |

> We only support the latest release. Please upgrade before reporting issues.

---

## Reporting a Vulnerability

🚨 **Found a security issue?**

**Please DO NOT open a public issue!**

Like leaving the freezer door open melts ice cream,
public disclosure of security issues puts users at risk.

### How to Report

1. **GitHub Security Advisory** (Recommended)
   → [Report a vulnerability](https://github.com/ai-screams/scoop-uv/security/advisories/new)

2. **Email**
   → Create a private security advisory on GitHub

### What to Include

- 🔍 Vulnerability description
- 📋 Steps to reproduce
- 💥 Impact assessment
- 💡 Suggested fix (if any)

---

## Response Timeline

| Stage | Expected Time |
|-------|:-------------:|
| 🍦 Acknowledgment | 48 hours |
| 🍨 Initial assessment | 7 days |
| 🧊 Fix release | Based on severity |

### Severity Levels

| Severity | Response |
|----------|----------|
| 🔴 Critical | Patch within 24-48 hours |
| 🟠 High | Patch within 7 days |
| 🟡 Medium | Next regular release |
| 🟢 Low | Scheduled update |

---

## Scope

### ✅ Security Issues

- Arbitrary code execution
- Path traversal vulnerabilities
- Privilege escalation
- Sensitive information disclosure
- Command injection via environment names

### ❌ Not Security Issues

| Issue Type | Where to Report |
|------------|-----------------|
| Feature bugs | [GitHub Issues](https://github.com/ai-screams/scoop-uv/issues) |
| Performance issues | [GitHub Issues](https://github.com/ai-screams/scoop-uv/issues) |
| Documentation errors | Pull Request welcome! |

---

## Security Best Practices

When using scoop:

```bash
# ✅ Good - environment names are validated
scoop create myproject 3.12

# ⚠️ Be cautious with untrusted .scoop-version files
# scoop validates names, but always review before entering untrusted directories
```

---

## Acknowledgments

We appreciate responsible disclosure. Security researchers who help keep scoop safe:

| Name | Contribution | Date |
|------|--------------|------|
| *Your name here* | *Be the first!* | - |

---

> 🍨 **Thank you for helping keep scoop secure!**
>
> *"A secure scoop is a happy scoop."*
