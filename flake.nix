{
  description = "eBPF Talk Demo and Slides";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          # Rust tooling
          rustup
          
          # For eBPF
          bpf-linker
          bpftools
          cargo-generate
          presenterm
          
          # Build deps
          llvmPackages.clang
          llvmPackages.libclang
          pkg-config
          zlib
          elfutils
          openssl
        ];
        
        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        
        # Ensure nightly rust-src is available for eBPF
        shellHook = ''
          if ! rustup toolchain list | grep -q nightly; then
            echo "Installing nightly toolchain for eBPF..."
            rustup toolchain install nightly --component rust-src
          fi
        '';
      };
    };
}
