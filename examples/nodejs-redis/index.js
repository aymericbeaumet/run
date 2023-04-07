import http from "http";
import { createClient } from "redis";

const [hostname, port] = ["127.0.0.1", 8080];

// TODO: remove this when run allows to wait for a command to be marked as "ready"
await new Promise((resolve) => setTimeout(resolve, 1000));

const redisClient = createClient({ url: "redis://127.0.0.1:6379/0" });
await redisClient.connect();

http
  .createServer(async function (_req, res) {
    const count = await redisClient.incr("count");
    res.end(`Count is now ${count}`);
  })
  .listen(port, hostname, () => {
    console.log(`Server is running on http://${hostname}:${port}`);
  });
