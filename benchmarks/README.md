# VelesDB Benchmark Kit

Benchmark suite comparing VelesDB against pgvector (HNSW).

## Quick Start

```bash
# 1. Start pgvector (Docker required)
docker-compose up -d

# 2. Install dependencies
pip install -r requirements.txt

# 3. Run benchmark
python benchmark_recall.py --vectors 10000 --clusters 50
```

## Benchmark Script

**`benchmark_recall.py`** - Main benchmark comparing VelesDB vs pgvector HNSW

```bash
# Options
python benchmark_recall.py --vectors 10000    # Dataset size
python benchmark_recall.py --dim 768          # Vector dimension
python benchmark_recall.py --queries 100      # Number of queries
python benchmark_recall.py --clusters 50      # Data clusters (realistic)
python benchmark_recall.py --velesdb-only     # Skip pgvector
```

## Test Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| `--vectors` | 10,000 | Number of vectors to insert |
| `--dim` | 768 | Vector dimension (OpenAI/Cohere-sized) |
| `--queries` | 100 | Number of search queries |
| `--top-k` | 10 | Results per query |
| `--clusters` | 50 | Data clusters (realistic embeddings) |

## Methodology

### Data Generation
- **Clustered vectors**: Realistic embeddings with natural clusters
- L2-normalized (simulating real AI embeddings)
- Same dataset for both databases

### Recall Calculation
- Ground truth via brute-force exact search
- Recall@k = |predicted ∩ ground_truth| / k

## Latest Results (v0.4.1)

| Dataset | VelesDB Recall | pgvector Recall | VelesDB P50 | pgvector P50 | Speedup |
|---------|----------------|-----------------|-------------|--------------|---------|
| 1,000 | 100.0% | 100.0% | **0.5ms** | 50ms | **100x** |
| 10,000 | 99.0% | 100.0% | **2.5ms** | 50ms | **20x** |
| 100,000 | 97.8% | 100.0% | **4.3ms** | 50ms | **12x** |

## Hardware Requirements

- **Minimum**: 8GB RAM, 4 CPU cores
- **Recommended**: 16GB RAM, 8+ CPU cores

## License

Elastic License 2.0 (ELv2)
