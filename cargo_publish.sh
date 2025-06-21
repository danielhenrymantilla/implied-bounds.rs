#!/bin/sh

set -euxo pipefail

cargo update -vw
[[ -z "$(git status --porcelain)" ]]


(cd src/proc_macros
    cargo publish
)

cargo publish
