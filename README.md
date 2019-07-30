[![Build Status](https://dev.azure.com/sklosegithub/disrustor/_apis/build/status/sklose.disrustor?branchName=master)](https://dev.azure.com/sklosegithub/disrustor/_build/latest?definitionId=2&branchName=master)

# Disrustor

This project is a port of the [LMAX Disruptor](https://lmax-exchange.github.io/disruptor/) to Rust.

# Features

- [x] Single Producer
- [x] Batch Consumer
- [X] Blocking Wait Strategy
- [X] Spinning Wait Strategy
- [ ] Multi Producer
- [ ] Worker Pools
- [ ] DSL
- [ ] Documentation

# Benchmarks

Preliminary benchmark results for sending i32-sized messages from a producer to a consumer.

| Name | Batch Size | Throughput |
|------|------------|------------|
| mpsc channel | 1<sup>[1](#mpsc-footnote)</sup> | 34.894 Melem/s |
| disrustor spinning | 1 | 38.260 Melem/s |
| disrustor spinning | 10 | 941.39 Melem/s |
| disrustor spinning | 50 | 940.77 Melem/s |
| disrustor spinning | 100 | 942.68 Melem/s |
| disrustor spinning | 1000 | 942.02 Melem/s |
| disrustor spinning | 2000 | 940.75 Melem/s |
| disrustor spinning | 4000 | 938.44 Melem/s |
| disrustor blocking | 1 | 7.0191 Melem/s |
| disrustor blocking | 10 | 85.386 Melem/s |
| disrustor blocking | 50 | 997.89 Melem/s |
| disrustor blocking | 100 | 998.97 Melem/s |
| disrustor blocking | 1000 | 1.0032 Gelem/s |
| disrustor blocking | 2000 | 999.67 Melem/s |
| disrustor blocking | 4000 | 999.15 Melem/s |

<a name="mpsc-footnote">1</a>: mpsc channels do not support batching

# Related Work

- [Turbine](https://github.com/polyfractal/Turbine)