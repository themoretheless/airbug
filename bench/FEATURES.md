# AirBench idea catalogue

Unprioritised ideas for airbug-bench (items 1–400), not a commitment. What is actually planned: [ROADMAP.md](../ROADMAP.md).

## 1. Performance Benchmarks & Profiling (1-100)

### Hardware-Level Instrumentation
1. Case-level wall clock precision to nanosecond granularity
2. Per-sample variance tracking across iterations
3. Memory allocation profiling per benchmark case
4. CPU frequency scaling correlation logging
5. **Cache miss rate instrumentation via hardware counters** ⭐ priority
6. Branch misprediction counting in hot paths
7. SIMD utilization metrics extraction
8. Instruction cache pressure measurement
9. TLB miss tracking during large datasets
10. Page fault counting at microsecond scale

### System Overhead Isolation
11. Context switch overhead isolation
12. Interrupt latency measurement windows
13. Hyperthreading interference detection
14. NUMA locality scoring for multi-socket systems
15. CPU affinity pinning verification
16. Thermal throttling detection and reporting
17. Power consumption correlation with speed
18. Energy-per-operation efficiency tracking
19. Wake-from-idle penalty accounting
20. Startup time decomposition (dso load, init, warmup)

### I/O Performance
22. Memory-mapped file I/O vs copy differentiation
23. Zero-copy path identification and benchmarking
24. Copy-on-write overhead measurement
25. Large page vs 4KB page performance delta
26. Transparent huge page impact analysis
27. Swap activity zero-correlation enforcement
28. Dirty page ratio monitoring during write-heavy benches
29. Buffer cache hit rate proxy measurement
30. I/O scheduler fairness scoring
31. Direct I/O throughput ceiling discovery
32. Asynchronous I/O completion latency tails
33. Queue depth optimization curves
34. Block size sweet-spot finding algorithms
35. RAID array stripe pattern sensitivity
36. ZFS compression impact on read paths
37. SSD wear leveling artifact suppression
38. NVMe queue parallelism scaling
39. PCIe lane contention detection
40. Storage controller firmware bugs workarounds

### Network Performance
41. Network round-trip baseline establishment
42. TCP window scaling optimization markers
43. UDP jitter distribution capture
44. Packet loss recovery overhead counting
45. TLS handshake latency breakdown
46. Certificate chain validation timing
47. Compression algorithm speed rankings
48. Encryption key schedule cost measurement

### Data Structures Benchmarking
49. Hash collision resolution strategy comparison
50. Bloom filter false positive tuning curves
51. LRU vs LFU eviction policy benchmarks
52. Write-back cache hit ratio tracking
53. Copy-on-read vs eager fetch strategies
54. Prefetch distance optimization loops
55. Loop unrolling factor search automation
56. Function inlining threshold exploration
57. Generic specialization trade-off mapping
58. Monomorphization explosion cost limits
59. Trait object dispatch overhead quantification
60. Virtual function table lookup elimination
61. Fat pointer vs thin pointer memory footprint
62. Alignment-aware data structure packing
63. Structure of arrays vs array of structures tests
64. Register pressure profiling through spill counting
65. Stack allocation vs heap choice heuristics

### Allocator Comparison
66. Arena allocator throughput comparison
67. jemalloc vs dlmalloc sizing benchmarks
68. Custom pool allocator gain estimation
69. Object reuse pattern mining from traces
70. Garbage collector pause time histograms
71. Mark-sweep compaction throughput measurement
72. Conservative stack scanning overhead tally
73. Reference counting atomic operation cost
74. ARC weak reference management efficiency
75. Ownership borrow-checker runtime impact null
76. Unsafe block boundary crossing penalties
77. FFI call convention tax measurement
78. C-string conversion overhead in loops
79. Slice bounds check elimination gains
80. Panic unwinding vs abort speed differential

### Error Handling Patterns
81. Try-catch mechanism comparison where applicable
82. Error propagation pattern cost tracing
83. Option enum pattern boxing decisions
84. Result type error message formatting delay
85. String interning deduplication benefits
86. Symbol table lookup caching strategies
87. Dictionary hash map vs tree map comparisons
88. Inline small buffer optimization testing
89. Vector capacity growth factor tuning
90. Deque amortized constant time verification
91. Skip list randomized height selection
92. Red-black tree rebalance rotation count
93. B-tree fanout optimization by sector size
94. Radix sort bucket count calibration
95. Quicksort pivot selection heuristic ranking
96. Mergesort stability overhead accounting
97. Heapsort sift-down path length tracking
98. Insertion sort threshold crossover point
99. Timsort run detection efficiency
100. Counting sort range bound discovery

