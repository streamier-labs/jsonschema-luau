{ pkgs, ... }:

{
  # ============================================================================
  # Development Packages
  # ============================================================================

  packages = with pkgs; [
    cargo-dist
  ];

  # ============================================================================
  # Language & Environment Setup
  # ============================================================================

  languages = {
    nix.enable = true;
    rust.enable = true;
  };

  # ============================================================================
  # Testing Suite
  # ============================================================================

  enterTest = ''
    cargo test
  '';

  # ============================================================================
  # Code Formatting (treefmt)
  # ============================================================================

  treefmt = {
    enable = true;
    config.programs = {
      nixfmt.enable = true;
      taplo.enable = true;
      yamlfmt.enable = true;
      rustfmt.enable = true;
    };
  };

  # ============================================================================
  # Git Hooks Configuration
  # ============================================================================

  git-hooks = {
    hooks = {
      # CI/CD Workflows & Repository Hygiene
      actionlint.enable = true;
      commitizen.enable = true;
      treefmt.enable = true;
      typos.enable = true;

      # Code Safety & File Formatting
      check-added-large-files.enable = true;
      check-case-conflicts.enable = true;
      check-merge-conflicts.enable = true;
      end-of-file-fixer.enable = true;
      mixed-line-endings.enable = true;
      trim-trailing-whitespace.enable = true;

      # Documentation & Markup
      markdownlint.enable = true;

      # Nix Static Analysis
      deadnix.enable = true;
      statix.enable = true;

      # Rust Quality Checks
      clippy.enable = true;
    };
    # Auto-generated files
    excludes = [
      "^\\.github/workflows/release\\.yml$"
    ];
  };
}
