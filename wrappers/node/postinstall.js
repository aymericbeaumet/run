const path = require("path");
const Wrapper = require("./wrapper");
const pkg = require("./package.json");
const artifacts = require("./artifacts.json");

const assetsPrefix = `${pkg.repository.url}/releases/download/v${pkg.version}/`;
const name = Object.keys(pkg.bin)[0];
const dest = path.join(__dirname, pkg.bin[name]);

const GOOS_TO_NODETYPE = {
  linux: "Linux",
  darwin: "Darwin",
  windows: "Windows_NT",
  freebsd: "Freebsd",
};

const GOARCH_TO_NODEARCH = {
  amd64: "x64",
  arm64: "arm64",
  386: "ia32",
};

const platforms = artifacts
  .filter((a) => a.type === "Archive")
  .map((a) => ({
    type: GOOS_TO_NODETYPE[a.goos],
    arch: GOARCH_TO_NODEARCH[a.goarch],
    url: assetsPrefix + a.name,
    checksum: a.extra.Checksum,
  }));

const wrapper = new Wrapper(name, dest, platforms);
wrapper.install();
