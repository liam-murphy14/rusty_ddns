{
  rustPlatform,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "rusty_ddns";
  version = "0.1.0";

  src = ./.;

  cargoHash = "sha256-Gd1RfhcDx1A3/8smQwgFDShvLq7dC+QvEarhTNJYYMY=";

  meta = {
    description = "A minimal DDNS client, written in Rust with 0 dependencies.";
    homepage = "TODO";
  };
})
