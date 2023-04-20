/*
 * TODO: move this file to its own package and publish to npm when it's been proven to work on all platforms.
 */

const crypto = require("crypto");
const fs = require("fs");
const https = require("https");
const os = require("os");
const path = require("path");
const tar = require("tar");
const Zip = require("adm-zip");

module.exports = class Wrapper {
  constructor(binName, binDest, platforms) {
    const platform = Wrapper._findPlatform(platforms);

    console.log(platform);

    this.binName = binName;
    this.binDest = binDest;
    this.binPrefix = platform.binPrefix || "";
    this.binSuffix = platform.binSuffix || "";
    this.checksum = platform.checksum;
    this.url = new URL(platform.url);
  }

  install() {
    Wrapper._downloadArchive(this.url, (err, archiveFile) => {
      if (err) {
        throw err;
      }

      Wrapper._verifyChecksum(archiveFile, this.checksum, (err) => {
        if (err) {
          throw err;
        }

        Wrapper._extractArchive(archiveFile, (err, extractedDir) => {
          if (err) {
            throw err;
          }

          const binPath = path.join(
            extractedDir,
            this.binPrefix + this.binName + this.binSuffix
          );
          Wrapper._installBinary(binPath, this.binDest, (err) => {
            if (err) {
              throw err;
            }

            console.log(`Binary successfully installed: ${this.binDest}`);
          });
        });
      });
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
              `Unexpected status code ${res.statusCode} when requesting ${url}`
            )
          );
        }

        res
          .pipe(fs.createWriteStream(outfile))
          .on("error", (err) => {
            return cb(err);
          })
          .on("finish", () => {
            return cb(null, outfile);
          });
      });
    });
  }

  static _verifyChecksum(filepath, checksum, cb) {
    fs.readFile(filepath, (err, data) => {
      if (err) {
        return cb(err);
      }

      const hash = crypto.createHash("sha256").update(data).digest("hex");
      if (`sha256:${hash}` !== checksum) {
        return cb(new Error("Checksum mismatch"));
      }

      return cb(null);
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

  static _installBinary(archiveBinPath, installBinPath, cb) {
    fs.mkdir(path.dirname(installBinPath), { recursive: true }, (err) => {
      if (err) {
        return cb(err);
      }
      fs.rename(archiveBinPath, installBinPath, cb);
    });
  }

  static _findPlatform(platforms) {
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
        res.on("data", () => {}); // consume all data so the script does not hang
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
