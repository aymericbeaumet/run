const Wrapper = require("./wrapper");
const pkg = require("./package.json");

const assetsPrefix = `${pkg.repository.url}/releases/download/v0.0.12`; // TODO: $pkg.version
const bin_name = Object.keys(pkg.bin)[0];
const bin_dest = path.join(__dirname, pkg.bin[BIN_NAME]);

const wrapper = new Wrapper(bin_name, bin_dest, [
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

wrapper.install();
