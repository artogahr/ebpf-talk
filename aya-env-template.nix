{
  description = "eBPF Development Environment";

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
          rustup
          bpf-linker
          cargo-generate
          
          llvmPackages.clang
          llvmPackages.libclang
          pkg-config
          zlib
          elfutils
        ];
        
        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        
        shellHook = ''
          if ! rustup toolchain list | grep -q nightly; then
            rustup toolchain install nightly --component rust-src
          fi
        '';
      };
    };
}
