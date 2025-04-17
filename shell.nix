{ pkgs ? import <nixpkgs> {} }:

let
  pythonEnv = pkgs.python3.withPackages (ps: with ps; [
    pip
    virtualenv
    yt-dlp
  ]);
in

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    pythonEnv
    ffmpeg
  ];

  shellHook = ''
    # Set up Rust environment
    export RUST_BACKTRACE=1

    # Create and activate Python virtual environment
    if [ ! -d "venv" ]; then
      python -m venv venv
    fi
    source venv/bin/activate

    # Install Python dependencies if not already installed
    if ! pip show yt-dlp > /dev/null 2>&1; then
      pip install yt-dlp
    fi

    echo "Development environment ready!"
    echo "Run 'cargo build' to build the project"
    echo "Run 'cargo run' to start the bot"
  '';
} 