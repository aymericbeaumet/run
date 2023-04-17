const childProcess = require("node:child_process");
const fs = require("fs");
const https = require("https");
const os = require("os");
const path = require("path");
const tar = require("tar");
const Zip = require("adm-zip");

// TODO: move this class to its own package and publish to npm
module.exports = class Wrapper {
  constructor(name, destFile, platforms) {
    const platform = Wrapper._platform(platforms);
    const nameWithExt = platform.type === "Windows_NT" ? `${name}.exe` : name;

    this.url = new URL(platform.url);
    this.name = nameWithExt;
    this.destFile = destFile;
  }

  install = () => {
    Wrapper._downloadArchive(this.url, (err, archivePath) => {
      if (err) {
        throw err;
      }
      Wrapper._extractArchive(archivePath, (err, archiveDir) => {
        if (err) {
          throw err;
        }
        Wrapper._installBinary(
          path.join(archiveDir, this.name),
          this.destFile,
          (err) => {
            if (err) {
              throw err;
            }
            console.log(`Binary successfully installed: ${this.destFile}`);
          }
        );
      });
    });
  };

  static _installBinary(archiveBinPath, installBinPath, cb) {
    const parentDir = path.dirname(installBinPath);

    fs.mkdir(path.dirname(installBinPath), { recursive: true }, (err) => {
      if (err) {
        return cb(err);
      }
      fs.rename(archiveBinPath, installBinPath, cb);
    });
  }

  static _downloadArchive(url, cb) {
    Wrapper._tempdir((err, dir) => {
      if (err) {
        return cb(err);
      }
      const outfile = path.join(
        dir,
        url.toString().replace(/[^a-zA-Z0-9.]/g, "_")
      );

      Wrapper._httpsGet(url, (res) => {
        if (res.statusCode !== 200) {
          return cb(
            new Error(
              `Unexpected status code ${res.statusCode} when requesting ${this.url}`
            )
          );
        }

        res
          .pipe(fs.createWriteStream(outfile))
          .on("error", (err) => {
            return cb(err);
          })
          .on("finish", () => {
            return cb(err, outfile);
          });
      });
    });
  }

  static _extractArchive(filepath, cb) {
    console.log({ filepath });
    if (filepath.endsWith(".tar.gz") || filepath.endsWith(".tgz")) {
      Wrapper._tempdir((err, dir) => {
        if (err) {
          return cb(err);
        }
        tar.x({ file: filepath, cwd: dir }, (err) => {
          cb(err, dir);
        });
      });
    } else if (filepath.endsWith(".zip")) {
      Wrapper._tempdir((err, dir) => {
        if (err) {
          return cb(err);
        }
        try {
          new Zip(filepath).extractAllTo(dir, true);
          return cb(null, dir);
        } catch (err) {
          return cb(err);
        }
      });
    } else {
      return cb(new Error("unknown file type"));
    }
  }

  static _platform(platforms) {
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

  static _httpsGet(url, cb) {
    https.get(url, (res) => {
      if (
        res.statusCode > 300 &&
        res.statusCode < 400 &&
        res.headers.location
      ) {
        res.on("data", () => {}); // consume all data so the script do not hang
        res.on("end", () => Wrapper._httpsGet(res.headers.location, cb));
      } else {
        cb(res);
      }
    });
  }

  static _tempdir(cb) {
    fs.mkdtemp(path.join(os.tmpdir(), "wrapper-"), cb);
  }
};
