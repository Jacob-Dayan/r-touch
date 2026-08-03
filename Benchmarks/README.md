# Benchmarks

> **Note:** Tests were run using the `--no-log` flag to ensure a fair comparison, as `rtouch` enables file creation & error logging by default, whereas GNU `touch` has no built-in logging.

> **Benchmark Environments:** `Windows Subsystem for Linux (WSL2 / Ubuntu)` for _Linux_ benchmarks, and `Windows 11 Home` for _Windows_ benchmarks.

### Benchmark Breakdown
1. **Standard Alphabet:** Basic file creation using standard ASCII characters.
2. **Alphanumeric:** File creation with numbers and letters.
3. **Special Characters:** Testing resilience and behavior with symbols and special characters.
4. **Nested File Creation (`--parents`):** Attempting to create a file inside a non-existent directory structure.

### Why `rtouch` Wins on Developer Experience
Setting up a new project workspace usually requires chaining multiple commands:
```bash
mkdir -p .cargo && touch .cargo/config.toml
```

With rtouch, the parent directory creation is natively handled in a single atomic step:

```Bash
rtouch -p .cargo/config.toml
```
> **Note:** R-touch automatically replaces the `/` with a `\` character on windows workplaces, with zero-cost of overhead and performance.

This makes `rtouch` both more efficient and significantly more practical for everyday workflow automation.
