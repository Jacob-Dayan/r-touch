# AI Agent Instructions: Benchmarking & Output Generation

> **CRITICAL DIRECTIVE FOR THE AI AGENT:**  
> Read all constraints before executing. Any deviation from the output structure, any execution of intermediate scripts, or any inclusion of unapproved metadata will invalidate the entire benchmark run.

---

## 1. ABSOLUTE PROHIBITIONS (ZERO TOLERANCE)

1. **NO INTERMEDIATE SCRIPTS:**  
   Do **NOT** write, generate, or execute any shell scripts (e.g., `bench.sh`, `run.py`, `script.sh`) to automate this workflow. You must execute each CLI command directly in the shell one by one.
2. **NO SETUP / RECON COMMANDS IN OUTPUT:**  
   Never include commands like `whoami`, `pwd`, `ls`, `chmod`, `tree`, `git status`, `git diff`, `cd`, `sudo`, or compilation outputs inside the benchmark result log.
3. **NO AI COMMENTARY / CHATTER:**  
   Do **NOT** insert custom explanatory lines, debugging text, markdown headers (e.g., `## Step 1`, `## Now running...`), or narrative commentary into the log file.
4. **ALLOWED LOG CONTENTS ONLY:**  
   The output log file must contain **ONLY** and **EXCLUSIVELY**:
   - The exact benchmark execution lines (the shell prompt, the benchmarked command, and its direct execution output).
   - Verification `stat` commands (only when validating `atime`/`mtime` updates).
   - Explicit cleanup commands (`rm -rf ...`).
   - The specified `#` score / feature comment lines formatted strictly as defined below.
5. Do **NOT** modify default, toolchains, caching configuration, general configurations, and so on.

---

## 2. EXECUTION RULES

- **Binary Name:** The command-line binary is `rtouch`. Always append `--no-log` to all `rtouch` executions during benchmarks.
- **Path Resolution:** Do not resolve remote `github.com` URLs. Treat repository paths as relative to the local repository root.
- **Prompt Anonymization:** Every prompt in the final log file must be strictly formatted as:  
  `user@pc:/dev/shm$ <command>`
- **Post-Step Cleanup:** After each benchmark comparison, clean up the created target immediately:
  ```bash
  rm -rf <FILE_OR_DIRECTORY_TOUCHED>
  ```
- **Comment Conventions:**
  - After each benchmark scenario, log the score update using exact formatting:
    ```text
    user@pc:/dev/shm$ # <SCORE_RTOUCH> - <SCORE_TOUCH>: <LEADER> is leading
    ```
  - For features unique to `rtouch` (e.g., parent directory auto-creation or unsupported date formats in `touch`):
    ```text
    user@pc:/dev/shm$ # rtouch supports <FEATURE/FORMAT>, touch does not support this
    ```
  - At the very end of the benchmark log:
    ```text
    user@pc:/dev/shm$ # <WINNER> wins!
    ```

---

## 3. STEP-BY-STEP WORKFLOW

### Phase A: Setup & Compilation (DO NOT LOG THIS PHASE)
Run the following directly in the shell. **Do not write these commands or outputs to the benchmark log file**:
```bash
sudo rm -f /usr/local/bin/rtouch &> /dev/null
cd <ROOT_OF_R-TOUCH>
chmod 755 ./build/build_unix.sh
./build/build_unix.sh
cd /dev/shm
sudo rm -rf ./*
```

---

### Phase B: Benchmark Scenarios (LOG THIS PHASE VERBATIM)
Execute directly in `/dev/shm` and capture every line:

1. **Timestamp Modifications (`atime` & `mtime`):**
   - Benchmark `rtouch -d="..." --no-log <file>` vs `touch -d="..." <file>`.
   - Test across 4 timestamps: `now`, `today at 10:00 PM`, `1970-01-01 00:00`, `2055-06-01 00:00`.
   - Run `stat <file>` to verify timestamps.
   - Clean up using `rm -rf <file>`.

2. **Format Differences:**
   - Run custom date formats supported by `rtouch` (refer to `<ROOT>/src/datetime/mod.rs`) against `touch`.
   - Log the failure on `touch` and success on `rtouch`.

3. **Empty File Creation:**
   - Standard alphabetic filenames.
   - Numeric filenames (`1234567890`).
   - Filenames containing special characters valid in `bash`.
   - Clean up after each test.

4. **Parent Directory Creation:**
   - Compare `rtouch -p --no-log dir/file` vs `mkdir -p dir && touch dir/file`.
   - Clean up using `rm -rf dir`.

5. **Updating Directory Timestamps:**
   - Test `mtime`/`atime` updates on empty directories.
   - Verify with `stat`.
   - Clean up.

6. **Directory Replacement:**
   - `touch` method: `rm -rf 'test_dir/' && touch 'test_dir'`
   - `rtouch` method: `rtouch -rd -f --no-log test_dir/`
   - Clean up.

---

## 4. OUTPUT FILE DEFINITION

- **Destination Path:** `<ROOT_OF_R-TOUCH>/Benchmarks/results-<DD>-<MM>-<YY>.txt`
- **Format Verification Sample:** The output file must match the format below (no extra text, no markdown backticks inside the file, no shell logs from setup):

```text
user@pc:/dev/shm$ rtouch --no-log -d="now" test_file
user@pc:/dev/shm$ stat test_file
...
user@pc:/dev/shm$ rm -rf test_file
user@pc:/dev/shm$ touch -d="now" test_file
user@pc:/dev/shm$ stat test_file
...
user@pc:/dev/shm$ rm -rf test_file
user@pc:/dev/shm$ # 1 - 0: rtouch is leading
...
user@pc:/dev/shm$ # rtouch wins!
```

---

## 5. GIT COMMIT
Stage and commit **only** the newly generated results file:
```bash
git add Benchmarks/results-<DD>-<MM>-<YY>.txt
git commit -m "benchmarks: add results for <DD>-<MM>-<YY>"
```
and don't dare to push.
