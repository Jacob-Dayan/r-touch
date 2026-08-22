## Running instructions for benchmarking and posting

> **Note**: This file is designed for AI agents to run and log the benchmarking process. For more information
> about the benchmarking process, refer to `Benchmarks/README.md`.

## RULES
* The project name is **R-touch** (or **r-touch**), but the CLI command is `rtouch`.
* Do not run commands unless explicitly instructed, except for `stat` when modifying `atime`/`mtime` to verify that the modification succeeded.
* Ignore `<ROOT_OF_R-TOUCH>` when navigating, as you are already working inside the repository's root directory.
* Run every `rtouch` command with the `--no-log` flag.
* Do not attempt to fetch internal documentation via `https://github.com/...` links. Instead, map links starting with `https://github.com/Jacob-Dayan/r-touch/blob/main/` directly to local files relative to the repository root.
* After each individual benchmark step, clean up by running:
  ```bash
  rm -rf <FILE_OR_DIRECTORY_TOUCHED>
```

## instructions
* After every competition in the benchmarking process, run:
```bash
$ # <SCORE_OF_R-TOUCH> - <SCORE_OF_TOUCH>: <ONE_THAT_HAS_MORE_POINTS> is leading  
```
And at the end run: 
```bash
$ # <WINNER> wins!
```

Also, run some time expressions with both that you know that touch doesn't support but R-touch does. You can find what formats R-touch
supports by looking at the [file where the times parser is defined](https://github.com/Jacob-Dayan/r-touch/blob/main/src/datetime/mod.rs)
And run:
```bash
$ # R-touch supports <FORMAT_USED>, touch throws an error for this
```

## Step-By-Step

 1. Build and Update
 Ensure r-touch is up to date:
 ```bash
	 sudo rm -f /usr/local/bin/rtouch &> /dev/null # Remove old binary
	 cd <ROOT_OF_R-TOUCH>
	 chmod 755 ./build/build_unix.sh
	 ./build/build_unix.sh
	 cd /dev/shm
	 sudo rm -rf ./* # Clear temporary benchmark directory
	 ls -A # Verify directory is empty
 ```
Verify that `rtouch --version` outputs the expected version number.

2. Executing
Benchmark both touch and rtouch on the following scenarios:

 - Changing both mtime and atime simultaneously (using `rtouch -d="..."`) across four timestamps: now, today at 10:00 PM, 1970-01-01 00:00, and 2055-06-01 00:00.
 - Creating empty files using:
  1. Standard alphabetic names.
  2. Numeric names (1234567890).
  3. All standard bash-compatible special characters allowed in filenames.
 - Updating mtime/atime on new empty directories.
 - Replacing directories with empty files:
  - Using touch: `rm -rf 'directory_name/' && touch 'directory_name'`
  - Using rtouch: `rtouch -rd -f directory_name/`
3. Logging Results

 1. Copy the full terminal output.
 2. Anonymize the shell prompt by replacing custom hostnames (e.g., `Jacob@Jacob-<computer_model>:`) with user@pc:. Example format:
    ```text
   	user@pc:/dev/shm$ echo "sample command"
   	```
 3. Save the formatted output to:
 `<ROOT_OF_R-TOUCH>/Benchmarks/results-<DD>-<MM>-<YY>.txt`
