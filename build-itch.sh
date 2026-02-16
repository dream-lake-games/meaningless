#!/bin/bash
set -e

echo "Building release..."
trunk build --release

echo "Creating zip..."
cd dist && zip -r ../meaningless-web.zip . \
  -x "*.aseprite" \
  -x "*.DS_Store" \
  -x "*/_aseprite/*" \
  -x "*/sound/*.wav" \
  -x "*/music/*.wav" \
  && cd ..

echo "Done! Upload meaningless-web.zip to itch.io"
echo "Bundle size:"
ls -lh meaningless-web.zip
