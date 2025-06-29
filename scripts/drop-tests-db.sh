#!/bin/bash

# Script Bash pour supprimer les bases de données PostgreSQL dont le nom est un UUID
# Assurez-vous que psql est dans votre PATH ou spécifiez le chemin complet

# Configuration de la connexion
CONNECTION_STRING="postgresql://postgres:password@localhost:5432/postgres"

# Couleurs pour l'affichage
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

# Fonction pour vérifier si une chaîne est un UUID valide
is_uuid() {
    local uuid_pattern='^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$'
    [[ $1 =~ $uuid_pattern ]]
}

# Fonction pour exécuter une commande psql
execute_psql() {
    local query="$1"
    local result
    
    if ! result=$(psql "$CONNECTION_STRING" -t -c "$query" 2>&1); then
        echo -e "${RED}Erreur psql: $result${NC}" >&2
        return 1
    fi
    echo "$result"
    return 0
}

# Fonction pour afficher un message d'erreur et quitter
error_exit() {
    echo -e "${RED}ERREUR: $1${NC}" >&2
    exit 1
}

echo -e "${GREEN}=== Script de suppression des bases de données UUID ===${NC}"

# Test de connexion
echo -e "${YELLOW}Test de connexion à PostgreSQL...${NC}"
if ! execute_psql "SELECT version();" > /dev/null; then
    error_exit "Impossible de se connecter à PostgreSQL. Vérifiez votre chaîne de connexion."
fi
echo -e "${GREEN}Connexion réussie!${NC}"

# Récupération de la liste des bases de données
echo -e "${YELLOW}Récupération de la liste des bases de données...${NC}"
db_list_query="SELECT datname FROM pg_database WHERE datistemplate = false AND datname NOT IN ('postgres', 'template0', 'template1');"

if ! databases=$(execute_psql "$db_list_query"); then
    error_exit "Impossible de récupérer la liste des bases de données."
fi

# Nettoyage et filtrage des bases de données avec des noms UUID
uuid_databases=()
while IFS= read -r line; do
    db_name=$(echo "$line" | tr -d '[:space:]')
    if [[ -n "$db_name" ]] && is_uuid "$db_name"; then
        uuid_databases+=("$db_name")
    fi
done <<< "$databases"

if [[ ${#uuid_databases[@]} -eq 0 ]]; then
    echo -e "${YELLOW}Aucune base de données avec un nom UUID trouvée.${NC}"
    exit 0
fi

# Affichage des bases de données à supprimer
echo -e "\n${CYAN}Bases de données UUID trouvées:${NC}"
for db in "${uuid_databases[@]}"; do
    echo -e "  ${WHITE}- $db${NC}"
done

# Demande de confirmation
echo -e "\n${RED}ATTENTION: Cette opération va supprimer définitivement ces bases de données!${NC}"
read -p "Voulez-vous continuer? (oui/non): " confirmation

if [[ "$confirmation" != "oui" ]]; then
    echo -e "${YELLOW}Opération annulée.${NC}"
    exit 0
fi

# Suppression des bases de données
echo -e "\n${YELLOW}Suppression des bases de données...${NC}"
success_count=0
error_count=0

for db_name in "${uuid_databases[@]}"; do
    echo -e "${CYAN}Suppression de: $db_name${NC}"
    
    # Terminer les connexions actives à la base de données
    terminate_query="SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '$db_name' AND pid <> pg_backend_pid();"
    execute_psql "$terminate_query" > /dev/null
    
    # Supprimer la base de données
    drop_query="DROP DATABASE IF EXISTS \"$db_name\";"
    if execute_psql "$drop_query" > /dev/null; then
        echo -e "  ${GREEN}✓ Supprimée avec succès${NC}"
        ((success_count++))
    else
        echo -e "  ${RED}✗ Erreur lors de la suppression${NC}"
        ((error_count++))
    fi
done

# Résumé
echo -e "\n${GREEN}=== Résumé ===${NC}"
echo -e "${GREEN}Bases de données supprimées avec succès: $success_count${NC}"
if [[ $error_count -gt 0 ]]; then
    echo -e "${RED}Erreurs rencontrées: $error_count${NC}"
fi
echo -e "${GREEN}Opération terminée.${NC}"

# Pause optionnelle (décommentez si nécessaire)
# read -p "Appuyez sur Entrée pour continuer..."