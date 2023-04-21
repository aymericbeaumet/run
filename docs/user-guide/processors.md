Processors are systems that process the output of your commands. They are already embedded in Run.
See `run --help` for more information on how you can enable, disable or configure the processors.

> By default only the _Prefix_ processor is enabled. You can disable all processors by using the `--raw` flag. This could give a visible improvement in performance in some cases.

## Prefix

This processors prefixes stdout and stderr with the command name.

> This processors is enabled by default.

```bash
$ run 'echo foobar'
[echo] foobar
[echo] echo foobar terminated with status code 0
```

## OpenAI

You can enable the OpenAI processor to ask ChatGPT for feedback when your command fails. A prompt is sent with stderr every time it fails. Keep an eye on the usage, and I'd recommend to set [billing limits](https://platform.openai.com/account/billing/limits) just in case.

You need an active [OpenAI account](https://platform.openai.com/account) and an [API key](https://platform.openai.com/account/api-keys) for this to work.

````bash
$ run 'ls /tmp/missing'
[ls] ls: /tmp/missing: No such file or directory

+==================================[ OpenAI ]==================================+
|                                                                              |
| The issue is that the command "ls" is trying to list the contents of a       |
| directory that does not exist. "/tmp/missing" is not a valid directory path  |
| which is why the terminal is throwing an error message.                      |
|                                                                              |
| To fix this issue, we need to provide a valid directory path for the "ls"    |
| command to work with. Double-check the directory path to make sure it exists |
| and is spelled correctly. If it doesn't exist, we need to create it before   |
| running the "ls" command.                                                    |
|                                                                              |
| One command that might help fix this issue is "mkdir". This command is used  |
| to create a new directory. So, if we want to fix the issue by creating a new |
| directory, we could use the following command:                               |
|                                                                              |
| ```                                                                          |
| mkdir /tmp/newdirectory                                                      |
| ```                                                                          |
|                                                                              |
| This command will create a new directory called "newdirectory" within the    |
| "/tmp" directory. After creating the directory, we could then run the "ls"   |
| command to list the contents of the new directory:                           |
|                                                                              |
| ```                                                                          |
| ls /tmp/newdirectory                                                         |
| ```                                                                          |
|                                                                              |
+==============================================================================+

[ls] ls /tmp/missing terminated with status code 1
````
