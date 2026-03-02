{
  rustPlatform,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "rusty_ddns";
  version = "0.1.0";

  src = ./.;

  cargoHash = "sha256-pJ1J8fdFTgaS4A5aomCs8d6K6QeN3G1SsEg4+JTbiOw=";

  meta = {
    description = "A minimal DDNS client, written in Rust with 0 dependencies.";
    homepage = "TODO";
  };
})
