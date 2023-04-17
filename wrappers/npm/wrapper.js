const fs = require("fs");
const https = require("https");
const os = require("os");
const path = require("path");

// TODO: move this class to its own package and publish to npm
module.exports = class Wrapper {
  constructor(platforms) {
    const platform = Wrapper.platform(platforms);

    this.url = new URL(platform.url);
    this.downloadsDir = path.join(__dirname, "downloads");
  }

  install = () => {
    const filename = this.url.toString().replace(/[^a-zA-Z0-9.]/g, "_");
    const filepath = path.join(this.downloadsDir, `${filename}`);
    const suffix = `-${Math.random()}`;

    fs.mkdirSync(path.dirname(filepath), { recursive: true });
    const fileStream = fs.createWriteStream(filepath + suffix);

    Wrapper.httpsGet(this.url, (res) => {
      if (res.statusCode !== 200) {
        throw new Error(
          `Unexpected status code ${res.statusCode} when requesting ${this.url}`
        );
      }
      res.pipe(fileStream).on("finish", () => {
        console.log("Downloaded", filepath + suffix);
      });
    });
  };

  exec = () => {
    console.log("exec");
  };

  static extname(s) {
    if (s.endsWith(".tar.gz")) {
      return ".tar.gz";
    }
    return path.extname(s);
  }

  static platform(platforms) {
    const type = os.type();
    const arch = os.arch();

    for (const platform of platforms) {
      if (type === platform.type && arch === platform.arch) {
        return platform;
      }
    }

    throw new Error(
      `Your platform has type=${type} and arch=${arch}, and is not supported.`
    );
  }

  static httpsGet(url, cb) {
    https.get(url, (res) => {
      if (
        res.statusCode > 300 &&
        res.statusCode < 400 &&
        res.headers.location
      ) {
        res.on("data", () => {});
        res.on("end", () => Wrapper.httpsGet(res.headers.location, cb));
      } else {
        cb(res);
      }
    });
  }
};
