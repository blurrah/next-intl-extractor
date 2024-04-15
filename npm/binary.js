#!/usr/bin/env node
const { Binary } = require("binary-install");

const error = (msg) => {
  console.error(msg);
  process.exit(1);
};

const { version, name, repository } = require("./package.json");

const supportedPlatforms = [
  {
    TYPE: "Windows_NT",
    ARCHITECTURE: "x64",
    RUST_TARGET: "x86_64-pc-windows-msvc",
    BINARY_NAME: "i18n-label-merger.exe",
  },
  {
    TYPE: "Linux",
    ARCHITECTURE: "x64",
    RUST_TARGET: "x86_64-unknown-linux-musl",
    BINARY_NAME: "i18n-label-merger",
  },
  {
    TYPE: "Darwin",
    ARCHITECTURE: "x64",
    RUST_TARGET: "x86_64-apple-darwin",
    BINARY_NAME: "i18n-label-merger",
  },
  {
    TYPE: "Darwin",
    ARCHITECTURE: "arm64",
    RUST_TARGET: "aarch64-apple-darwin",
    BINARY_NAME: "i18n-label-merger",
  },
];

const getPlatformMetadata = () => {
  const type = os.type();
  const architecture = os.arch();

  for (let supportedPlatform of supportedPlatforms) {
    if (
      type === supportedPlatform.TYPE &&
      architecture === supportedPlatform.ARCHITECTURE
    ) {
      return supportedPlatform;
    }
  }

  error(
    `Platform with type "${type}" and architecture "${architecture}" is not supported by ${name}.\nYour system must be one of the following:\n\n${cTable.getTable(
      supportedPlatforms
    )}`
  );
};

function getBinary() {
  const platformMetadata = getPlatformMetadata();

  const url = `https://github.com/blurrah/i18n-label-merger/releases/download/${version}/${name}-${version}-${platformMetadata.RUST_TARGET}.tar.gz`;

  return new Binary(platformMetadata.BINARY_NAME, url, version, {
    installDirectory: path.join(__dirname, "node_modules", ".bin"),
  });
}

const install = () => {
  const binary = getBinary();
  binary.install();
};

const run = () => {
  const binary = getBinary();
  binary.run();
};

module.exports = {
  install,
  run,
};
