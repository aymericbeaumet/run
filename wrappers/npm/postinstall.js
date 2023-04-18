const path = require("path");
const pkg = require("./package.json");
const Wrapper = require("./wrapper");

const assetsPrefix = `${pkg.repository.url}/releases/download/v${pkg.version}`;
const name = Object.keys(pkg.bin)[0];
const dest = path.join(__dirname, pkg.bin[name]);

const wrapper = new Wrapper(name, dest, [
  // amd64
  {
    type: "Linux",
    arch: "x64",
    url: `${assetsPrefix}/run-x86_64-unknown-linux-gnu.tar.gz`,
  },
  {
    type: "Freebsd",
    arch: "x64",
    url: `${assetsPrefix}/run-x86_64-unknown-freebsd.tar.gz`,
  },
  {
    type: "Darwin",
    arch: "x64",
    url: `${assetsPrefix}/run-x86_64-apple-darwin.tar.gz`,
  },
  {
    type: "Windows_NT",
    arch: "x64",
    url: `${assetsPrefix}/run-x86_64-pc-windows-msvc.zip`,
    binSuffix: ".exe",
  },
  // arm64
  {
    type: "Linux",
    arch: "arm64",
    url: `${assetsPrefix}/run-aarch64-unknown-linux-gnu.tar.gz`,
  },
  {
    type: "Darwin",
    arch: "arm64",
    url: `${assetsPrefix}/run-aarch64-apple-darwin.tar.gz`,
  },
  // i686
  {
    type: "Linux",
    arch: "ia32",
    url: `${assetsPrefix}/run-i686-unknown-linux-gnu.tar.gz`,
  },
  {
    type: "Windows_NT",
    arch: "ia32",
    url: `${assetsPrefix}/run-i686-pc-windows-msvc.zip`,
    binSuffix: ".exe",
  },
]);

wrapper.install();
