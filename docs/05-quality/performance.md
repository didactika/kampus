# Performance of 0.1.0

Date: 2026-11-23. Requirement: RNF-004, a 95th percentile of API latency under 200 ms with 1,000 concurrent people.

Result: **met**, with a 95th percentile of 10.4 ms and no failed request.

## How it is measured

`tools/load/campus.js` (k6) signs in once and runs 1,000 virtual people who browse as the web client does: the home (the account, the profile, the menu, the layout and the latest notifications), the catalogue (the categories and a page of courses) and a course (the course, its sections, the progress and the grades of the person, and the next 30 days of its calendar), with a pause of one to three seconds between pages. The load ramps up to 200 people in 30 seconds and to 1,000 in a minute, holds for two minutes and ramps down. It fails when the 95th percentile of any page goes over 200 ms, or when more than 1% of the requests fail.

```sh
docker run --rm --network kampus_default \
  -e KAMPUS_LOAD_URL=http://server:8080 -e KAMPUS_LOAD_USER=admin -e KAMPUS_LOAD_PASSWORD=<password> \
  -v "$PWD/tools/load:/load" grafana/k6:1.3.0 run /load/campus.js
```

`KAMPUS_LOAD_PEOPLE` and `KAMPUS_LOAD_HOLD` change the number of people and how long the full load lasts.

## Reference hardware

One machine runs everything in Docker: Kampus in one process (`all`, release build, pool of 10 connections), PostgreSQL 17, Valkey 8.1 and k6 itself; an Intel Core i9-14900KF (32 threads) with 31 GB of memory available to Docker. The campus holds 33 courses.

## Results

| Measure | Value |
| --- | --- |
| Requests | 341,476 (1,444 per second), none failed |
| Latency | median 2.2 ms, 95th percentile 10.4 ms, 99th percentile 19.5 ms, maximum 437 ms |
| Home | 95th percentile 4.1 ms |
| Catalogue | 95th percentile 3.8 ms |
| Course | 95th percentile 13.5 ms |

The first runs found that a request could hold a database connection while it waited for another, which starved the pool under load: 0.04% of the requests failed after 30 seconds, with a maximum latency of 31 s. The fix and the guards against it are in the [security review](./security-review.md) (S-02); the results above are after it.

## Limits of this measure

The virtual people share one session, so the cost of opening sessions and of permission checks for many different people is not measured; reads dominate, as on a real campus, and changes such as grading are not part of the mix. It runs on one machine, with the load generator beside the server. The test is repeated before each release on the same hardware, and the results recorded here.
