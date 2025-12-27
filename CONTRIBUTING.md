# Contributing to Orbis

First, thank you for your interest in contributing to Orbis. Whether you're reporting bugs, improving documentation, building plugins, or contributing core features, your participation helps us build a better platform for extensible desktop applications.

## Code of Conduct

This project adheres to the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/). By participating, you're expected to uphold this code. Please report unacceptable behavior to the maintainers.

## Ways to Contribute

### Report Bugs

Found a bug? Open an issue with:

- Clear title describing the issue
- Steps to reproduce
- Expected vs. actual behavior
- Your environment: OS, Rust version, relevant dependency versions
- Screenshots or error logs (if applicable)

Label it as `bug` so we can prioritize it.

### Suggest Features or Enhancements

Have an idea to improve Orbis? Open an issue with the `enhancement` label and include:

- Clear description of the feature
- Use case or problem it solves
- Potential implementation approach (optional but appreciated)
- Relevant examples or references

### Improve Documentation

Documentation improvements are always welcome:

- Fix typos or unclear explanations
- Add examples or tutorials
- Improve API documentation
- Add architecture diagrams or technical guides
- Translate existing docs

Documentation PRs don't require the same testing rigor as code changes, but clarity and accuracy are essential.

### Build Plugins

Orbis is designed for extensibility. Contributing plugins demonstrates real-world use cases and helps us refine the plugin API:

- Build a plugin solving a specific problem
- Share it in the Discord or as a reference implementation
- Contribute to the plugin examples directory
- Help document plugin development patterns

### Contribute Code

Code contributions are always appreciated. Start with:

- **Help wanted** — labeled `help-wanted` for tasks needing external input
- **Documentation** — labeled `documentation` for doc improvements

## Development Setup

Reference the official online documentation [here](https://orbis.cyberpath-hq.com/docs/getting-started/installation/) for updated installation and environment requirements.

## Making Changes

### Branch Naming

Use descriptive branch names:

- `feature/plugin-hot-reload` for new features
- `fix/wasm-memory-leak` for bug fixes
- `docs/plugin-api-guide` for documentation
- `refactor/plugin-loader-architecture` for refactoring

### Commit Messages

Write clear, atomic commits:

- Use present tense: "Add plugin hot-reload" not "Added plugin hot-reload"
- First line should be ≤50 characters
- Separate title from body with a blank line
- Reference issues when relevant: "Fixes #123"

Example:

```
Add WASM plugin timeout configuration

Plugins can now specify execution timeout via manifest.json.
Improves safety for long-running plugin operations.

Fixes #456
```

### Code Style

#### Rust

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` to format code: `cargo fmt --all`
- Run `cargo clippy` for linting: `cargo clippy --all -- -D warnings`
- Aim for meaningful error messages and comments on non-obvious logic

#### TypeScript/React

- Use consistent formatting (ESLint recommended)
- Follow React best practices and component composition patterns
- Type everything where possible

#### Documentation

- Use clear, concise language
- Include examples where helpful
- Link to related concepts
- Keep API docs up-to-date with code changes

## Submitting Pull Requests

### Before Submitting

1. **Create an issue** for major features (so we can discuss approach)
2. **Fetch latest changes**: `git fetch origin main`
3. **Test locally**: `cargo test --all` and build in release mode
4. **Format and lint**: `cargo fmt --all && cargo clippy --all`
5. **Update docs** if your change affects user-facing behavior

### PR Guidelines

1. **One feature per PR** — keep PRs focused and reviewable
2. **Descriptive title** — describe *what* and *why*, not just *what*
3. **Link related issues** — use "Fixes #123" or "Related to #456"
4. **Write a clear description**:
   - What problem does this solve?
   - How does it work (high-level)?
   - Any breaking changes or considerations?
   - Testing approach

5. **Keep commits clean** — we may squash on merge, but clean history helps during review

### What to Expect

- **Initial response**: Within 7 days
- **Review process**: Maintainers will review for correctness, safety, and alignment with project goals
- **Changes requested**: Be open to feedback; revisions are normal and valued
- **Merge**: Once approved, PRs are merged promptly

## Architecture and Plugin Development

### Plugin API

Plugins run in WASM sandboxes and communicate with the host via a defined API:

- See `orbis-plugin-api` crate for the public plugin interface
- Check `plugins/*` for reference implementations
- Documentation: [orbis.cyberpath-hq.com](https://orbis.cyberpath-hq.com/)

### Making Changes to Plugin API

The plugin API is the contract between host and plugins. Changes here are breaking and require:

- Detailed justification in the PR
- Consideration of backward compatibility
- Updated plugin examples
- Changelog entry

## Testing

### Unit Tests

```bash
cargo test --lib
```

### Integration Tests

```bash
cargo test --test '*'
```

### Writing Tests

- Test both happy paths and error conditions
- Use descriptive assertion messages
- Keep tests focused (one concern per test)
- Mock external dependencies when appropriate

### Testing Plugins

Plugin sandboxing means some behaviors are hard to test in isolation. Use:

- Unit tests for plugin code itself
- Integration tests for plugin-host interaction
- Manual testing for runtime behavior

## Documentation

### API Documentation

Update doc comments for public items:

```rust
/// Loads a plugin from the given path.
///
/// # Arguments
///
/// * `path` - Filesystem path to the plugin
///
/// # Returns
///
/// A loaded plugin or an error if loading fails
///
/// # Errors
///
/// Returns an error if the plugin is not a valid WASM module
pub fn load_plugin(path: &Path) -> Result<Plugin, Error>
```

Run `cargo doc --open` to preview generated docs.

### User Documentation

- Update [orbis.cyberpath-hq.com](https://orbis.cyberpath-hq.com/) for major features
- Keep README.md current with setup instructions
- Add examples demonstrating new functionality

## Community and Communication

### Discord

Join the [CyberPath Discord](https://discord.gg/WmPc56hYut) to:

- Ask questions and get help
- Discuss design decisions before major PRs
- Share plugin projects
- Connect with other contributors

### Issue Discussions

Use GitHub issues for:

- Bug reports and feature requests
- Design discussions for major changes
- Coordination on larger features

Keep discussions respectful and focused.

## Versioning

Orbis follows [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes (plugin API changes, runtime incompatibilities)
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes

The plugin API is considered part of the public interface.

## Licensing

By contributing, you agree that your contributions are licensed under the same license as Orbis. Check LICENSE for details.

## Recognition

Contributors are valued. We recognize contributions through:

- GitHub contributor graphs
- Mention in release notes for significant contributions
- Listing in a CONTRIBUTORS file (coming soon)

## Questions?

Can't find what you're looking for? Reach out:

- **Discord**: [CyberPath Community](https://discord.gg/WmPc56hYut)
- **GitHub Issues**: [Open an issue](https://github.com/cyberpath-HQ/orbis/issues)
- **GitHub Discussions**: For broader conversations

Thank you for helping make Orbis better. We're excited to build this with you.
