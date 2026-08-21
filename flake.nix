{
	inputs = {
		nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
		flake-utils.url = "github:numtide/flake-utils";
		rust-overlay = { # rust version/toolchain overlay: to add wasm target
			url = "github:oxalica/rust-overlay";
			inputs.nixpkgs.follows = "nixpkgs";
		};
		naersk.url = "github:nix-community/naersk"; # incremental rust builds
	};

	outputs = { self, nixpkgs, flake-utils, rust-overlay, naersk }: flake-utils.lib.eachDefaultSystem (system:
		let
			overlays = [ rust-overlay.overlays.default ];
			pkgs = import nixpkgs { inherit system overlays; };
			rust = pkgs.rust-bin.fromRustupToolchainFile ./arcviz/rust-toolchain.toml;
			naerskLib = pkgs.callPackage naersk {
				cargo = rust;
				rustc = rust;
			};
		in
		rec {
			devShell = pkgs.mkShell {
				buildInputs = with pkgs; [
					yarn
					# snapcraft
					wineWowPackages.stable
				];
				shellHook = ''
					set -e
					cd electron-wrapper
					yarn install
					ln -s ${packages.arcviz-web}/share app
					yarn run build
					exit
				'';
			};

			packages.default = packages.arcviz-desktop;
			packages.arcviz-desktop = pkgs.stdenv.mkDerivation {
				pname = "arcviz-desktop";
				version = "0.1.0";
				src = ./electron-wrapper;
				nativeBuildInputs = with pkgs; [
					makeWrapper
				];
				installPhase = ''
					mkdir -p $out/share
					install -m 444 -D package.json $out/share/package.json
					install -m 444 -D main.mjs $out/share/main.mjs
					install -m 444 -D preload.js $out/share/preload.js
					ln -s ${packages.arcviz-web}/share $out/share/app
					mkdir -p $out/bin
					makeWrapper ${pkgs.electron}/bin/electron $out/bin/arcviz-desktop \
						--add-flags $out/share \
				'';
			};

			packages.arcviz-web = pkgs.stdenv.mkDerivation {
				pname = "arcviz-web";
				version = "0.1.0";
				src = ./static;
				nativeBuildInputs = with pkgs; [
					nodePackages.sass
				];
				buildPhase = ''
					sass style/styles.sass styles.css
				'';
				installPhase = ''
					mkdir -p $out/share
					cp -r ./* $out/share
					ln -s ${packages.arcviz-wasm}/share $out/share/pkg
				'';
			};

			packages.arcviz-wasm = naerskLib.buildPackage {
				name = "arcviz-wasm";
				version = "0.1.0";
				src = ./arcviz;
				nativeBuildInputs = with pkgs; [
					wasm-bindgen-cli
				];
				CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
				postInstall = ''
					mkdir -p $out/share
					wasm-bindgen --target web --out-dir $out/share target/wasm32-unknown-unknown/release/arcviz.wasm 
				'';
			};
		}
	);
}
