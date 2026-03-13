#!/bin/bash
set -e

# Only run in remote cloud environments
if [ "$CLAUDE_CODE_REMOTE" != "true" ]; then
  exit 0
fi

# Install mise if not present
if ! command -v mise &> /dev/null; then
  echo "Installing mise..."
  curl -fsSL https://mise.jdx.dev/install.sh | sh

  if [ -n "$CLAUDE_ENV_FILE" ]; then
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$CLAUDE_ENV_FILE"
  fi
  export PATH="$HOME/.local/bin:$PATH"
fi

# Install tools defined in mise configuration
echo "Running mise install..."
mise install

echo "Remote setup complete."
