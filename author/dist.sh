#!/bin/bash
set -ex

dist="dist"

for dir in $(\ls $dist); do
  target_dir="$dist/$dir"
  if [[ ! -d $target_dir ]]; then
    continue
  fi

  for file in "README.md" "LICENSE" "credits.html" "CHANGELOG.md"; do
    cp $file $target_dir
  done

  if [[ "$dir" == *"linux"* ]]; then
    sh -c "cd $dist && tar -czvf $dir.tar.gz $dir"
  else
    sh -c "cd $dist && zip -r $dir.zip $dir"
  fi
  rm -rf $target_dir
done

rm -rf $dist/SHA256SUMS
cd $dist && shasum -a 256 $(find * -type f -maxdepth 0) > SHA256SUMS
