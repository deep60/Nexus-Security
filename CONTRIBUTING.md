# Contributing to Nexus-Security

Thank you for your interest in contributing to Nexus-Security! We welcome contributions from the community.

## How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported in [Issues](https://github.com/your-org/nexus-security/issues)
2. If not, create a new issue with:
   - Clear description of the bug
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment details (OS, versions, etc.)

### Suggesting Features

1. Open an issue with the `enhancement` label
2. Describe the feature and its use case
3. Explain why it would benefit the project

### Code Contributions

#### Setup Development Environment

```bash
# Clone the repository
git clone https://github.com/your-org/nexus-security.git
cd nexus-security

# Install dependencies
cd backend && cargo build
cd ../frontend && npm install
cd ../blockchain && npm install

# Set up local environment
cp .env.example .env
docker-compose up -d
```

#### Making Changes

1. **Fork the repository**
2. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes**
   - Follow the coding standards (see below)
   - Write tests for new functionality
   - Update documentation as needed

4. **Test your changes**
   ```bash
   # Backend tests
   cd backend && cargo test

   # Frontend tests
   cd frontend && npm test

   # Smart contract tests
   cd blockchain && npx hardhat test
   ```

5. **Commit your changes**
   ```bash
   git commit -m "feat: add new feature description"
   ```

   Follow [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat:` New feature
   - `fix:` Bug fix
   - `docs:` Documentation changes
   - `test:` Adding tests
   - `refactor:` Code refactoring
   - `chore:` Maintenance tasks

6. **Push to your fork**
   ```bash
   git push origin feature/your-feature-name
   ```

7. **Create a Pull Request**
   - Provide a clear description
   - Reference any related issues
   - Ensure CI passes

## Coding Standards

### Rust (Backend)

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Write documentation for public APIs
- Maintain test coverage above 80%

### TypeScript (Frontend & Blockchain)

- Follow the existing ESLint configuration
- Use TypeScript strict mode
- Write unit tests for components and functions
- Use meaningful variable and function names

### Solidity (Smart Contracts)

- Follow [Solidity Style Guide](https://docs.soliditylang.org/en/latest/style-guide.html)
- Write comprehensive tests
- Document all public functions with NatSpec
- Perform security audits for critical changes

## Code Review Process

1. All submissions require review from at least one maintainer
2. Address review feedback promptly
3. Maintain a respectful and constructive dialogue
4. Once approved, a maintainer will merge your PR

## Community Guidelines

- Be respectful and inclusive
- Welcome newcomers
- Provide constructive feedback
- Follow our [Code of Conduct](CODE_OF_CONDUCT.md)

## Getting Help

- Join our [Discord](https://discord.gg/your-invite) for discussions
- Check the [documentation](docs/)
- Ask questions in GitHub Discussions

## Recognition

Contributors will be recognized in:
- README.md contributors section
- Release notes
- Project website (if applicable)

Thank you for helping make Nexus-Security better!
