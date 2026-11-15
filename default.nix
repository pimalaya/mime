{
  pimalaya ? import (fetchTarball "https://github.com/pimalaya/nix/archive/master.tar.gz"),
  ...
}@args:

pimalaya.mkDefault (
  {
    src = ./.;
    version = "0.2.0";
    mkPackage = (
      {
        lib,
        pkgs,
        rustPlatform,
        defaultFeatures,
        features,
        buildPackages,
      }:

      pkgs.callPackage ./package.nix {
        inherit lib rustPlatform buildPackages;
        buildNoDefaultFeatures = !defaultFeatures;
        buildFeatures = lib.splitString "," features;
      }

    );
  }
  // removeAttrs args [ "pimalaya" ]
)