## 2. Benchmark Infrastructure (101-200)

### Statistical Analysis
101. Deterministic RNG seeding reproducibility
102. Random seed injection via environment variable
103. Fixture lazy evaluation mode flag
104. Warmup iteration auto-detection algorithm
105. Steady-state convergence testing
106. Statistical outlier rejection policy
107. Trimmed mean vs median central tendency
108. Confidence interval width calculation
109. Bootstrap resampling distribution fitting
110. Kolmogorov-Smirnov normality check
111. Jarque-Bera test integration for skew
112. Anscombe quartet visualization of results
113. Regression line slope tracking over sessions
114. Baseline drift detection in long runs
115. CI/CD gate comparison against golden values
116. Delta report generation between commits
117. Blame annotation linking regressions to authors
118. Flaky test detection via rerun probability
119. Hardware signature fingerprinting (CPUID)
120. OS kernel version correlation metadata

### Build Artifacts Metadata
121. Rust compiler version in build artifacts
122. Feature flag set enumeration recording
123. Debug vs release binary distinction tagging
124. Profile-guided optimization impact logging
125. Link-time optimization enablement checking
126. Crate dependency tree snapshot export
127. Lockfile checksum reproducibility verification
128. Cross-compilation target benchmark consistency

### Container & VM Performance
129. Emulator vs native execution gap measurement
130. QEMU syscall trace overhead baseline
131. Docker container sandbox performance cost
132. WSL2 filesystem translation penalty
133. VM snapshot restore timing accuracy
134. Sandbox namespace creation latency
135. Seccomp filter application overhead
136. SELinux policy decision timing samples
137. AppArmor profile match computation
138. Capability dropping early start effect
139. cgroup memory limit violation triggering
140. OOM killer interaction avoidance strategy

### Synchronization Primitives
141. Signal handler registration cost
142. Alt-stack signal delivery measurement
143. User-mode locking vs futex contention
144. Mutex spinlock threshold tuning
145. Condvar wake-up spurious retry counting
146. Atomic compare-exchange loop unrolling
147. Seqlock read-side critical section cost
148. RCU grace period wait sampling
149. Read-write lock upgrade/downgrade overhead
150. Barrier synchronization idle worker waste
151. Join handle spawn latency measurement
152. Thread pool scheduling granularity analysis
153. Work stealing deque contention profiling
154. Task queue batching efficiency measurement

### Async & Future Patterns
155. Future poll delay accounting in async
156. Spawn await synchronization overhead
157. tokio runtime thread starvation detection
158. rayon worker idle time fraction tracking
159. crossbeam channel blocking vs nonblocking
160. mpsc single-producer vs multi-producer test
161. broadcast channel fanout scaling curve
162. Oneshot channel signaling latency spike
163. Sync semaphore permit acquisition wait
164. Busy-wait spin budget compliance audit
165. Yield now hint effectiveness verification
166. Sleep accuracy jitter histogram collection
167. Clock monotonic vs real-time selection
168. High-resolution timer syscall overhead
169. getrandom syscall entropy source timing
170. /dev/urandom mmap vs read comparison

### Cryptographic RNG Benchmarks
171. OpenSSL RAND_bytes fallback path cost
172. ChaCha20 CSPRNG seeding overhead
173. Xoshiro state transition cycle length
174. PCG permutation shuffle quality check
175. SplitMix64 stride collision resistance
176. Mulberry32 speed vs randomness tradeoff
177. xorshift128+ throughput peak finding
178. Threefry cryptographic permutation timing
179. Philox counter mode parallelizability
180. Random123 dimension scalability test
181. Sprng vectorized stream generation
182. Disperstone hash mixing speed check

### Hash Function Suite
183. CityHash variant family benchmark
184. MurmurHash finalizer avalanche property
185. xxHash SIMD block processing count
186. FNV-1a byte-wise folding comparison
187. SuperFastHash instruction count analysis
188. Jenkins lookup3 carry propagation cost
189. DJB2 XOR fold simplicity measurement
190. SDBM bitwise multiply emulation price
191. ELF-style CRC-32 polynomial division
192. Polynomial rolling hash incremental update
193. Rabin fingerprint sliding window timing
194. Karp-Rabin modulo inversion cost
195. Zobrist hashing update locality check
196. SipHash ARM NEON acceleration path

### Modern Cryptography
197. AES-NI GCM mode authentication speed
198. SHA-256 AVX2 block expansion throughput
199. Blake3 portability vs speed balance
200. MD5 compatibility legacy system support

