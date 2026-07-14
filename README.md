# r-touch
The modern replacement for "touch" bash command
setting an alias is reccomended

# bin
you can download the compiled version from the releases section, but I highly reccomend compiling it yourself.

# how to use
first, create an alias (I recommend "rtouch") to the path of the binary. 
run rtouch /path/to/file
then try /path/to/a/file/that/has/no/parent/directory/example_file.txt -p
the "-p" argument makes a parent directory for your file and then creating it in it.
In default, the program is logging your actions (locally on your PC, not to a server). if you want to secretly create files, without them getting into the logs, tou can run:
rtouch /secret_files/secret_file.txt --no-log
the argument "--no-log" will make sure that the file will not be logged. it also works on directories:
rtouch /secrets/new_folder/new_secret_file.txt -p --no-log
and NONE parent-directory or file creating will be logged.

# NOTICE HERE WINDOWS USERS!!!
the binary file in the release section is a Linux-exeutable file (ELF) not an EXE, tho the program is fully Cross-platform. you'd have to compile the code yourself.
