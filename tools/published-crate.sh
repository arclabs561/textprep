#!/usr/bin/env bash
# Print first (unregistered name), missing (new version), or present (verified artifact).
set -euo pipefail

name=${1:?crate name required}
case "$name" in
  textprep-source-map) package_path=crates/source-map ;;
  textprep) package_path="" ;;
  *) echo "Unknown release package: $name" >&2; exit 1 ;;
esac
version=$(cargo metadata --no-deps --format-version 1 | jq -er --arg name "$name" '.packages[] | select(.name == $name) | .version')
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
url="https://index.crates.io/te/xt/$name"
http=$(curl --retry 3 -sSL -o "$tmp/index" -w '%{http_code}' "$url")
case "$http" in
  404) echo first; exit 0 ;;
  200) ;;
  *) echo "Registry index returned HTTP $http for $name" >&2; exit 1 ;;
esac
entry=$(jq -cs --arg version "$version" 'map(select(.vers == $version)) | first // empty' "$tmp/index")
if [[ -z "$entry" ]]; then
  echo missing
  exit 0
fi
checksum=$(jq -er 'select(.yanked == false) | .cksum' <<< "$entry")
archive="$name-$version"
curl --fail --retry 3 -sSL "https://static.crates.io/crates/$name/$archive.crate" -o "$tmp/package"
echo "$checksum  $tmp/package" | sha256sum -c --status
vcs=$(tar -xzOf "$tmp/package" "$archive/.cargo_vcs_info.json")
source_commit=$(jq -er --arg path "$package_path" 'select(.path_in_vcs == $path and .git.dirty != true) | .git.sha1' <<< "$vcs")
[[ "$source_commit" =~ ^[0-9a-f]{40}$ ]] || exit 1
if [[ -n "$package_path" ]]; then
  # A later textprep release can reuse a previously published, unchanged dependency.
  git merge-base --is-ancestor "$source_commit" HEAD
  git diff --exit-code "$source_commit" HEAD -- "$package_path" >&2
elif [[ "$source_commit" != "$(git rev-parse HEAD)" ]]; then
  echo "$archive was published from a different release commit ($source_commit)" >&2
  exit 1
fi
echo present