## 3. Web Interface & Live Dashboards (201-300)

### Streaming Protocols
201. WebSocket live progress streaming protocol
202. SSE event source for progressive updates
203. HTTP/2 server push result fragments
204. GraphQL subscription for benchmark streams
205. gRPC bidirectional flow control channels
206. JSON Lines batched progress accumulation
207. MessagePack binary payload compression
208. CBOR selective field encoding in streams
209. Protocol Buffers schema definition sharing
210. FlatBuffers zero-copy shared memory transport
211. Cap'n RPC structured capability invocation
212. Thrift compact protocol bandwidth saving
213. Avro row-oriented storage layout efficiency
214. Parquet columnar pruning benefit logging
215. Arrow memory-mapped dataframe sharing
216. Feather interchange format simplicity
217. HDF5 hierarchical dataset organization
218. NetCDF self-describing scientific data
219. JSON5 human-editable config flexibility
220. TOML table ordering preservation
221. YAML anchor reuse for fixture templates
222. INI section grouping for scenarios
223. Properties file locale substitution
224. XML schema validation overhead marking
225. RDF graph triple store queries
226. Turtle prefix shortening for namespaces
227. N-Triples line-by-line parsing speed
228. JSON-LD context resolution latency
229. Linked data fragment URI dereferencing
230. ActivityPub actor federation latency

### Messaging & IPC
231. Matrix room history replay performance
232. IRC message relay hop timing
233. XMPP stanza routing overhead
234. MQTT broker publish-subscribe latency
235. CoAP blockwise transfer chunking
236. DDS sample identity freshness check
237. ZeroMQ bus topology fanout
238. Nanomsg SPAPC multicast efficiency
239. Pipe pair local IPC latency
240. Unix domain socket stream vs datagram
241. Named pipe cross-process communication
242. Eventfd signalfd synchronization primitives
243. Timerfd timeout accuracy measurement
244. Inotify file change detection overhead
245. kqueue edge-triggered notification timing
246. epoll LT vs ET level modes comparison
247. io_uring submission ring latency floor
248. AIO overlapped operation completion rate
249. Select timeout resolution accuracy
250. Poll scalability threshold measurement

### Filesystem Operations
251. Devpoll event mask efficiency check
252. Port polling implementation cost analysis
253. kqueue note filtering precision
254. syscalls::copy_file_range splice efficiency
255. sendfile zero-copy page cache use
256. scatter-gather I/O vscattervwritev cost
257. preadv pwritev positionless reads advantage
258. directio O_DIRECT alignment requirements
259. buffered I/O madvise random vs sequential
260. posix_fadvise prefetch hints effect
261. posix_fallocate space reservation speed
262. ftruncate growth shrink reversal time
263. flock advisory lock deadlock detection
264. fcntl record lock region overlap test
265. utime nanosecond timestamp precision
266. chown permission change side effect log
267. chmod group owner bit manipulation
268. umask file creation mask inheritance
269. link hard link inode sharing check
270. symlink relative path resolution cost
271. readlink canonical path reconstruction
272. stat lstat metadata difference measurement
273. fstat descriptor-based stat speed gain
274. lseek offset seeking penalty analysis
275. dup dup2 fd duplication overhead
276. close fd release syscall latency
277. openat directory relative openness
278. faccessat permission check virtualization
279. unlink remove directory entry atomicity
280. rmdir empty dir removal guarantee
281. mkdir mode permission bits effect
282. chmod AT_REMOVEDIR subtree traversal
283. fchmod fd-only permission update
284. fchown fd ownership transfer check
285. rename atomic cross-link movement
286. renameat renamedir relative operations
287. linkat newdir reference creation
288. symlinkat newpath symbolic creation
289. faccessat2 flags extended permissions
290. openat2 resolve restrictions exploration

### Virtualization & Mounting
291. mount bind remount idempotence check
292. umount lazy detached resource release
293. fsync dirty page flush guarantee
294. fdatasync metadata only sync option
295. syncfs whole filesystem consistency
296. ioctl custom device command timing
297. fcntl casefcntl command codes
298. prctl process control extension calls
299. setrlimit resource soft hard limits
300. getrlimit current usage inspection

## 4. Test Automation & CI Integration (301-400)

