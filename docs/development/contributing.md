# Contributing to Nexus Security

Thank you for your interest in contributing to Nexus Security! This guide will help you get started.

## Getting Started

### 1. Fork and Clone

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/YOUR_USERNAME/nexus-security.git
cd nexus-security
git remote add upstream https://github.com/nexus-security/nexus-security.git
```

### 2. Set Up Development Environment

Follow the [Local Setup Guide](../deployment/local-setup.md) to set up your development environment.

### 3. Create a Branch

```bash
git checkout -b feature/your-feature-name
```

## Development Workflow

### 1. Make Changes

- Follow the [Code Style Guide](code-style.md)
- Write tests for new functionality
- Update documentation as needed

### 2. Test Your Changes

```bash
# Backend tests
cd backend
cargo test

# Frontend tests
cd frontend
npm test

# Linting
cargo clippy
npm run lint
```

### 3. Commit Your Changes

Follow conventional commit format:

```bash
git add .
git commit -m "feat(component): add new feature description"
```

### 4. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub.

## Pull Request Guidelines

### PR Title

Use conventional commit format:

```
feat(analysis): add PE header parsing
fix(bounty): correct expiry validation
docs(api): update endpoint documentation
```

### PR Description

Include:

- Summary of changes
- Motivation and context
- How to test
- Screenshots (for UI changes)

### PR Template

```markdown
## Summary
Brief description of changes.

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## How Has This Been Tested?
Describe tests run.

## Checklist
- [ ] Code follows style guidelines
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] No breaking changes (or documented)
```

## Code Review Process

1. **Automated Checks**: CI must pass (tests, linting, security scan)
2. **Review**: At least one maintainer approval required
3. **Changes**: Address reviewer feedback
4. **Merge**: Maintainer merges when approved

## Types of Contributions

### Bug Reports

- Use the bug report template
- Include steps to reproduce
- Provide system information
- Attach relevant logs

### Feature Requests

- Use the feature request template
- Explain the use case
- Describe proposed solution
- Consider alternatives

### Code Contributions

- Bug fixes
- New features
- Performance improvements
- Test coverage
- Documentation

### Documentation

- Fix typos
- Improve clarity
- Add examples
- Translate content

## Architecture Decisions

For significant changes, create an Architecture Decision Record (ADR):

```markdown
# ADR-001: Use YARA for signature detection

## Status
Accepted

## Context
Need efficient pattern matching for malware signatures.

## Decision
Use YARA rules engine for signature-based detection.

## Consequences
- Fast pattern matching
- Industry-standard rule format
- Learning curve for rule writing
```

## Security Vulnerabilities

**Do NOT open public issues for security vulnerabilities.**

Email security@nexus-security.com with:

- Description of vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

## Community Guidelines

### Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Help others learn
- Focus on the issue, not the person

### Communication

- GitHub Issues: Bug reports, feature requests
- GitHub Discussions: Questions, ideas
- Discord: Real-time chat

## Recognition

Contributors are recognized in:

- CONTRIBUTORS.md file
- Release notes
- Project README

## Questions?

- Check existing documentation
- Search closed issues
- Ask in GitHub Discussions
- Join our Discord community

Thank you for contributing to Nexus Security!
