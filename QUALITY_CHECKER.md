# Quality Checker for Pull Requests

This repository includes automated quality checks that run on all pull requests to ensure code quality and naming convention compliance.

## What Gets Checked

### Rust Quality Checks
- **Code Formatting**: Ensures code follows standard Rust formatting (via `cargo fmt`)
- **Linting**: Runs Clippy to catch common mistakes and code smells
- **Naming Conventions** (enforced by Clippy):
  - Variables and functions: `snake_case` ✅
  - Structs and enums: `CamelCase` (PascalCase) ✅
  - Constants: `SCREAMING_SNAKE_CASE` ✅

### Flutter/Dart Quality Checks
- **Code Analysis**: Runs Flutter analyzer to catch issues (via `flutter analyze`)
- **Code Formatting**: Ensures code follows Dart formatting standards (via `dart format`)
- **Naming Conventions** (enforced by analyzer):
  - Classes, enums, typedefs: `UpperCamelCase` ✅
  - Variables and functions: `lowerCamelCase` ✅
  - Libraries and files: `lowercase_with_underscores` ✅
  - Constants: `lowerCamelCase` or `SCREAMING_CAPS` ✅

## How It Works

The quality checker runs automatically on every pull request through GitHub Actions. The workflow:

1. Detects which languages are used in your changes (Rust and/or Flutter)
2. Sets up the appropriate toolchains
3. Runs linting and formatting checks
4. Reports any violations as check failures

## Running Locally

Before submitting a pull request, you can run the checks locally:

### Rust
```bash
# Format your code
cargo fmt --all

# Run Clippy
cargo clippy --all-targets --all-features -- -D warnings -D clippy::all -W clippy::pedantic
```

### Flutter/Dart
```bash
# Get dependencies
flutter pub get

# Format your code
dart format .

# Run analyzer
flutter analyze --fatal-infos
```

## Configuration Files

- `.clippy.toml` - Clippy configuration for Rust
- `analysis_options.yaml` - Flutter/Dart analyzer configuration
- `.github/workflows/quality-check.yml` - GitHub Actions workflow

## Fixing Issues

If the quality check fails:

1. Review the error messages in the GitHub Actions log
2. Fix the naming convention violations or linting issues
3. Run the checks locally to verify fixes
4. Push your changes - the checks will run again automatically

## Examples

### ✅ Good Rust Naming
```rust
struct GameCard { }          // CamelCase for structs
enum CardSuit { }            // CamelCase for enums
fn calculate_score() { }     // snake_case for functions
let total_points = 0;        // snake_case for variables
```

### ❌ Bad Rust Naming
```rust
struct gameCard { }          // Should be GameCard
enum card_suit { }           // Should be CardSuit
fn CalculateScore() { }      // Should be calculate_score
let TotalPoints = 0;         // Should be total_points
```

### ✅ Good Dart Naming
```dart
class GameCard { }           // UpperCamelCase for classes
enum CardSuit { }            // UpperCamelCase for enums
void calculateScore() { }    // lowerCamelCase for functions
int totalPoints = 0;         // lowerCamelCase for variables
```

### ❌ Bad Dart Naming
```dart
class gameCard { }           // Should be GameCard
enum card_suit { }           // Should be CardSuit
void CalculateScore() { }    // Should be calculateScore
int TotalPoints = 0;         // Should be totalPoints
```