### CI/CD Platforms
301. GitHub Actions workflow template provision
302. GitLab CI pipeline snippet library
303. Jenkins declarative stage configuration
304. CircleCI context variable injection
305. Travis matrix build parallel execution
306. AppVeyor Windows subsystem logging
307. Azure Pipelines task download caching
308. Bitbucket steps conditional branching
309. Buildkite pipeline environment secrets
310. Drone secrets mounted as files
311. Wercker container build layers
312. Heroku dyno ephemeral filesystem warning
313. Netlify build hooks predeploy triggers
314. Vercel edge function cold start cost
315. Cloudflare Workers KV read latency
316. Deno deploy bundle analyze size
317. Bun quickstart execution time savings
318. denoland mod.ts bare import cost
319. Node.js --input-type=module flag
320. TypeScript tsconfig strictness effect

### Bundlers & Transpilers
321. ESBuild minify bundle size analysis
322. Rollup tree shaking dead code ratio
323. Webpack chunk splitting lazy load
324. Vite HMR update latency measurement
325. Parcel asset pipeline caching strategy
326. Snowpack ESM runtime import timing
327. Esbuild plugin hook order inspection
328. SWC transpiler AST mutation speed
329. Oxc parser fast path discovery
330. biome formatter opinionated ruleset

### Linters & Code Quality
331. Prettier plugin ecosystem compatibility
332. ESLint rule complexity scorecard
333. Turborepo monorepo dependency graph
334. Nx task execution caching layers
335. Turbopack webpack alternative loader
336. rspack lightning CSS processing
337. swc core WASM simd optimization
338. oxc_linter rust rewrite benefit
339. biome_lint rule coverage checklist

### Testing Frameworks
340. vitest inline assertion snapshot diff
341. jest module mock timing overhead
342. playwright browser launch slow path
343. Cypress component rendering freeze
344. puppeteer headless chrome startup
345. selenium grid node assignment cost
346. webdriverio capability negotiation
347. nightwatch e2e keyword abstraction
348. cucumber gherkin step definition glue

### Assertion Libraries
349. assert plus deep equality nesting
350. chai expect style fluent syntax
351. tape simple line-by-line reporter
352. mocha async promise handling
353. qunit module scoping organization
354. jasmine spy mock creation syntax
355. sinon stub method replacement
356. ava parallel test concurrency limiter
357. node:test built-in inspector ready

### Coverage & Reporting
358. vitest coverage v8 instrument speed
359. istanbul js coverage branch tracking
360. c8 v8 profiler output brevity
361. nyc reporter lcov json html
362. codecov uploader token expiry
363. coveralls github integration lag
364. codacy patch line coverage enforcement
365. sonarqube quality gate breach
366. deepscan static analysis comment
367. codereview bot review request
368. pullapprove requirement check

### Package Management
369. changesets semantic version bump
370. lerna bootstrap workspace packages
371. npm workspaces linking local pkgs
372. pnpm store deduplication efficiency
373. yarn pnp zero hoist mode speed
374. deno install remote URL caching
375. bun install symlink hard link
376. cargo install binary path precedence
377. go mod tidy dependency graph
378. java mvn dependency mediation
379. python pip req freeze pinning
380. ruby bundler lockfile consistency
381. php composer vendor autoload
382. lua luarocks rockspec manifest
383. rustup toolchain override local
384. nvm node version switching
385. pyenv python shim delegation
386. rbenv ruby shim lookup chain
387. go version global swap
388. dotnet global tool installation

### Build Tools
389. jbang script executable shebang
390. just runner make alternative
391. task GNU make dropin compatible
392. npm scripts shell portability
393. yarn workspaces folders true
394. pnpm overrides peer resolution
395. deno lint --rules config
396. deno fmt tab-width setting
397. bun test --coverage html report
398. bun run --env dotenv load
399. cargo expand macro expansion dump
400. cargo clippy --fix automigrate

## 5-10. Additional Categories

Items 401–1000 were never written down in this repository. Their intended categories:
- Data Structures & Algorithms (401-500)
- Optimization Patterns & Techniques (501-600)
- Memory Management & Allocation (601-700)
- Concurrency & Parallelism (701-800)
- Security & Stability (801-900)
- Monitoring & Observability (901-1000)

## Priority Items (⭐)

Based on immediate value for airbug-bench:

1. **Cache miss rate instrumentation via hardware counters** (#5)
2. **Lock-free queue Michael-Scott** (#701)
3. **Branch misprediction counting** (#6)
4. **SIMD utilization metrics extraction** (#7)
5. **Post-quantum cryptography benchmarks** (#892-900)
6. **Per-sample variance tracking** (#2)
7. **Hardware signature fingerprinting** (#119)
8. **OpenTelemetry SDK integration** (#905)

---

*Generated from mcp-rag corpus - airbug/roadmap-*
*Items in this file: 1–400*
