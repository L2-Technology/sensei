{
  description = "sensei";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, crane, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        craneLib = crane.lib.${system};

        myCrate = craneLib.buildPackage {
            src = ./.;

            buildInputs = with pkgs; [
                openssl
                protobuf
                rustfmt
                pkg-config
            ];

            PROTOC = "${pkgs.protobuf}/bin/protoc";
            PROTOC_INCLUDE="${pkgs.protobuf}/include";

            nativeBuildInputs = with pkgs; [
                pkg-config
            ];

            cargoTestCommand = "";
        };
      in
      {
          checks = {
            inherit myCrate;
          };

          packages.default = myCrate;
          packages.container = pkgs.dockerTools.buildImage {
            name = "sensei";
            tag = myCrate.version;
            created = "now";
            contents = myCrate;
            config.Cmd = [ "${myCrate}/bin/senseid" ];
          };
          apps.default = flake-utils.lib.mkApp {
            drv = myCrate;
          };
          devShells.default = pkgs.mkShell {
            inputsFrom = builtins.attrValues self.checks;

            # Extra inputs can be added here
            nativeBuildInputs = with pkgs; [
                cargo
                rustc
                rustfmt
            ];
        };
    });
}