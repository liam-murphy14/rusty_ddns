{
  rustPlatform,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "rusty_ddns";
  version = "0.1.0";

  src = ./.;

  cargoHash = "sha256-kMT3nTTuKkAZsJ3yILTyfPyRmZOrq3nGoykhlG6DRvo=";

  meta = {
    description = "A minimal DDNS client, written in Rust with 0 dependencies.";
    homepage = "TODO";
  };
})
