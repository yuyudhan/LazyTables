# 007 - Contributing

This document provides guidelines for contributing to LazyTables, from reporting issues to submitting code changes.

## How to Contribute

We welcome all types of contributions:

- 🐛 **Bug Reports**: Help us identify and fix issues
- 💡 **Feature Requests**: Suggest new functionality
- 📖 **Documentation**: Improve guides and documentation
- 🔧 **Code Contributions**: Implement features and bug fixes
- 🧪 **Testing**: Add or improve test coverage
- 🎨 **UI/UX**: Enhance the user interface and experience

## Getting Started

### 1. Development Setup

Follow the [Getting Started guide](001-getting-started.md) to set up your development environment.

### 2. Understanding the Codebase

- Read the [Architecture overview](002-architecture.md)
- Review [Project Structure](004-project-structure.md) 
- Understand [Database Support](005-database-support.md)
- Check out [Testing guidelines](006-testing.md)

### 3. Finding Something to Work On

- Browse [GitHub Issues](https://github.com/yuyudhan/LazyTables/issues) labeled `good first issue`
- Look for issues labeled `help wanted`
- Check the [PRD.md](../../PRD.md) for planned features
- Join [GitHub Discussions](https://github.com/yuyudhan/LazyTables/discussions) for ideas

## Contribution Workflow

### 1. Fork and Clone

```bash
# Fork the repository on GitHub, then clone your fork
git clone git@github.com:YOUR_USERNAME/LazyTables.git
cd LazyTables

# Add the original repository as upstream
git remote add upstream git@github.com:yuyudhan/LazyTables.git
```

### 2. Create a Branch

```bash
# Create and switch to a new branch
git checkout -b feature/your-feature-name

# Or for bug fixes
git checkout -b bugfix/issue-description

# Or for documentation
git checkout -b docs/improvement-description
```

### 3. Make Changes

- Follow the [Coding Standards](#coding-standards)
- Write or update tests for your changes
- Update documentation as needed
- Test your changes locally

### 4. Commit Changes

```bash
# Stage your changes
git add .

# Commit with a descriptive message
git commit -m "feat: add connection timeout configuration

- Add timeout field to ConnectionConfig
- Implement timeout handling in database adapters
- Add tests for timeout scenarios
- Update documentation with timeout examples"
```

### 5. Push and Create Pull Request

```bash
# Push your branch to your fork
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub with:
- Clear title describing the change
- Detailed description of what you've implemented
- Reference to any related issues
- Screenshots for UI changes

### 6. Respond to Review

- Address reviewer feedback promptly
- Push additional commits to the same branch
- Ask questions if feedback is unclear
- Be open to suggestions and improvements

## Coding Standards

### Rust Code Style

```rust
// Use rustfmt for formatting
cargo fmt

// Pass all clippy lints
cargo clippy -- -D warnings

// Follow Rust naming conventions
pub struct ConnectionManager { ... }  // PascalCase for types
pub fn create_connection() { ... }    // snake_case for functions
const MAX_CONNECTIONS: usize = 10;    // SCREAMING_SNAKE_CASE for constants
```

### Code Quality Guidelines

1. **Error Handling**: Use `Result<T, E>` and proper error types
   ```rust
   // Good
   pub fn connect_database(config: &Config) -> Result<Connection, DatabaseError> {
       // Implementation
   }
   
   // Avoid
   pub fn connect_database(config: &Config) -> Connection {
       // May panic on error
   }
   ```

2. **Documentation**: Document public APIs
   ```rust
   /// Creates a new database connection using the provided configuration.
   /// 
   /// # Arguments
   /// 
   /// * `config` - Database connection configuration
   /// 
   /// # Returns
   /// 
   /// A `Result` containing the connection or an error
   /// 
   /// # Examples
   /// 
   /// ```rust
   /// let config = ConnectionConfig::new("localhost", 5432);
   /// let conn = create_connection(&config)?;
   /// ```
   pub fn create_connection(config: &ConnectionConfig) -> Result<Connection, Error> {
       // Implementation
   }
   ```

3. **Testing**: Write tests for new functionality
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_connection_creation() {
           let config = ConnectionConfig::default();
           let result = create_connection(&config);
           assert!(result.is_ok());
       }
   }
   ```

### Performance Guidelines

1. **Async Operations**: Use async/await for I/O operations
   ```rust
   pub async fn execute_query(&self, query: &str) -> Result<QueryResult, Error> {
       let result = self.connection.query(query).await?;
       Ok(result.into())
   }
   ```

2. **Memory Efficiency**: Avoid unnecessary allocations
   ```rust
   // Good: Use string slices when possible
   pub fn format_table_name(schema: &str, table: &str) -> String {
       format!("{}.{}", schema, table)
   }
   
   // Good: Use iterators instead of collecting
   pub fn filter_tables(tables: &[Table]) -> impl Iterator<Item = &Table> {
       tables.iter().filter(|t| t.visible)
   }
   ```

3. **Database Operations**: Optimize queries and connections
   ```rust
   // Use connection pooling
   pub struct DatabasePool {
       pool: Arc<Pool<PostgresConnectionManager>>,
   }
   
   // Use prepared statements for repeated queries
   pub async fn get_tables(&self, schema: &str) -> Result<Vec<Table>, Error> {
       let stmt = self.prepare("SELECT * FROM tables WHERE schema = $1").await?;
       let rows = self.query(&stmt, &[&schema]).await?;
       Ok(rows.into_iter().map(Table::from).collect())
   }
   ```

## Commit Message Guidelines

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

### Format
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Build process, dependencies, etc.
- `perf`: Performance improvements

### Examples
```bash
# Feature addition
feat(database): add MySQL adapter support

# Bug fix
fix(ui): prevent crash when no connections configured

# Documentation
docs(api): add examples for connection configuration

# Breaking change
feat(config)!: change connection format to support multiple databases

BREAKING CHANGE: Connection configuration now requires database_type field
```

## Pull Request Guidelines

### PR Title
Use the same format as commit messages:
```
feat(scope): brief description of the change
```

### PR Description Template
```markdown
## Description
Brief description of the changes made.

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing completed

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review of code completed
- [ ] Code is properly commented
- [ ] Documentation updated if needed
- [ ] Tests pass locally

## Related Issues
Fixes #123
Closes #456
```

### PR Review Process

1. **Automated Checks**: All CI checks must pass
   - Code formatting (`cargo fmt --check`)
   - Linting (`cargo clippy`)
   - Tests (`cargo test`)

2. **Code Review**: At least one maintainer review required
   - Code quality and standards
   - Test coverage
   - Documentation updates
   - Performance implications

3. **Manual Testing**: For UI changes or new features
   - Test locally with different database types
   - Verify keyboard navigation works
   - Check visual elements render correctly

## Issue Guidelines

### Bug Reports

Use the bug report template:

```markdown
**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Go to '...'
2. Click on '....'
3. See error

**Expected behavior**
What you expected to happen.

**Screenshots**
If applicable, add screenshots.

**Environment:**
- OS: [e.g. macOS 13.0]
- LazyTables Version: [e.g. 0.1.0]
- Database: [e.g. PostgreSQL 15]
- Terminal: [e.g. iTerm2]

**Additional context**
Any other context about the problem.
```

### Feature Requests

Use the feature request template:

```markdown
**Is your feature request related to a problem?**
A clear description of what the problem is.

**Describe the solution you'd like**
A clear description of what you want to happen.

**Describe alternatives you've considered**
Alternative solutions or features you've considered.

**Additional context**
Any other context, screenshots, or examples.
```

## Documentation Guidelines

### Writing Style

- **Clear and Concise**: Use simple, direct language
- **Examples**: Include code examples for technical concepts
- **Structure**: Use headers, lists, and code blocks for readability
- **Audience**: Write for developers familiar with databases and terminals

### Documentation Types

1. **User Documentation**: README.md and user guides
2. **Developer Documentation**: This docs/dev/ directory
3. **API Documentation**: Rust doc comments (`///`)
4. **Code Comments**: Explain complex logic, not obvious code

### Updating Documentation

When making changes that affect:
- **User Interface**: Update README.md with new features/shortcuts
- **Configuration**: Update config examples and explanations  
- **Database Support**: Update database compatibility information
- **Architecture**: Update relevant docs/dev/ files

## Community Guidelines

### Code of Conduct

We are committed to providing a welcoming and inclusive experience for everyone. Please:

- **Be respectful** in all interactions
- **Be constructive** when providing feedback
- **Be patient** with newcomers
- **Be collaborative** in problem-solving

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and ideas
- **Pull Request Reviews**: Technical discussions about code
- **Discord** (if available): Real-time community chat

### Recognition

Contributors are recognized in:
- **CONTRIBUTORS.md** file (planned)
- **Release notes** for significant contributions
- **GitHub contributor graphs**
- **Special mentions** in documentation

## Advanced Contributing

### Becoming a Maintainer

Regular contributors may be invited to become maintainers with:
- **Code review permissions**
- **Issue triage responsibilities** 
- **Release planning participation**
- **Community moderation duties**

Maintainers are expected to:
- Review PRs promptly and constructively
- Help newcomers get started
- Participate in architectural decisions
- Maintain code quality standards

### Release Process

1. **Version Planning**: Follow semantic versioning
2. **Feature Freeze**: Stop accepting new features
3. **Release Testing**: Comprehensive testing across platforms
4. **Documentation Updates**: Update changelogs and docs
5. **Release Announcement**: Communicate changes to community

Thank you for contributing to LazyTables! Your efforts help make database management better for developers everywhere. 🚀