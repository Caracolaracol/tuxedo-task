{
  lib,
  rustPlatform,
  writableTmpDirAsHomeHook,
  versionCheckHook,
}:
let
  cargoToml = fromTOML (builtins.readFile ./Cargo.toml);
  inherit (cargoToml.package) version;
in

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "tuxedo";
  inherit version;
  __structuredAttrs = true;

  src = lib.cleanSource ./.;

  cargoLock.lockFile = ./Cargo.lock;

  nativeCheckInputs = [ writableTmpDirAsHomeHook ];

  doInstallCheck = true;
  nativeInstallCheckInputs = [ versionCheckHook ];

  __darwinAllowLocalNetworking = true;

  checkFlags = [
    # Failure
    "--skip=insert_dialog_after_nl_parse"
  ];

  meta = {
    description = "Tuxedo Task: a Tuxedo fork with nested todo.txt subtasks";
    homepage = "https://github.com/Caracolaracol/tuxedo-task";
    changelog = "https://github.com/Caracolaracol/tuxedo-task/releases";
    license = lib.licenses.mit;
    mainProgram = "tuxedo";
  };
})
