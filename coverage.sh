#!/usr/bin/env bash

set -o nounset
set -o errexit
set -o pipefail

bash --version
find --version
grep --version

echo "List files:"
find

wget https://github.com/SimonKagstrom/kcov/archive/master.tar.gz
tar xzf master.tar.gz
cd kcov-master
mkdir build
cd build
cmake ..
make
make install DESTDIR=../../kcov-build
cd ../..
rm -rf kcov-master
FILENAME=$(find | grep "native/target/debug/carta_schema")
echo "Found test executable: ${FILENAME}"
./kcov-build/usr/local/bin/kcov --exclude-pattern=/.cargo,/usr/lib --verify "native/target/cov/" "${FILENAME}"
bash <(curl -s https://codecov.io/bash)
