{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";

    # Best-practice Rust toolchains for flakes: pin toolchain + components + rust-analyzer.
    # (Fenix is flake-native and provides toolchains + rust-analyzer nightly.)
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, systems, fenix, ... }:
    let
      eachSystem = nixpkgs.lib.genAttrs (import systems);
      pkgsFor = nixpkgs.legacyPackages;
    in
    {
      devShells = eachSystem (system:
        let
          pkgs = pkgsFor.${system}.extend fenix.overlays.default;

          # Keep your Wayland/Iced runtime-dlopen set exactly as-is.
          dlopenLibraries = with pkgs; [
            libxkbcommon

            # GPU backend
            vulkan-loader
            # libGL

            # Window system
            wayland
            # xorg.libX11
            # xorg.libXcursor
            # xorg.libXi
          ];

          # Rust toolchain with the components you actually want in a dev env.
          # rust-src is important for rust-analyzer features like "go to definition".
          rustToolchain = pkgs.fenix.stable.withComponents [
            "cargo"
            "clippy"
            "rust-src"
            "rustc"
            "rustfmt"
          ];
        in
        {
          default = pkgs.mkShell {
            # mkShell "packages" is the modern equivalent of nativeBuildInputs for shells.
            packages = with pkgs; [
              rustToolchain
              rust-analyzer-nightly

              # Common quality-of-life / ecosystem tools for Rust projects:
              pkg-config

              # Frequently needed by crates using bindgen / native deps:
              clang
              llvmPackages.libclang
            ];

            # additional libraries that your project
            # links to at build time, e.g. OpenSSL
            buildInputs = [ ];

            # Keep your rpath trick for runtime dlopen() working.
            env.RUSTFLAGS =
              "-C link-arg=-Wl,-rpath,${nixpkgs.lib.makeLibraryPath dlopenLibraries}";

            # Optional: nice defaults for dev sessions
            env.RUST_BACKTRACE = "1";
          };
        });

      # Nice flake hygiene: provide a formatter so `nix fmt` works.
      formatter = eachSystem (system: (pkgsFor.${system}).nixfmt-rfc-style);
    };
}
