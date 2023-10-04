{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          rustToolchain = if builtins.pathExists "${self}/rust-toolchain.toml" then
	    pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile "${self}/rust-toolchain.toml" 
          else
            pkgs.pkgsBuildHost.rust-bin.stable.latest.default;
          nativeBuildInputs = with pkgs; [rustToolchain pkg-config ];
          buildInputs = with pkgs; [ alsa-lib udev ];
        in
        with pkgs;
        {
          devShells.default = mkShell {
            inherit buildInputs nativeBuildInputs;
          };
        }
      );
}
