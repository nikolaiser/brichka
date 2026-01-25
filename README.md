# Brichka

A lightweight CLI for running code on Databricks clusters with notebook-like execution contexts and Unity Catalog autocomplete.

## Features

* **Execute code** on Databricks interactive clusters (SQL, Scala, Python, R)
* **Shared contexts** - run multiple commands that share state, like notebook cells
* **Unity Catalog LSP** - autocomplete for catalogs, schemas, and tables in any editor
* **JSONL output** - structured results you can pipe to other tools

Works standalone or with the [Neovim plugin](https://github.com/nikolaiser/brichka.nvim).


## Installation

Prerequisites:
* Authenticated [databricks-cli](https://github.com/databricks/cli)
 
<details>
<summary>Installation methods</summary>

### Nix
Try it out:

```bash
nix shell github:nikolaiser/brichka
```

Add to your flake inputs:

```nix
{
  inputs = {
    brichka = {
      url = "github:nikolaiser/brichka";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };


  outputs = { self, nixpkgs, brichka, ... }: {
    nixosConfigurations.myhost = nixpkgs.lib.nixosSystem {
      modules = [{
        ...
        environment.systemPackages = [ inputs.brichka.packages."${system}".brichka ];
        ...
      }];
    };
  };
}
```

### Cargo
```bash
cargo install brichka
```

### Homebrew

```bash
brew install nikolaiser/tap/brichka
```

</details>

## Usage

<details>
<summary>CLI arguments</summary>

```bash
 brichka [OPTIONS] <COMMAND>

Commands:
  cluster  Cluster commands
  config   Config commands
  init     Initialize a new execution context in the current working directory
  status   Status commands
  run      Run code on the interactive cluster
  lsp      Start LSP server for Unity Catalog completion
  help     Print this message or the help of the given subcommand(s)

Options:
      --cwd <CWD>  Override the current working dirrectory
      --debug      Print debug logs
  -h, --help       Print help

```
</details>

### Quick Start

1. **Select a cluster** (uses fzf to choose from available clusters):

```bash
# For current directory only
brichka config cluster

# Or globally
brichka config --global cluster
```

2. **Run code** (inline or from stdin):

```bash
# Inline
brichka run --language "sql" "select * from foo.bar.bazz"

# From file/stdin
cat script.sc | brichka run --language "scala" -
```

Results are returned as JSONL in a temporary file:
```json
{"type":"table","path":"/tmp/b909c39f1a934c1eb76595601a413bcc.jsonl"}
```

View results with any tool that reads JSONL (e.g., [visidata](https://www.visidata.org/), jq, etc.)

### Shared Execution Contexts (Notebook Mode)

Create a shared context where commands can reference each other's output, like notebook cells:

```bash
brichka init
```

Now all commands in this directory share state:

```sql
-- First command
create or replace temporary view _data as select * from catalog.schema.table
```

```scala
// Second scala command can access _data
display(spark.table("_data"))
```

### Unity Catalog Language Server

Get autocomplete for catalog/schema/table names in any editor.

<details>
<summary>Setup</summary>

#### Nvim

Add `~/.config/nvim/lsp/brichka.lua`:

```lua
---@type vim.lsp.Config
return {
  cmd = { "brichka", "lsp" },
  filetypes = { "sql", "scala" },
}
```

Then enable in your config:

```lua
vim.lsp.enable("brichka")
```

</details>

### Working with Scala

For Metals (Scala LSP) support, use `.sc` files with this template:

```scala
// Adjust to your target scala version
//> using scala 2.13
// Adjust to your target spark version
//> using dep org.apache.spark::spark-sql::3.5.7

// brichka: exclude
import org.apache.spark.sql.functions._
import org.apache.spark.sql.{DataFrame, SparkSession}

val spark: SparkSession = ???
def display(df: DataFrame): Unit = ()
// brichka: include

// Your code here
```

The `// brichka: exclude` comments let you add dummy values for Databricks objects (like `spark`) that Metals needs but shouldn't be sent to the cluster.

For multiple files in notebook mode, subsequent files should reference the first:

```scala
// brichka: include

//> using file fst.sc

// brichka: exclude
import fst._
import org.apache.spark.sql.functions._
// brichka: include
```
