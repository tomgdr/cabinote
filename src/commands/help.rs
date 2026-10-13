// ヘルプの文言(英語)。

pub const HELP_TEXT: &str = "\
cn - a small note manager for the shell

Usage:
  cn <command> [options]

Commands:
  cabinet, c    Show the cabinet (storage) path
  drawer, d     Manage drawers
  memo, m       Manage memos in the current drawer

cn drawer:
  cn drawer                  Show the current drawer
  cn drawer -l               List drawers
  cn drawer -a <name>        Add a drawer
  cn drawer -c <name>        Switch to a drawer by name
  cn drawer -cn <number>     Switch to a drawer by number
  cn drawer -r <new name>    Rename the current drawer
  cn drawer -d <name>        Delete a drawer by name
  cn drawer -dn <number>     Delete a drawer by number

cn memo:
  cn memo <title>            Open a memo by title
  cn memo -n <number>        Open a memo by number
  cn memo -l                 List memos
  cn memo -a                 Create a new memo and open it
  cn memo -e                 Show the editor used to open memos
  cn memo -d <title>         Delete a memo by title
  cn memo -dn <number>       Delete a memo by number

Numbers start at 1 and match the numbers shown by the list options.

Options:
  -h, --help       Show this help
  -V, --version    Show the version";
