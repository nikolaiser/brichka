release VERSION:
  jj new main && sed -i 's/^version = ".*"/version = "{{VERSION}}"/' Cargo.toml && jj describe -m "release: v{{VERSION}}" && jj bookmark move main --to @ && jj git push && git tag v{{VERSION}} && git push --tags

