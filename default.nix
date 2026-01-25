{
  rustPlatform,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "rusty_ddns";
  version = "1.0.0";

  src = ./.;

  cargoHash = "sha256-/5df2d04xpsNrb8khvMZNLtwzi3PHT6XjMae+h6IzYc=";

  meta = {
    description = "A minimal DDNS client, written in Rust with 0 dependencies.";
    homepage = "TODO";
  };
})
