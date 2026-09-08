#!/usr/bin/env bash
# Rejoue localement les portes que la CI exécute — en les LISANT dans ci.yml.
#
# POURQUOI DÉRIVER PLUTÔT QUE RECOPIER
# ------------------------------------
# Une liste de commandes recopiée à la main dérive de la CI, et la dérive est
# silencieuse : elle se manifeste par une porte locale verte suivie d'une CI
# rouge. C'est arrivé — `cargo clippy -p velesdb-core --lib` passait pendant que
# la CI, qui lance `--workspace --all-targets`, compilait le code de TEST et
# refusait. Ce script lit donc les `run:` du workflow ; il ne les connaît pas.
#
# Une étape que ce script ne peut pas rejouer est ANNONCÉE, jamais omise en
# silence : une porte sautée sans le dire vaut moins qu'une porte absente.
set -uo pipefail

# Surchargeable pour que la suite puisse pointer un workflow synthetique et
# prouver que la liste des portes est DERIVEE, pas recopiee.
WORKFLOW="${WORKFLOW:-.github/workflows/ci.yml}"
JOBS="${JOBS:-lint}"
LIST_ONLY=0
[ "${1:-}" = "--list" ] && LIST_ONLY=1

[ -f "$WORKFLOW" ] || { echo "ERREUR: $WORKFLOW introuvable — lancer depuis la racine du dépôt" >&2; exit 2; }

steps_json=$(python3 - "$WORKFLOW" "$JOBS" <<'PY'
import json, sys, yaml
wf, jobs = sys.argv[1], sys.argv[2].split(",")
doc = yaml.safe_load(open(wf))
out = []
for job in jobs:
    spec = (doc.get("jobs") or {}).get(job)
    if spec is None:
        print(f"ERREUR: job '{job}' absent de {wf}", file=sys.stderr); sys.exit(2)
    for st in spec.get("steps", []):
        run = st.get("run")
        if run is None:
            continue
        # YAML rend `run: true` comme un booleen ; sans coercition le lecteur
        # plante et — bien pire — la boucle appelante traite zero etape en
        # silence tout en annoncant un succes.
        run = run if isinstance(run, str) else str(run)
        out.append({"job": job, "name": st.get("name", "(sans nom)"), "run": run})
json.dump(out, sys.stdout)
PY
) || exit 2

# Ce qu'un poste de travail ne peut pas rejouer : l'infrastructure du runner.
skippable='GITHUB_|RUNNER_|apt-get|actions/|\$\{\{'

total=0; ran=0; failed=0; skipped=0
declare -a failures=()

while IFS=$'\t' read -r job name run; do
  total=$((total+1))
  cmd=$(printf '%s' "$run" | base64 -d)
  if printf '%s' "$cmd" | grep -qE "$skippable"; then
    skipped=$((skipped+1))
    printf '  ⤼ SAUTÉE  [%s] %s\n     (infrastructure du runner, non rejouable localement)\n' "$job" "$name"
    continue
  fi
  if [ "$LIST_ONLY" = "1" ]; then
    printf '  · [%s] %s\n' "$job" "$name"
    continue
  fi
  printf '  ▶ %s ... ' "$name"
  if out=$(bash -c "$cmd" 2>&1); then
    ran=$((ran+1)); printf 'ok\n'
  else
    failed=$((failed+1)); failures+=("$name")
    printf 'ÉCHEC\n'
    printf '%s\n' "$out" | tail -15 | sed 's/^/       /'
  fi
done < <(printf '%s' "$steps_json" | python3 -c '
import base64, json, sys
# base64 pour la commande : un aller-retour par sequences d echappement
# transforme les continuations de ligne `\` en `\`+`n` litteral, et le shell
# recoit un argument `n`. Mesure faite — le gate rendait un FAUX ROUGE sur
# clippy, ce qui est la facon la plus sure de faire desactiver une porte.
for s in json.load(sys.stdin):
    print("\t".join([s["job"], s["name"], base64.b64encode(s["run"].encode()).decode()]))
')

echo
if [ "$LIST_ONLY" = "1" ]; then
  if [ "$total" -eq 0 ]; then
    echo "ERREUR: aucune étape lue dans $WORKFLOW (job(s): $JOBS)" >&2
    exit 2
  fi
  echo "$total étape(s) déclarée(s), $skipped non rejouable(s) localement."
  exit 0
fi
# Un gate qui annonce un succes sans avoir rien execute est pire qu'un gate
# absent : l'operateur croit avoir verifie. Ce cas s'est produit — le lecteur
# plantait sur une valeur inattendue et la boucle tournait a vide en rapportant
# « toutes les portes passent ». Zero etape traitee est desormais une ERREUR.
if [ "$total" -eq 0 ]; then
  echo "ERREUR: aucune étape lue dans $WORKFLOW (job(s): $JOBS) — refus de rapporter un succès qui n'a rien vérifié" >&2
  exit 2
fi
echo "Portes rejouées: $ran — échecs: $failed — non rejouables: $skipped (sur $total déclarées)"
if [ "$failed" -gt 0 ]; then
  printf 'ÉCHOUÉ: %s\n' "${failures[*]}" >&2
  exit 1
fi
echo "Toutes les portes rejouables passent. Les $skipped autres ne sont vérifiées QUE par la CI."
exit 0
