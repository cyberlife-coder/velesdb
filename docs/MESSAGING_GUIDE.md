# VelesDB Messaging Guide

> ⚠️ **DOCUMENT INTERNE - WISCALE FRANCE**  
> **NE PAS DIFFUSER PUBLIQUEMENT**

---

> **Version**: 1.0  
> **Date**: 2026-01-20  
> **Owner**: Marketing & Developer Relations - Wiscale France  
> **Classification**: 🔒 Interne

---

## 🎯 Core Positioning

### Primary Tagline

> **"The Local Knowledge Engine for AI Agents"**

### Secondary Tagline (Technical)

> **"Vector + Graph Fusion • 57µs Search • Single Binary • Privacy-First"**

### Elevator Pitch (30 seconds)

> VelesDB is the only embedded database that **natively fuses vector search with knowledge graphs** in a single 15MB binary. While others force you to glue multiple systems together, VelesDB gives you **semantic search AND relationship traversal** in one query—at **57 microseconds latency**, 1000x faster than cloud alternatives. It runs everywhere: server, browser, mobile, desktop. Zero cloud dependencies. Your data stays local.

---

## 🔑 Key Messages

### For Developers

| Message | Proof Point |
|---------|-------------|
| **Microsecond latency** | 57µs p50 vector search (benchmarked) |
| **Single binary simplicity** | 15MB, zero dependencies, `cargo install` |
| **Vector + Graph in ONE query** | `WHERE similarity() > 0.8` in MATCH clauses |
| **Works everywhere** | Server, WASM, iOS, Android, Tauri |
| **SQL you already know** | VelesQL: familiar syntax, powerful extensions |

### For Technical Decision Makers

| Pain Point | VelesDB Solution |
|------------|------------------|
| "RAG needs both semantic search AND relationships" | Native vector + graph fusion |
| "Cloud latency kills our agent UX" | Local-first, 57µs search |
| "Too many moving parts" | Single binary, no cluster |
| "Privacy/compliance concerns" | Zero cloud, data stays local |
| "Complex deployment" | `cargo install` or single Docker image |

### For Enterprise (Premium)

| Enterprise Need | Premium Solution |
|-----------------|------------------|
| Data security | AES-256-GCM encryption at rest |
| High availability | Raft consensus, <5s failover |
| Compliance | Audit logging, RBAC, tenant isolation |
| Operations | WebAdmin UI, Prometheus, Helm/Operator |
| AI automation | Agent hooks (on_add_node, on_query) |

---

## 📝 Tone of Voice

### DO ✅

- **Be confident** — We have real benchmarks to back claims
- **Be technical but accessible** — Developers appreciate precision
- **Be result-oriented** — Focus on what users achieve, not feature lists
- **Be honest** — Acknowledge trade-offs when relevant
- **Use concrete numbers** — "57µs" not "fast", "15MB" not "small"

### DON'T ❌

- ~~Hype without substance~~ ("revolutionary", "game-changing")
- ~~Attack competitors directly~~ (compare features, not companies)
- ~~Overclaim~~ (if we don't have a benchmark, don't claim it)
- ~~Use enterprise jargon~~ ("leverage synergies", "paradigm shift")
- ~~Promise features not shipped~~ (mark as "Coming in vX.Y")

---

## 🏷️ Keywords & Phrases

### Use These ✅

| Keyword | Context |
|---------|---------|
| **Local Knowledge Engine** | Primary positioning |
| **Vector + Graph Fusion** | Key differentiator |
| **Single binary** | Simplicity |
| **Privacy-first** | Data sovereignty |
| **Embedded database** | Architecture |
| **AI Agent memory** | Use case |
| **Microsecond latency** | Performance |
| **VelesQL** | Query language |
| **Source available (ELv2)** | Licensing |

### Avoid These ❌

| Avoid | Use Instead |
|-------|-------------|
| "Open source" (for Core) | "Source available (ELv2)" |
| "Serverless" | "Embedded" or "local-first" |
| "NoSQL" | "Vector + Graph database" |
| "AI-powered database" | "Database for AI agents" |
| "Blazing fast" | "57µs latency" (be specific) |

---

## 🆚 Competitive Positioning

### vs. Vector-Only Databases (Qdrant, Pinecone, Milvus)

> "Vector-only databases miss relationships. When your agent asks 'Who wrote this document?', similarity search fails. VelesDB's native graph traversal answers relationship questions without separate systems."

### vs. Graph Databases (Neo4j)

> "Neo4j is a graph database with vector plugins. VelesDB is vector + graph unified from the ground up. One query language. One storage engine. One binary."

### vs. Cloud Solutions

> "Cloud vector DBs add 50-100ms per query. 10 retrievals = 1+ second delay. VelesDB at 57µs gives you 1000x faster agent responses—and your data never leaves your infrastructure."

---

## 📊 Proof Points (Use in Marketing)

| Claim | Source |
|-------|--------|
| **57µs p50 latency** | Internal benchmarks, reproducible |
| **15MB binary size** | Release artifact measurement |
| **100% recall @ 10** | HNSW benchmark suite |
| **1M vectors in 2GB RAM** | Stress test results |
| **WASM < 5MB gzipped** | wasm-pack build output |

---

## 🎨 Visual Identity

### Logo Usage

- Primary: Wolf icon + "VelesDB" wordmark
- Icon only: Wolf silhouette (for favicons, social)
- Colors: Deep purple (#6B21A8), White, Black

### Code Snippet Style

Always show the **differentiator** first:

```sql
-- ❌ Don't lead with basic search
SELECT * FROM docs WHERE vector NEAR $query;

-- ✅ Lead with vector + graph fusion
MATCH (d:Document)-[:AUTHORED_BY]->(p:Person)
WHERE similarity(d.embedding, $question) > 0.8
RETURN p.name, p.email;
```

---

## 📣 Social Media Templates

### Twitter/X (280 chars)

> 🐺 VelesDB: The Local Knowledge Engine for AI Agents
> 
> ✅ Vector + Graph in ONE query
> ✅ 57µs latency (1000x faster than cloud)
> ✅ 15MB single binary
> ✅ Runs everywhere: server, browser, mobile
> 
> github.com/cyberlife-coder/VelesDB

### LinkedIn (Professional)

> Excited to share VelesDB—the first embedded database to natively fuse vector search with knowledge graphs.
> 
> Why does this matter for AI agents?
> 
> 1️⃣ Semantic search alone isn't enough. Agents need relationships.
> 2️⃣ Cloud latency kills UX. Local-first means 57µs, not 50ms.
> 3️⃣ Simplicity wins. One binary, zero dependencies.
> 
> Check it out: https://velesdb.com

---

## 📧 Email Signature Block

```
[Name]
[Title] | VelesDB by Wiscale

🐺 The Local Knowledge Engine for AI Agents
🔗 velesdb.com | github.com/cyberlife-coder/VelesDB
📩 contact@wiscale.fr
```

---

## 🔄 Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01-20 | Initial messaging guide |
