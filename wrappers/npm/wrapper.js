const childProcess = require("node:child_process");
const fs = require("fs");
const https = require("https");
const os = require("os");
const path = require("path");
const tar = require("tar");
const Zip = require("adm-zip");

// TODO: move this class to its own package and publish to npm
module.exports = class Wrapper {
  constructor(name, platforms) {
    const platform = Wrapper._platform(platforms);
    const nameWithExt = platform.type === "Windows_NT" ? `${name}.exe` : name;

    this.url = new URL(platform.url);
    this.downloadsDir = path.join(__dirname, "downloads");
    this.installDir = path.join(__dirname, "bin");
    this.binName = nameWithExt;
    this.binPath = path.join(this.installDir, this.binName);
  }

  install = () => {
    const archiveName = this.url.toString().replace(/[^a-zA-Z0-9.]/g, "_");
    const archivePath = path.join(this.downloadsDir, `${archiveName}`);

    Wrapper._downloadArchive(this.url, archivePath, (err) => {
      if (err) {
        throw err;
      }
      Wrapper._extractArchive(archivePath, (err, archiveDir) => {
        if (err) {
          throw err;
        }
        Wrapper._installBinary(
          path.join(archiveDir, this.binName),
          this.binPath,
          (err) => {
            if (err) {
              throw err;
            }
            console.log(`Binary successfully installed: ${this.binPath}`);
          }
        );
      });
    });
  };

  exec = () => {
    const result = childProcess.spawnSync(this.binPath, process.argv.slice(2), {
      cwd: process.cwd(),
      stdio: "inherit",
    });
    process.exit(result.status);
  };

  static _installBinary(archiveBinPath, installBinPath, cb) {
    const parentDir = path.dirname(installBinPath);

    fs.rm(parentDir, { recursive: true }, (err) => {
      if (err) {
        return cb(err);
      }
      fs.mkdir(parentDir, { recursive: true }, (err) => {
        if (err) {
          return cb(err);
        }
        fs.rename(archiveBinPath, installBinPath, cb);
      });
    });
  }

  static _downloadArchive(url, filepath, cb) {
    Wrapper._tempdir((err, dir) => {
      if (err) {
        return cb(err);
      }
      const outfile = path.join(dir, "archive");

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
            fs.rename(outfile, filepath, (err) => {
              return cb(err);
            });
          });
      });
    });
  }

  static _extractArchive(filepath, cb) {
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
