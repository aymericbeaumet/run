const Wrapper = require("./wrapper");
const { repository, version } = require("./package.json");

const assetsPrefix = `${repository.url}/releases/download/v0.0.12`; // TODO: $version

// https://github.com/aymericbeaumet/run/blob/master/.github/workflows/release.yml
module.exports = new Wrapper([
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
  },
]);
