# cabinote

*A lazy person's note manager for the shell*

キャビネットに放り込んで、欲しいときに引き出す。
面倒くさがりのための、シェルで完結するメモ管理ツール。

- メモは、Markdownファイルとしてローカルに保存されます。
- メモは「cabinet(保存先)」の中の「drawer(引き出し)」に整理します。

保存先の構成:

```
<cabinet>/
├─ home/
│   └─ yyyymmdd_hhmmss.md
└─ <drawer>/
    └─ <memo>.md
```

## 対応環境

- Linux
- Windows

### 対応エディタ

- vi
- vim
- nvim
- nano
- notepad
- zed
- code


> [!WARNING] notepadは、`.md` を編集すると `#` を `\#` に書き換えることがあります。**`.md` の編集には、notepad以外のエディタをおすすめします**(notepadの書式設定を変更することで避けることができます)。

## インストール

Rustの開発環境(`cargo`)が必要です。

```
cargo install --git https://github.com/tomgdr/cabinote --locked
```

- 更新するときは、同じコマンドに `--force` を付けて、もう一度実行します。
- アンインストールは `cargo uninstall cabinote` です。
- 依存クレート(`chrono`、`yaml-rust2`)は、自動で取得され、実行ファイルに含まれます。

ソースから自分でビルドする場合:

```
git clone https://github.com/tomgdr/cabinote
cd cabinote
cargo build --release
```

できた実行ファイル(`target/release/cn`、Windowsは `cn.exe`)を、PATHの通ったディレクトリに置いてください。ビルドはOSごとに必要です。

## 初回セットアップ

`cn` は、ディレクトリを自動では作りません。最初に、次を作ってください。

Linux / Mac:

```
mkdir -p ~/.cn/home
```

Windows(PowerShell):

```
mkdir $env:USERPROFILE\.cn\home
```

- `~/.cn/`(Windowsは `%USERPROFILE%\.cn\`)は、デフォルトのcabinetです。
- `home` は最初からある引き出しで、削除も改名もできません。

続いて、使うエディタを「[設定](#設定)」で指定してください。エディタが未設定だと、メモを開く操作はエラーになります。

## 設定

環境変数または設定ファイル(`config.yaml`)で設定します。環境変数が設定ファイルより優先されます。
環境変数 > 設定ファイル > デフォルト
空文字は「未設定」として扱います。

### 設定項目

| 項目 | 環境変数 | 設定ファイルのキー | デフォルト | 説明 |
|---|---|---|---|---|
| エディタ | `CN_EDITOR` | `EDITOR` | なし(必須) | メモを開くエディタのコマンド名 |
| cabinet(保存先) | `CN_CABINET` | `CABINET` | `~/.cn/`(Windowsは `%USERPROFILE%\.cn\`) | メモを保存するディレクトリ(絶対パス) |

> [!IMPORTANT]
> - 保存先は、**絶対パス**で書いてください(`~` や相対パスは拒否されます)。保存先のディレクトリも、先に作っておいてください。
> - 引き出しは、保存先の**直下**のディレクトリです。保存先を、引き出しのある場所の1つ上や下に指定すると、`No drawers` と表示されます。

> [!NOTE]
> エディタは、コマンド名だけを書きます。`code --wait` のような引数付きは使えません。

### 設定ファイル

設定ファイルは自動では作られません。次の場所に、ディレクトリごと作成してください。

| OS | パス |
|---|---|
| Linux, Mac | `~/.config/cn/config.yaml` |
| Windows | `%APPDATA%\cn\config.yaml` |

> [!WARNING]
> Windowsのパスは、**ダブルクォートで囲まないでください**。`"C:\Users\..."` は、`\U` がエスケープとして解釈されエラーになります。囲む場合は、シングルクォートにします。

<details>
<summary>設定ファイルの記述例</summary>

Linux:

```yaml
EDITOR: nvim
CABINET: /home/me/notes
```

Windows:

```yaml
EDITOR: nvim
CABINET: C:\Users\me\notes
```

</details>

### 設定の確認

- `cn memo -e`: 使用するエディタを表示
- `cn cabinet`: 保存先を表示

## 使い方

### ヘルプ・バージョン

| コマンド | 動作 |
|---|---|
| `cn --help` / `cn -h` | コマンドとオプションの一覧を表示 |
| `cn --version` / `cn -V` | バージョンを表示 |

### cabinet(保存先)

| コマンド | 動作 |
|---|---|
| `cn cabinet` / `cn c` | 保存先のパスを表示 |

### drawer(引き出し)

| コマンド | 動作 |
|---|---|
| `cn drawer` | 現在の引き出しを表示 |
| `cn drawer -l` | 一覧(現在の引き出しに `*`) |
| `cn drawer -a <name>` | 追加 |
| `cn drawer -c <name>` | 名前で切り替え |
| `cn drawer -cn <number>` | 番号で切り替え |
| `cn drawer -r <new name>` | 現在の引き出しの名前を変更 |
| `cn drawer -d <name>` | 名前で削除(確認あり) |
| `cn drawer -dn <number>` | 番号で削除(確認あり) |

- 現在の引き出しを削除すると、`home` に戻ります。
- 引き出しの名前には、`/` `\` `.` など、使えない文字があります。50文字までです。

### memo(メモ)

| コマンド | 動作 |
|---|---|
| `cn memo <title>` | タイトルで開く(タイトルがないメモは日時で指定) |
| `cn memo -n <number>` | 番号で開く |
| `cn memo -l` | 現在の引き出しのメモ一覧 |
| `cn memo -a` | 新しいメモを作って開く |
| `cn memo -e` | 使うエディタを表示 |
| `cn memo -d <title>` | タイトルで削除(確認あり) |
| `cn memo -dn <number>` | 番号で削除(確認あり) |

- 番号は、一覧に表示される番号(`[1]` の `1`。1から始まります)です。
- `cn memo <title>` と `cn memo -d <title>` には、一覧に表示されている名前(タイトル、無ければ日時)を指定します。
- タイトルは1つの引数です。スペースを含む場合は、引用符で囲んでください(`cn memo "my title"`)。
- `-` で始まるタイトルは、オプションとして扱われます。その場合は `-n` で番号を指定してください。
- メモのタイトルは、本文の最初の `# 見出し`(レベル1)です。コードブロックの中は対象外です。見出しがないメモは、一覧にタイトルの代わりに日時(例: `20261011120000`)が出ます。
- 同じタイトルのメモが複数あると、開く・削除はエラーになり、日時付きの候補が表示されます。`-n` / `-dn` で番号を指定してください。
- メモのファイル名は `YYYYMMDD_HHmmss.md` です。ファイル名は保存先全体で一意で、同じ秒に2つ作るとエラーになります。1秒待って、もう一度実行してください。

## 文字コード

メモは **UTF-8** のみに対応しています。Shift-JISなど、UTF-8以外のメモは、一覧でタイトルが出ません(日時のみ表示。開く・削除はできます)。

## ライセンス

[MIT License](LICENSE)
