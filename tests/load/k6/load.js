import http from "k6/http";
import { check, sleep } from "k6";

export const options = {
  stages: [
    { duration: "1m", target: 10 },
    { duration: "3m", target: 10 },
    { duration: "1m", target: 0 },
  ],
  thresholds: {
    http_req_failed: ["rate<0.05"],
    http_req_duration: ["p(95)<1000"],
  },
};

const baseUrl = __ENV.VERDYX_API_BASE_URL || "http://localhost:8080";

export default function () {
  const health = http.get(`${baseUrl}/health`);
  check(health, {
    "health returns 2xx": (r) => r.status >= 200 && r.status < 300,
  });

  const bounties = http.get(`${baseUrl}/api/v1/bounties`);
  check(bounties, {
    "bounties is available or protected": (r) =>
      [200, 401, 403].includes(r.status),
  });

  sleep(1);
}
