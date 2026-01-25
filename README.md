# Brichka

CLI tools for Databricks

## Installation

Prerequisites:
* Authenticated [databricks-cli](https://github.com/databricks/cli)
* 
<details>
<summary>Installation methods</summary>

### Nix
Try it out 

```bash
nix shell github:nikolaiser/kent
```

Add to your flake inputs 

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
