{
	inputs = {
		nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
		flake-utils.url = "github:numtide/flake-utils";
		rust-overlay = {
			url = "github:oxalica/rust-overlay";
			inputs.nixpkgs.follows = "nixpkgs";
		};
	};

	outputs = { self, nixpkgs, flake-utils, rust-overlay }: flake-utils.lib.eachDefaultSystem (system:
		let
			overlays = [ rust-overlay.overlays.default ];
			pkgs = import nixpkgs { inherit system overlays; };
			rust = pkgs.rust-bin.fromRustupToolchainFile ./arcviz/rust-toolchain.toml;
		in
		rec {
			devShell = pkgs.mkShell {
				buildInputs = with pkgs; [
					yarn
					snapcraft
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
					sass --no-source-map style/styles.sass styles.css
				'';
				installPhase = ''
					mkdir -p $out/share
					cp -r ./* $out/share
					ln -s ${packages.arcviz-wasm}/share $out/share/pkg
				'';
			};

			packages.arcviz-wasm = pkgs.rustPlatform.buildRustPackage {
				pname = "arcviz-wasm";
				version = "0.1.0";
				src = ./arcviz;
				cargoLock.lockFile = ./arcviz/Cargo.lock;
				cargoLock.outputHashes = {
					# need to be specified explicitly because they are git dependencies in Cargo.toml
					"result_or_err-0.1.0" = "sha256-LOOnHKY+G6Gb2VuEixMN4r3Dd3UXb4kKlVNZZLNizYk=";
					"webbit-0.1.0" = "sha256-rfNo8labW67aooQGUcf9A7y0mOIwn37zQ4q5/Auy6KI=";
				};
				nativeBuildInputs = with pkgs; [
					rust # NOTE: this is necessary: it provides the rust build tools
					wasm-bindgen-cli
				];
				buildPhase = ''
					cargo build --release --target=wasm32-unknown-unknown
					wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/arcviz.wasm 
				'';
				installPhase = ''
					mkdir -p $out/share
					cp -r pkg/* $out/share
				'';
			};
		}
	);
}
