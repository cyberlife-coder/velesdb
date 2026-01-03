# VelesDB Release Process

Guide simplifié pour publier une nouvelle version de VelesDB.

## Architecture des Workflows

VelesDB utilise **3 workflows GitHub Actions** :

| Workflow | Trigger | Fonction |
|----------|---------|----------|
| `ci.yml` | Push/PR sur main | Tests, lint, security audit |
| `release.yml` | Tag `v*` | Publication complète |
| `bench-regression.yml` | Push sur main | Benchmarks de régression |

## Publier une Release

### 1. Préparer la version

```bash
# Mettre à jour la version dans Cargo.toml (workspace)
# version = "0.8.6"

# Mettre à jour CHANGELOG.md
```

### 2. Commit et tag

```bash
git add -A
git commit -m "release: v0.8.6"
git tag v0.8.6
git push origin main v0.8.6
```

### 3. Le workflow `release.yml` publie automatiquement

| Destination | Package |
|-------------|---------|
| **GitHub Release** | Binaries Linux/Windows/macOS + .deb |
| **crates.io** | velesdb-core, velesdb-cli, velesdb-server, velesdb-migrate, velesdb-mobile, tauri-plugin-velesdb |
| **PyPI** | velesdb |
| **npm** | @wiscale/velesdb-wasm, @wiscale/velesdb-sdk |

### 4. Vérifier le déploiement

- GitHub Actions : https://github.com/cyberlife-coder/VelesDB/actions
- GitHub Releases : https://github.com/cyberlife-coder/VelesDB/releases
- crates.io : https://crates.io/crates/velesdb-core
- PyPI : https://pypi.org/project/velesdb/
- npm : https://www.npmjs.com/package/@wiscale/velesdb-wasm

## Pre-releases

Pour une pre-release (beta, rc) :

```bash
git tag v0.9.0-beta.1
git push origin v0.9.0-beta.1
```

Le workflow détecte automatiquement les pre-releases et :
- Crée une GitHub Release marquée "Pre-release"
- **Ne publie PAS** sur crates.io/PyPI/npm

## Secrets requis

| Secret | Usage |
|--------|-------|
| `CARGO_REGISTRY_TOKEN` | Publication crates.io |
| `NPM_TOKEN` | Publication npm |
| `PYPI_API_TOKEN` | Publication PyPI (ou trusted publishing) |

## Dépannage

### Le workflow ne se déclenche pas

Vérifier que le tag suit le format `v[0-9]+.[0-9]+.[0-9]+` :
- ✅ `v0.8.6`
- ✅ `v1.0.0-beta.1`
- ❌ `0.8.6` (pas de "v")
- ❌ `v0.8` (version incomplète)

### Publication déjà existante

Si une version existe déjà sur crates.io/PyPI/npm, le workflow skip cette étape avec un message `⏭️ already published`.

### Force-update un tag

```bash
git tag -d v0.8.6
git tag v0.8.6
git push origin v0.8.6 --force
```

## Workflow manuel

Pour déclencher manuellement une release sans tag :

1. Aller sur GitHub Actions
2. Sélectionner "Release"
3. Cliquer "Run workflow"
4. Entrer la version (ex: `0.8.6`)
